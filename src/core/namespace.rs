use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use crate::core::FjallWrapper;
use crate::vector::UsearchWrapper;
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
#[cfg(feature = "vector")]
use usearch::{MetricKind, ScalarKind};
#[cfg(not(feature = "vector"))]
#[derive(Clone, Copy)]
pub enum MetricKind { Cos, L2sq }
#[cfg(not(feature = "vector"))]
#[derive(Clone, Copy)]
pub enum ScalarKind { F32, F16 }

/// Metadata for a namespace, persisted to disk
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NamespaceMetadata {
    pub name: String,
    pub dimensions: usize,
    pub metric: String,
    pub scalar: String,
}

#[derive(Clone)]
pub struct Namespace {
    pub db: Arc<FjallWrapper>,
    pub vector_db: Arc<UsearchWrapper>,
}

impl Namespace {
    pub fn new(db: FjallWrapper, vector_db: UsearchWrapper) -> Self {
        Self { 
            db: Arc::new(db), 
            vector_db: Arc::new(vector_db) 
        }
    }
}

pub struct NamespaceManager {
    namespaces: Arc<RwLock<HashMap<String, Namespace>>>,
    data_dir: PathBuf,
    metadata_db: Arc<FjallWrapper>,
}

impl NamespaceManager {
    /// Create a new NamespaceManager with persistence
    pub fn new(data_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&data_dir)
            .context("Failed to create data directory")?;

        let metadata_db = FjallWrapper::new(data_dir.join("_metadata"))
            .context("Failed to create metadata database")?;

        let mut manager = Self {
            namespaces: Arc::new(RwLock::new(HashMap::new())),
            data_dir,
            metadata_db: Arc::new(metadata_db),
        };

        manager.load_existing()?;
        Ok(manager)
    }

    /// Load existing namespaces from persistent storage
    fn load_existing(&mut self) -> Result<()> {
        let mut loaded_count = 0;

        for result in self.metadata_db.iter() {
            let (key, value) = result?;
            let name = String::from_utf8(key)
                .context("Invalid namespace name in metadata")?;

            let metadata: NamespaceMetadata = serde_json::from_slice(&value)
                .context(format!("Failed to deserialize metadata for namespace '{}'", name))?;

            // Convert string metric/scalar back to enum types
            let metric = Self::parse_metric(&metadata.metric)?;
            let scalar = Self::parse_scalar(&metadata.scalar)?;

            // Open existing Fjall database
            let db = FjallWrapper::new(self.data_dir.join(&name))
                .context(format!("Failed to open Fjall for namespace '{}'", name))?;

            // Create vector index and try to load from disk
            let vector_db = UsearchWrapper::new(metadata.dimensions, metric, scalar)
                .context(format!("Failed to create UsearchWrapper for namespace '{}'", name))?;

            // Try to load vector index if it exists
            let vector_path = self.data_dir.join(&name).join("vectors.idx");
            if vector_path.exists() {
                if let Err(e) = vector_db.load(vector_path.to_str().unwrap()) {
                    tracing::warn!("Failed to load vector index for '{}': {}", name, e);
                }
            }

            let mut namespaces = self.namespaces.write().unwrap();
            namespaces.insert(name.clone(), Namespace::new(db, vector_db));
            loaded_count += 1;
            tracing::info!("Loaded namespace '{}' from disk", name);
        }

        if loaded_count > 0 {
            tracing::info!("Loaded {} namespaces from persistent storage", loaded_count);
        }

        Ok(())
    }

    /// Persist namespace metadata to disk
    fn persist_metadata(&self, name: &str, metadata: &NamespaceMetadata) -> Result<()> {
        let value = serde_json::to_vec(metadata)
            .context("Failed to serialize namespace metadata")?;
        self.metadata_db.put(name.as_bytes(), &value)
            .context("Failed to persist namespace metadata")?;
        Ok(())
    }

    /// Delete namespace metadata from disk
    fn delete_metadata(&self, name: &str) -> Result<()> {
        self.metadata_db.delete(name.as_bytes())
            .context("Failed to delete namespace metadata")?;
        Ok(())
    }

    /// Convert metric string to enum
    fn parse_metric(s: &str) -> Result<MetricKind> {
        match s {
            "cosine" | "Cos" => Ok(MetricKind::Cos),
            "euclidean" | "L2sq" => Ok(MetricKind::L2sq),
            _ => Err(anyhow::anyhow!("Unknown metric kind: {}", s)),
        }
    }

    /// Convert scalar string to enum
    fn parse_scalar(s: &str) -> Result<ScalarKind> {
        match s {
            "f32" | "F32" => Ok(ScalarKind::F32),
            "f16" | "F16" => Ok(ScalarKind::F16),
            _ => Err(anyhow::anyhow!("Unknown scalar kind: {}", s)),
        }
    }

    /// Convert metric enum to string for persistence
    fn metric_to_string(metric: MetricKind) -> &'static str {
        match metric {
            MetricKind::Cos => "cosine",
            MetricKind::L2sq => "euclidean",
            #[allow(unreachable_patterns)]
            _ => "cosine",
        }
    }

    /// Convert scalar enum to string for persistence
    fn scalar_to_string(scalar: ScalarKind) -> &'static str {
        match scalar {
            ScalarKind::F32 => "f32",
            ScalarKind::F16 => "f16",
            #[allow(unreachable_patterns)]
            _ => "f32",
        }
    }

    pub fn create_namespace(&self, name: &str, dimensions: usize, metric: MetricKind, scalar: ScalarKind) -> Result<()> {
        let mut namespaces = self.namespaces.write().unwrap();
        if namespaces.contains_key(name) {
            return Err(anyhow::anyhow!("Namespace '{}' already exists", name));
        }

        // Create namespace directory
        let ns_dir = self.data_dir.join(name);
        std::fs::create_dir_all(&ns_dir)
            .context(format!("Failed to create namespace directory '{}'", name))?;

        let db = FjallWrapper::new(&ns_dir)
            .context(format!("Failed to create Fjall for namespace '{}'", name))?;
        let vector_db = UsearchWrapper::new(dimensions, metric, scalar)
            .context(format!("Failed to create UsearchWrapper for namespace '{}'", name))?;

        // Persist metadata
        let metadata = NamespaceMetadata {
            name: name.to_string(),
            dimensions,
            metric: Self::metric_to_string(metric).to_string(),
            scalar: Self::scalar_to_string(scalar).to_string(),
        };
        self.persist_metadata(name, &metadata)?;

        namespaces.insert(name.to_string(), Namespace::new(db, vector_db));
        tracing::info!("Created namespace '{}' with {} dimensions", name, dimensions);
        Ok(())
    }

    pub fn get_namespace(&self, name: &str) -> Result<Namespace> {
        let namespaces = self.namespaces.read().unwrap();
        namespaces.get(name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Namespace '{}' not found", name))
    }

    pub fn delete_namespace(&self, name: &str) -> Result<()> {
        let mut namespaces = self.namespaces.write().unwrap();
        namespaces.remove(name)
            .ok_or_else(|| anyhow::anyhow!("Namespace '{}' not found", name))?;

        // Delete metadata
        self.delete_metadata(name)?;

        // Delete namespace directory
        let ns_dir = self.data_dir.join(name);
        if ns_dir.exists() {
            std::fs::remove_dir_all(&ns_dir)
                .context(format!("Failed to delete namespace directory '{}'", name))?;
        }

        tracing::info!("Deleted namespace '{}'", name);
        Ok(())
    }

    pub fn list_namespaces(&self) -> Vec<String> {
        let namespaces = self.namespaces.read().unwrap();
        namespaces.keys().cloned().collect()
    }

    pub fn namespace_exists(&self, name: &str) -> bool {
        let namespaces = self.namespaces.read().unwrap();
        namespaces.contains_key(name)
    }

    /// Save all vector indices to disk
    pub fn save_all(&self) -> Result<()> {
        let namespaces = self.namespaces.read().unwrap();
        for (name, ns) in namespaces.iter() {
            let vector_path = self.data_dir.join(name).join("vectors.idx");
            ns.vector_db.save(vector_path.to_str().unwrap())
                .context(format!("Failed to save vector index for namespace '{}'", name))?;
        }
        self.metadata_db.flush()?;
        tracing::info!("Saved all namespace data to disk");
        Ok(())
    }

    /// Save a specific namespace's vector index
    pub fn save_namespace(&self, name: &str) -> Result<()> {
        let namespaces = self.namespaces.read().unwrap();
        if let Some(ns) = namespaces.get(name) {
            let vector_path = self.data_dir.join(name).join("vectors.idx");
            ns.vector_db.save(vector_path.to_str().unwrap())
                .context(format!("Failed to save vector index for namespace '{}'", name))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_namespace_manager() {
        let temp_dir = TempDir::new().unwrap();
        let manager = NamespaceManager::new(temp_dir.path().to_path_buf()).unwrap();

        // Create a namespace
        assert!(manager.create_namespace("test1", 128, MetricKind::Cos, ScalarKind::F32).is_ok());

        // Check if namespace exists
        assert!(manager.namespace_exists("test1"));
        assert!(!manager.namespace_exists("nonexistent"));

        // Try to create a duplicate namespace
        assert!(manager.create_namespace("test1", 128, MetricKind::Cos, ScalarKind::F32).is_err());

        // Get a namespace
        let namespace = manager.get_namespace("test1");
        assert!(namespace.is_ok());

        // List namespaces
        let namespaces = manager.list_namespaces();
        assert_eq!(namespaces, vec!["test1"]);

        // Delete a namespace
        assert!(manager.delete_namespace("test1").is_ok());

        // Try to get a deleted namespace
        assert!(manager.get_namespace("test1").is_err());

        // Try to delete a non-existent namespace
        assert!(manager.delete_namespace("nonexistent").is_err());
    }

    #[test]
    fn test_namespace_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let data_path = temp_dir.path().to_path_buf();

        // Create manager and add a namespace
        {
            let manager = NamespaceManager::new(data_path.clone()).unwrap();
            manager.create_namespace("persistent", 256, MetricKind::Cos, ScalarKind::F32).unwrap();
            manager.save_all().unwrap();
        }

        // Create new manager and verify namespace was loaded
        {
            let manager = NamespaceManager::new(data_path).unwrap();
            assert!(manager.namespace_exists("persistent"));
            let ns = manager.get_namespace("persistent").unwrap();
            assert_eq!(ns.vector_db.dimensions(), 256);
        }
    }
}