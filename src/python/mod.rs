//! Python bindings for Liath
//!
//! This module provides PyO3 bindings to expose Liath's functionality to Python.
//! The key feature is `execute()` which runs Lua code safely.

use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use pyo3::types::{PyDict, PyList};

use crate::{EmbeddedLiath, Config};
use crate::lua::LuaValidator;

// Python exceptions for Liath errors
pyo3::create_exception!(liath, LiathError, pyo3::exceptions::PyException);
pyo3::create_exception!(liath, LiathValidationError, LiathError);
pyo3::create_exception!(liath, LiathRuntimeError, LiathError);

/// Python-exposed validation error
#[pyclass(name = "ValidationError")]
#[derive(Clone)]
pub struct PyValidationError {
    #[pyo3(get)]
    pub error_type: String,
    #[pyo3(get)]
    pub message: String,
    #[pyo3(get)]
    pub line: Option<usize>,
    #[pyo3(get)]
    pub suggestion: String,
    #[pyo3(get)]
    pub code_snippet: Option<String>,
}

#[pymethods]
impl PyValidationError {
    fn __repr__(&self) -> String {
        format!("ValidationError(type='{}', line={:?}, message='{}')",
            self.error_type, self.line, self.message)
    }
}

/// Python-exposed validation result
#[pyclass(name = "ValidationResult")]
#[derive(Clone)]
pub struct PyValidationResult {
    #[pyo3(get)]
    pub valid: bool,
    #[pyo3(get)]
    pub errors: Vec<PyValidationError>,
    #[pyo3(get)]
    pub warnings: Vec<String>,
}

#[pymethods]
impl PyValidationResult {
    fn __repr__(&self) -> String {
        format!("ValidationResult(valid={}, errors={})", self.valid, self.errors.len())
    }
}

/// Search result
#[pyclass(name = "SearchResult")]
#[derive(Clone)]
pub struct PySearchResult {
    #[pyo3(get)]
    pub id: String,
    #[pyo3(get)]
    pub content: String,
    #[pyo3(get)]
    pub distance: f32,
}

#[pymethods]
impl PySearchResult {
    fn __repr__(&self) -> String {
        format!("SearchResult(id='{}', distance={:.4})", self.id, self.distance)
    }
}

/// Main Liath database class
///
/// Note: This class is not thread-safe and should only be used from a single thread.
#[pyclass(name = "Liath", unsendable)]
pub struct PyLiath {
    inner: EmbeddedLiath,
    validator: LuaValidator,
    runtime: tokio::runtime::Runtime,
}

#[pymethods]
impl PyLiath {
    /// Create a new Liath instance
    ///
    /// Args:
    ///     data_dir: Path to the data directory
    #[new]
    #[pyo3(signature = (data_dir = "./data"))]
    fn new(data_dir: &str) -> PyResult<Self> {
        let config = Config {
            data_dir: data_dir.into(),
            ..Default::default()
        };

        let inner = EmbeddedLiath::new(config)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create Liath: {}", e)))?;

        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create runtime: {}", e)))?;

        Ok(PyLiath {
            inner,
            validator: LuaValidator::new(),
            runtime,
        })
    }

    /// Validate Lua code without executing it
    ///
    /// Args:
    ///     code: Lua code to validate
    ///
    /// Returns:
    ///     ValidationResult with errors and warnings
    fn validate(&self, code: &str) -> PyValidationResult {
        let result = self.validator.validate(code);
        PyValidationResult {
            valid: result.valid,
            errors: result.errors.into_iter().map(|e| PyValidationError {
                error_type: format!("{:?}", e.error_type),
                message: e.message,
                line: e.line,
                suggestion: e.suggestion,
                code_snippet: e.code_snippet,
            }).collect(),
            warnings: result.warnings.into_iter().map(|w| w.message).collect(),
        }
    }

    /// Execute Lua code safely
    ///
    /// This is the core operation - agents write Lua code to query memory.
    ///
    /// Args:
    ///     code: Lua code to execute
    ///     user_id: User ID for authorization (default: "default")
    ///
    /// Returns:
    ///     dict with 'success', 'value', and 'error' keys
    #[pyo3(signature = (code, user_id = "default"))]
    fn execute<'py>(&self, py: Python<'py>, code: &str, user_id: &str) -> PyResult<PyObject> {
        let result = PyDict::new_bound(py);

        // Step 1: Validate
        let validation = self.validator.validate(code);
        if !validation.valid {
            result.set_item("success", false)?;
            result.set_item("value", py.None())?;

            let error = PyDict::new_bound(py);
            error.set_item("type", "validation")?;
            error.set_item("message", format!("{} validation error(s)", validation.errors.len()))?;

            if let Some(first_error) = validation.errors.first() {
                error.set_item("suggestion", &first_error.suggestion)?;
                error.set_item("line", first_error.line)?;
            }

            let errors_list = PyList::new_bound(py, Vec::<PyObject>::new());
            for e in validation.errors {
                let err_dict = PyDict::new_bound(py);
                err_dict.set_item("type", format!("{:?}", e.error_type))?;
                err_dict.set_item("message", e.message)?;
                err_dict.set_item("line", e.line)?;
                err_dict.set_item("suggestion", e.suggestion)?;
                errors_list.append(err_dict)?;
            }
            error.set_item("errors", errors_list)?;
            result.set_item("error", error)?;

            return Ok(result.into());
        }

        // Step 2: Execute
        let executor = self.inner.query_executor();
        let exec_result = self.runtime.block_on(async {
            executor.execute(code, user_id).await
        });

        match exec_result {
            Ok(value) => {
                result.set_item("success", true)?;
                result.set_item("error", py.None())?;

                // Try to parse as JSON
                match serde_json::from_str::<serde_json::Value>(&value) {
                    Ok(json) => {
                        result.set_item("value", json_to_py(py, &json)?)?;
                    }
                    Err(_) => {
                        result.set_item("value", &value)?;
                    }
                }
            }
            Err(e) => {
                result.set_item("success", false)?;
                result.set_item("value", py.None())?;

                let error = PyDict::new_bound(py);
                error.set_item("type", "runtime")?;
                error.set_item("message", e.to_string())?;
                error.set_item("suggestion", "Check the Lua code for errors")?;
                result.set_item("error", error)?;
            }
        }

        Ok(result.into())
    }

    /// Store a value with automatic embedding generation
    #[cfg(feature = "embedding")]
    fn store(&self, namespace: &str, key: &str, content: &str) -> PyResult<()> {
        // Generate a unique ID from the key hash
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        let id = hasher.finish();

        self.inner.store_with_embedding(namespace, id, key.as_bytes(), content)
            .map_err(|e| PyRuntimeError::new_err(format!("Store failed: {}", e)))
    }

    /// Semantic search - find similar content
    #[cfg(feature = "embedding")]
    #[pyo3(signature = (namespace, query, limit = 10))]
    fn search(&self, namespace: &str, query: &str, limit: usize) -> PyResult<Vec<PySearchResult>> {
        let results = self.inner.semantic_search(namespace, query, limit)
            .map_err(|e| PyRuntimeError::new_err(format!("Search failed: {}", e)))?;

        Ok(results.into_iter().map(|(id, content, distance)| {
            PySearchResult {
                id: id.to_string(),
                content,
                distance,
            }
        }).collect())
    }

    /// Store a key-value pair
    fn put(&self, namespace: &str, key: &str, value: &str) -> PyResult<()> {
        self.inner.put(namespace, key.as_bytes(), value.as_bytes())
            .map_err(|e| PyRuntimeError::new_err(format!("Put failed: {}", e)))
    }

    /// Get a value by key
    fn get(&self, namespace: &str, key: &str) -> PyResult<Option<String>> {
        let result = self.inner.get(namespace, key.as_bytes())
            .map_err(|e| PyRuntimeError::new_err(format!("Get failed: {}", e)))?;

        Ok(result.map(|v| String::from_utf8_lossy(&v).to_string()))
    }

    /// Delete a key
    fn delete(&self, namespace: &str, key: &str) -> PyResult<()> {
        self.inner.delete(namespace, key.as_bytes())
            .map_err(|e| PyRuntimeError::new_err(format!("Delete failed: {}", e)))
    }

    /// List all namespaces
    fn list_namespaces(&self) -> Vec<String> {
        self.inner.query_executor().list_namespaces()
    }

    /// Create a new namespace
    #[cfg(feature = "vector")]
    #[pyo3(signature = (name, dimensions = 384, metric = "cosine"))]
    fn create_namespace(&self, name: &str, dimensions: usize, metric: &str) -> PyResult<()> {
        use usearch::{MetricKind, ScalarKind};

        let metric_kind = match metric.to_lowercase().as_str() {
            "euclidean" | "l2" => MetricKind::L2sq,
            _ => MetricKind::Cos,
        };

        self.inner.create_namespace(name, dimensions, metric_kind, ScalarKind::F32)
            .map_err(|e| PyRuntimeError::new_err(format!("Create namespace failed: {}", e)))
    }

    /// Generate embedding for text
    #[cfg(feature = "embedding")]
    fn embed(&self, text: &str) -> PyResult<Vec<f32>> {
        self.inner.generate_embedding(text)
            .map_err(|e| PyRuntimeError::new_err(format!("Embedding failed: {}", e)))
    }

    /// Get help text with available Lua functions
    fn help(&self) -> String {
        self.validator.format_help()
    }

    /// Save all data to disk
    fn save(&self) -> PyResult<()> {
        self.inner.save()
            .map_err(|e| PyRuntimeError::new_err(format!("Save failed: {}", e)))
    }

    /// Close the database
    fn close(&self) -> PyResult<()> {
        self.inner.close()
            .map_err(|e| PyRuntimeError::new_err(format!("Close failed: {}", e)))
    }

    fn __repr__(&self) -> String {
        format!("Liath(namespaces={})", self.list_namespaces().len())
    }
}

/// Convert serde_json::Value to PyObject
fn json_to_py(py: Python<'_>, value: &serde_json::Value) -> PyResult<PyObject> {
    match value {
        serde_json::Value::Null => Ok(py.None()),
        serde_json::Value::Bool(b) => Ok(b.into_py(py)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.into_py(py))
            } else if let Some(f) = n.as_f64() {
                Ok(f.into_py(py))
            } else {
                Ok(py.None())
            }
        }
        serde_json::Value::String(s) => Ok(s.into_py(py)),
        serde_json::Value::Array(arr) => {
            let items: Vec<PyObject> = arr.iter()
                .map(|v| json_to_py(py, v))
                .collect::<PyResult<Vec<_>>>()?;
            Ok(PyList::new_bound(py, items).into())
        }
        serde_json::Value::Object(obj) => {
            let dict = PyDict::new_bound(py);
            for (k, v) in obj {
                dict.set_item(k, json_to_py(py, v)?)?;
            }
            Ok(dict.into())
        }
    }
}

/// Python module definition
#[pymodule]
fn _liath(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyLiath>()?;
    m.add_class::<PyValidationResult>()?;
    m.add_class::<PyValidationError>()?;
    m.add_class::<PySearchResult>()?;
    m.add("LiathError", m.py().get_type_bound::<LiathError>())?;
    m.add("LiathValidationError", m.py().get_type_bound::<LiathValidationError>())?;
    m.add("LiathRuntimeError", m.py().get_type_bound::<LiathRuntimeError>())?;
    Ok(())
}
