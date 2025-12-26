//! Structured error types for Lua execution
//!
//! These errors are designed to be LLM-friendly, providing clear messages
//! and actionable suggestions that help AI agents fix their code.

use serde::{Deserialize, Serialize};

/// Result of validating and/or executing Lua code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Whether the execution was successful
    pub success: bool,
    /// The result value (if successful)
    pub value: Option<serde_json::Value>,
    /// Validation errors (if any)
    pub validation_errors: Vec<ValidationError>,
    /// Runtime error (if execution failed)
    pub runtime_error: Option<RuntimeError>,
}

impl ExecutionResult {
    /// Create a successful result
    pub fn success(value: serde_json::Value) -> Self {
        Self {
            success: true,
            value: Some(value),
            validation_errors: Vec::new(),
            runtime_error: None,
        }
    }

    /// Create a validation failure result
    pub fn validation_failed(errors: Vec<ValidationError>) -> Self {
        Self {
            success: false,
            value: None,
            validation_errors: errors,
            runtime_error: None,
        }
    }

    /// Create a runtime error result
    pub fn runtime_failed(error: RuntimeError) -> Self {
        Self {
            success: false,
            value: None,
            validation_errors: Vec::new(),
            runtime_error: Some(error),
        }
    }
}

/// Result of validating Lua code (without executing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the code is valid
    pub valid: bool,
    /// List of errors found
    pub errors: Vec<ValidationError>,
    /// List of warnings (non-fatal issues)
    pub warnings: Vec<ValidationWarning>,
    /// List of available functions (for LLM reference)
    pub available_functions: Vec<FunctionInfo>,
}

impl ValidationResult {
    /// Create a successful validation result
    pub fn valid() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            available_functions: Vec::new(),
        }
    }

    /// Create a failed validation result
    pub fn invalid(errors: Vec<ValidationError>) -> Self {
        Self {
            valid: false,
            errors,
            warnings: Vec::new(),
            available_functions: Vec::new(),
        }
    }

    /// Add available functions info (for LLM assistance)
    pub fn with_functions(mut self, functions: Vec<FunctionInfo>) -> Self {
        self.available_functions = functions;
        self
    }
}

/// A validation error with LLM-friendly details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Type of error
    pub error_type: ErrorType,
    /// Human-readable error message
    pub message: String,
    /// Line number where error occurred (1-indexed)
    pub line: Option<usize>,
    /// Column number where error occurred (1-indexed)
    pub column: Option<usize>,
    /// LLM-friendly suggestion for fixing the error
    pub suggestion: String,
    /// Code snippet showing the error context
    pub code_snippet: Option<String>,
}

impl ValidationError {
    /// Create a syntax error
    pub fn syntax(message: &str, line: Option<usize>, suggestion: &str) -> Self {
        Self {
            error_type: ErrorType::SyntaxError,
            message: message.to_string(),
            line,
            column: None,
            suggestion: suggestion.to_string(),
            code_snippet: None,
        }
    }

    /// Create a forbidden function error
    pub fn forbidden_function(function: &str, suggestion: &str) -> Self {
        Self {
            error_type: ErrorType::ForbiddenFunction,
            message: format!("Function '{}' is not allowed for security reasons", function),
            line: None,
            column: None,
            suggestion: suggestion.to_string(),
            code_snippet: None,
        }
    }

    /// Create an undefined variable error
    pub fn undefined_variable(variable: &str, did_you_mean: Option<&str>) -> Self {
        let suggestion = match did_you_mean {
            Some(s) => format!("Did you mean '{}'?", s),
            None => "Check that the variable is defined before use".to_string(),
        };
        Self {
            error_type: ErrorType::UndefinedVariable,
            message: format!("Undefined variable '{}'", variable),
            line: None,
            column: None,
            suggestion,
            code_snippet: None,
        }
    }

    /// Add code snippet context
    pub fn with_snippet(mut self, snippet: &str) -> Self {
        self.code_snippet = Some(snippet.to_string());
        self
    }

    /// Add line/column info
    pub fn at_location(mut self, line: usize, column: Option<usize>) -> Self {
        self.line = Some(line);
        self.column = column;
        self
    }
}

/// Types of validation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorType {
    /// Lua syntax error
    SyntaxError,
    /// Attempted to use a forbidden function (io, os, etc.)
    ForbiddenFunction,
    /// Reference to undefined variable
    UndefinedVariable,
    /// Type mismatch in function call
    TypeMismatch,
    /// Missing return statement
    MissingReturn,
    /// Code complexity exceeded limits
    ComplexityExceeded,
}

/// A non-fatal warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// Warning type
    pub warning_type: WarningType,
    /// Warning message
    pub message: String,
    /// Line number
    pub line: Option<usize>,
    /// Suggestion
    pub suggestion: String,
}

impl ValidationWarning {
    /// Create a missing return warning
    pub fn missing_return() -> Self {
        Self {
            warning_type: WarningType::MissingReturn,
            message: "Code does not have an explicit return statement".to_string(),
            line: None,
            suggestion: "Add 'return <value>' at the end to return a result".to_string(),
        }
    }

    /// Create an unused variable warning
    pub fn unused_variable(name: &str, line: Option<usize>) -> Self {
        Self {
            warning_type: WarningType::UnusedVariable,
            message: format!("Variable '{}' is defined but never used", name),
            line,
            suggestion: "Remove unused variables or use them in your code".to_string(),
        }
    }
}

/// Types of warnings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WarningType {
    /// Missing return statement
    MissingReturn,
    /// Unused variable
    UnusedVariable,
    /// Deprecated function
    DeprecatedFunction,
}

/// A runtime error with LLM-friendly details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeError {
    /// Type of runtime error
    pub error_type: RuntimeErrorType,
    /// Human-readable error message
    pub message: String,
    /// Lua stack trace (if available)
    pub lua_traceback: Option<String>,
    /// LLM-friendly suggestion for fixing the error
    pub suggestion: String,
}

impl RuntimeError {
    /// Create a namespace not found error
    pub fn namespace_not_found(namespace: &str, available: Vec<String>) -> Self {
        let suggestion = if available.is_empty() {
            "No namespaces exist. Create one first with create_namespace().".to_string()
        } else {
            format!(
                "Available namespaces: {}. Use one of these or create '{}'.",
                available.join(", "),
                namespace
            )
        };
        Self {
            error_type: RuntimeErrorType::NamespaceNotFound {
                namespace: namespace.to_string(),
                available,
            },
            message: format!("Namespace '{}' not found", namespace),
            lua_traceback: None,
            suggestion,
        }
    }

    /// Create a key not found error
    pub fn key_not_found(key: &str, namespace: &str) -> Self {
        Self {
            error_type: RuntimeErrorType::KeyNotFound {
                key: key.to_string(),
            },
            message: format!("Key '{}' not found in namespace '{}'", key, namespace),
            lua_traceback: None,
            suggestion: format!(
                "Check that the key exists. Use get() which returns nil for missing keys, or store a value first with put()."
            ),
        }
    }

    /// Create a type error
    pub fn type_error(expected: &str, got: &str, context: &str) -> Self {
        Self {
            error_type: RuntimeErrorType::TypeError {
                expected: expected.to_string(),
                got: got.to_string(),
            },
            message: format!("Type error in {}: expected {}, got {}", context, expected, got),
            lua_traceback: None,
            suggestion: format!(
                "Convert the value to {} or use a different function.",
                expected
            ),
        }
    }

    /// Create an authorization error
    pub fn unauthorized(function: &str, user: &str) -> Self {
        Self {
            error_type: RuntimeErrorType::AuthorizationDenied {
                function: function.to_string(),
                user: user.to_string(),
            },
            message: format!("User '{}' is not authorized to call '{}'", user, function),
            lua_traceback: None,
            suggestion: "Contact an administrator to grant permissions for this operation."
                .to_string(),
        }
    }

    /// Create a timeout error
    pub fn timeout(duration_ms: u64) -> Self {
        Self {
            error_type: RuntimeErrorType::Timeout,
            message: format!("Execution timed out after {}ms", duration_ms),
            lua_traceback: None,
            suggestion:
                "Simplify your query or process data in smaller batches to avoid timeouts."
                    .to_string(),
        }
    }

    /// Create a generic Lua error
    pub fn lua_error(message: &str) -> Self {
        Self {
            error_type: RuntimeErrorType::LuaError,
            message: message.to_string(),
            lua_traceback: None,
            suggestion: "Check the Lua code for errors. Ensure all functions are called correctly."
                .to_string(),
        }
    }

    /// Add traceback
    pub fn with_traceback(mut self, traceback: &str) -> Self {
        self.lua_traceback = Some(traceback.to_string());
        self
    }
}

/// Types of runtime errors
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeErrorType {
    /// Namespace does not exist
    NamespaceNotFound {
        namespace: String,
        available: Vec<String>,
    },
    /// Key not found in namespace
    KeyNotFound { key: String },
    /// Type mismatch
    TypeError { expected: String, got: String },
    /// User not authorized
    AuthorizationDenied { function: String, user: String },
    /// Execution timeout
    Timeout,
    /// Generic Lua error
    LuaError,
}

/// Information about an available function (for LLM reference)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    /// Function name
    pub name: String,
    /// Function signature (e.g., "put(namespace, key, value)")
    pub signature: String,
    /// Short description
    pub description: String,
    /// Return type
    pub returns: String,
    /// Example usage
    pub example: Option<String>,
}

impl FunctionInfo {
    /// Create function info
    pub fn new(name: &str, signature: &str, description: &str, returns: &str) -> Self {
        Self {
            name: name.to_string(),
            signature: signature.to_string(),
            description: description.to_string(),
            returns: returns.to_string(),
            example: None,
        }
    }

    /// Add example
    pub fn with_example(mut self, example: &str) -> Self {
        self.example = Some(example.to_string());
        self
    }
}

/// Get the list of available functions for Liath
pub fn available_functions() -> Vec<FunctionInfo> {
    vec![
        // Storage
        FunctionInfo::new("put", "put(namespace, key, value)", "Store a value", "nil")
            .with_example("put('config', 'theme', 'dark')"),
        FunctionInfo::new("get", "get(namespace, key)", "Retrieve a value", "string|nil")
            .with_example("local theme = get('config', 'theme')"),
        FunctionInfo::new("delete", "delete(namespace, key)", "Delete a key", "nil")
            .with_example("delete('config', 'old_key')"),
        // Semantic
        FunctionInfo::new(
            "store_with_embedding",
            "store_with_embedding(namespace, id, content)",
            "Store text with auto-generated embedding",
            "nil",
        )
        .with_example("store_with_embedding('docs', 'doc1', 'Hello world')"),
        FunctionInfo::new(
            "semantic_search",
            "semantic_search(namespace, query, limit)",
            "Search by text similarity",
            "list of {id, content, distance}",
        )
        .with_example("local results = semantic_search('docs', 'greeting', 5)"),
        // Utilities
        FunctionInfo::new(
            "json.encode",
            "json.encode(value)",
            "Convert Lua value to JSON string",
            "string",
        )
        .with_example("return json.encode({name = 'test'})"),
        FunctionInfo::new(
            "json.decode",
            "json.decode(string)",
            "Parse JSON string to Lua value",
            "table",
        )
        .with_example("local data = json.decode('{\"a\": 1}')"),
        FunctionInfo::new(
            "filter",
            "filter(list, fn)",
            "Filter list by predicate",
            "list",
        )
        .with_example("filter(items, function(x) return x.age > 18 end)"),
        FunctionInfo::new("map", "map(list, fn)", "Transform each list element", "list")
            .with_example("map(items, function(x) return x.name end)"),
        FunctionInfo::new(
            "reduce",
            "reduce(list, fn, initial)",
            "Reduce list to single value",
            "any",
        )
        .with_example("reduce(nums, function(a, b) return a + b end, 0)"),
        FunctionInfo::new("now", "now()", "Get current Unix timestamp", "number")
            .with_example("local timestamp = now()"),
        FunctionInfo::new("id", "id()", "Generate unique ID", "string")
            .with_example("local unique_id = id()"),
    ]
}

/// Blocked functions and their suggestions
pub fn blocked_functions() -> Vec<(&'static str, &'static str)> {
    vec![
        ("io.open", "File I/O is not allowed. Store data using put() instead."),
        ("io.read", "File I/O is not allowed. Retrieve data using get() instead."),
        ("io.write", "File I/O is not allowed. Store data using put() instead."),
        ("os.execute", "System commands are not allowed for security."),
        ("os.remove", "File deletion is not allowed. Use delete() for keys."),
        ("os.rename", "File operations are not allowed."),
        ("os.exit", "Exiting is not allowed."),
        ("require", "Loading external modules is not allowed."),
        ("loadfile", "Loading files is not allowed."),
        ("dofile", "Executing files is not allowed."),
        ("load", "Loading code strings is not allowed."),
        ("loadstring", "Loading code strings is not allowed."),
        ("debug.getinfo", "Debug functions are not allowed."),
        ("debug.sethook", "Debug functions are not allowed."),
        ("rawget", "Raw table access is not allowed. Use normal indexing."),
        ("rawset", "Raw table access is not allowed. Use normal indexing."),
        ("setmetatable", "Metatable manipulation is not allowed."),
        ("getmetatable", "Metatable access is not allowed."),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_result_success() {
        let result = ExecutionResult::success(serde_json::json!({"key": "value"}));
        assert!(result.success);
        assert!(result.value.is_some());
        assert!(result.validation_errors.is_empty());
        assert!(result.runtime_error.is_none());
    }

    #[test]
    fn test_validation_error_serialization() {
        let error = ValidationError::forbidden_function("os.execute", "Use Liath functions instead");
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("forbidden_function"));
        assert!(json.contains("os.execute"));
    }

    #[test]
    fn test_runtime_error_namespace() {
        let error = RuntimeError::namespace_not_found("myns", vec!["default".to_string()]);
        assert!(error.suggestion.contains("default"));
    }
}
