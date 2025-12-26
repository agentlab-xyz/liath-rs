//! Long-term semantic memory for agents

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::{Result, Context};
use crate::EmbeddedLiath;
use super::types::{MemoryId, MemoryEntry, MemoryMetadata};

/// Long-term semantic memory storage for an agent
///
/// Memory provides semantic storage and retrieval of information that persists
/// across conversations. It uses vector embeddings for similarity-based recall.
pub struct Memory {
    agent_id: String,
    namespace: String,
    db: Arc<EmbeddedLiath>,
    next_id: std::sync::atomic::AtomicU64,
}

impl Memory {
    /// Create a new Memory instance for an agent
    pub fn new(agent_id: &str, db: Arc<EmbeddedLiath>) -> Result<Self> {
        let namespace = format!("agent_{}_memory", agent_id);

        // Create namespace if it doesn't exist
        #[cfg(feature = "vector")]
        if !db.namespace_exists(&namespace) {
            db.create_namespace(&namespace, 384, usearch::MetricKind::Cos, usearch::ScalarKind::F32)?;
        }

        // Load the next ID from metadata
        let next_id = Self::load_next_id(&db, &namespace)?;

        Ok(Self {
            agent_id: agent_id.to_string(),
            namespace,
            db,
            next_id: std::sync::atomic::AtomicU64::new(next_id),
        })
    }

    fn load_next_id(db: &EmbeddedLiath, namespace: &str) -> Result<u64> {
        if let Ok(Some(data)) = db.get(namespace, b"_next_id") {
            let id = u64::from_le_bytes(data.try_into().unwrap_or([0u8; 8]));
            Ok(id)
        } else {
            Ok(1)
        }
    }

    fn save_next_id(&self) -> Result<()> {
        let id = self.next_id.load(std::sync::atomic::Ordering::SeqCst);
        self.db.put(&self.namespace, b"_next_id", &id.to_le_bytes())?;
        Ok(())
    }

    fn get_next_id(&self) -> u64 {
        self.next_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Store content in memory with optional tags
    /// Returns the ID of the stored memory
    pub fn store(&self, content: &str, tags: &[&str]) -> Result<MemoryId> {
        let id = self.get_next_id();
        let timestamp = Self::current_timestamp();

        // Store the content
        let content_key = format!("content:{}", id);
        self.db.put(&self.namespace, content_key.as_bytes(), content.as_bytes())?;

        // Store metadata
        let metadata = MemoryMetadata {
            id,
            tags: tags.iter().map(|s| s.to_string()).collect(),
            created_at: timestamp,
        };
        let metadata_key = format!("meta:{}", id);
        let metadata_bytes = serde_json::to_vec(&metadata)
            .context("Failed to serialize memory metadata")?;
        self.db.put(&self.namespace, metadata_key.as_bytes(), &metadata_bytes)?;

        // Store tags index
        for tag in tags {
            let tag_key = format!("tag:{}:{}", tag, id);
            self.db.put(&self.namespace, tag_key.as_bytes(), &id.to_le_bytes())?;
        }

        // Generate and store embedding
        let embedding = self.db.generate_embedding(content)?;
        self.db.add_vector(&self.namespace, id, &embedding)?;

        // Save the next ID
        self.save_next_id()?;

        Ok(id)
    }

    /// Recall memories similar to the query
    pub fn recall(&self, query: &str, k: usize) -> Result<Vec<MemoryEntry>> {
        let results = self.db.search_vectors(
            &self.namespace,
            &self.db.generate_embedding(query)?,
            k,
        )?;

        let mut entries = Vec::with_capacity(results.len());
        for (id, distance) in results {
            if let Some(entry) = self.get_memory_entry(id, distance)? {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    /// Get a specific memory by ID
    fn get_memory_entry(&self, id: MemoryId, distance: f32) -> Result<Option<MemoryEntry>> {
        // Get content
        let content_key = format!("content:{}", id);
        let content = match self.db.get(&self.namespace, content_key.as_bytes())? {
            Some(data) => String::from_utf8_lossy(&data).to_string(),
            None => return Ok(None),
        };

        // Get metadata
        let metadata_key = format!("meta:{}", id);
        let metadata: MemoryMetadata = match self.db.get(&self.namespace, metadata_key.as_bytes())? {
            Some(data) => serde_json::from_slice(&data)
                .context("Failed to deserialize memory metadata")?,
            None => return Ok(None),
        };

        Ok(Some(MemoryEntry {
            id,
            content,
            tags: metadata.tags,
            distance,
            created_at: metadata.created_at,
        }))
    }

    /// Recall memories by specific tags
    /// Finds memories that have ALL specified tags (intersection)
    pub fn recall_by_tags(&self, tags: &[&str], k: usize) -> Result<Vec<MemoryEntry>> {
        use std::collections::HashSet;

        if tags.is_empty() {
            return Ok(Vec::new());
        }

        // For each tag, collect all memory IDs that have that tag
        let mut tag_id_sets: Vec<HashSet<MemoryId>> = Vec::new();

        for tag in tags {
            let mut ids = HashSet::new();

            // Scan all memory IDs to find those with this tag
            let next_id = self.next_id.load(std::sync::atomic::Ordering::SeqCst);
            for id in 1..next_id {
                let tag_key = format!("tag:{}:{}", tag, id);
                if let Ok(Some(_)) = self.db.get(&self.namespace, tag_key.as_bytes()) {
                    ids.insert(id);
                }
            }

            if ids.is_empty() {
                // If any tag has no matches, intersection will be empty
                return Ok(Vec::new());
            }
            tag_id_sets.push(ids);
        }

        // Find intersection of all tag sets
        let mut matching_ids: HashSet<MemoryId> = tag_id_sets.remove(0);
        for id_set in tag_id_sets {
            matching_ids = matching_ids.intersection(&id_set).cloned().collect();
        }

        // Retrieve memory entries for matching IDs (limited to k)
        let mut entries = Vec::new();
        for id in matching_ids.into_iter().take(k) {
            // Use distance 0.0 for tag-based recall (not similarity-based)
            if let Some(entry) = self.get_memory_entry(id, 0.0)? {
                entries.push(entry);
            }
        }

        // Sort by created_at (most recent first)
        entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(entries)
    }

    /// Delete a memory by ID
    pub fn forget(&self, id: MemoryId) -> Result<()> {
        // Delete content
        let content_key = format!("content:{}", id);
        self.db.delete(&self.namespace, content_key.as_bytes())?;

        // Delete metadata
        let metadata_key = format!("meta:{}", id);

        // Get metadata first to delete tag indices
        if let Some(data) = self.db.get(&self.namespace, metadata_key.as_bytes())? {
            if let Ok(metadata) = serde_json::from_slice::<MemoryMetadata>(&data) {
                for tag in &metadata.tags {
                    let tag_key = format!("tag:{}:{}", tag, id);
                    let _ = self.db.delete(&self.namespace, tag_key.as_bytes());
                }
            }
        }

        self.db.delete(&self.namespace, metadata_key.as_bytes())?;

        // Note: Vector index doesn't support deletion in usearch without rebuild
        // This is a known limitation

        Ok(())
    }

    /// Get the agent ID this memory belongs to
    pub fn agent_id(&self) -> &str {
        &self.agent_id
    }

    /// Get the namespace used for storage
    pub fn namespace(&self) -> &str {
        &self.namespace
    }
}
