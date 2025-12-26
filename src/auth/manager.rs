use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;
use anyhow::{Result, anyhow, Context};
use serde::{Serialize, Deserialize};
use crate::core::FjallWrapper;

/// Persisted user permissions
#[derive(Serialize, Deserialize, Debug, Clone)]
struct UserPermissions {
    user_id: String,
    permissions: Vec<String>,
}

pub struct AuthManager {
    user_permissions: HashMap<String, HashSet<String>>,
    store: Option<Arc<FjallWrapper>>,
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthManager {
    /// Create a new in-memory only AuthManager
    pub fn new() -> Self {
        Self {
            user_permissions: HashMap::new(),
            store: None,
        }
    }

    /// Create a new AuthManager with persistence
    pub fn with_persistence(data_dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(data_dir)
            .context("Failed to create auth data directory")?;

        let store = FjallWrapper::new(data_dir.join("_auth"))
            .context("Failed to create auth database")?;

        let mut manager = Self {
            user_permissions: HashMap::new(),
            store: Some(Arc::new(store)),
        };

        manager.load_all()?;
        Ok(manager)
    }

    /// Load all users from persistent storage
    fn load_all(&mut self) -> Result<()> {
        if let Some(ref store) = self.store {
            for result in store.iter() {
                let (key, value) = result?;
                let user_id = String::from_utf8(key)
                    .context("Invalid user ID in auth store")?;

                let user_perms: UserPermissions = serde_json::from_slice(&value)
                    .context(format!("Failed to deserialize permissions for user '{}'", user_id))?;

                self.user_permissions.insert(
                    user_id.clone(),
                    user_perms.permissions.into_iter().collect(),
                );
                tracing::debug!("Loaded auth for user '{}'", user_id);
            }
        }
        Ok(())
    }

    /// Persist user permissions to disk
    fn persist_user(&self, user_id: &str) -> Result<()> {
        if let Some(ref store) = self.store {
            if let Some(perms) = self.user_permissions.get(user_id) {
                let user_perms = UserPermissions {
                    user_id: user_id.to_string(),
                    permissions: perms.iter().cloned().collect(),
                };
                let value = serde_json::to_vec(&user_perms)
                    .context("Failed to serialize user permissions")?;
                store.put(user_id.as_bytes(), &value)
                    .context("Failed to persist user permissions")?;
            }
        }
        Ok(())
    }

    /// Delete user from persistent storage
    fn delete_user_from_store(&self, user_id: &str) -> Result<()> {
        if let Some(ref store) = self.store {
            store.delete(user_id.as_bytes())
                .context("Failed to delete user from store")?;
        }
        Ok(())
    }

    pub fn add_user(&mut self, user_id: &str, permissions: Vec<String>) {
        self.user_permissions.insert(user_id.to_string(), permissions.into_iter().collect());
        if let Err(e) = self.persist_user(user_id) {
            tracing::warn!("Failed to persist user '{}': {}", user_id, e);
        }
    }

    pub fn is_authorized(&self, user_id: &str, permission: &str) -> bool {
        self.user_permissions
            .get(user_id)
            .map(|permissions| permissions.contains(permission))
            .unwrap_or(false)
    }

    pub fn remove_user(&mut self, user_id: &str) -> Result<()> {
        self.user_permissions.remove(user_id)
            .ok_or_else(|| anyhow!("User not found"))?;
        self.delete_user_from_store(user_id)?;
        Ok(())
    }

    pub fn update_permissions(&mut self, user_id: &str, permissions: Vec<String>) -> Result<()> {
        self.user_permissions.get_mut(user_id)
            .ok_or_else(|| anyhow!("User not found"))?
            .clear();
        self.user_permissions.get_mut(user_id).unwrap().extend(permissions);
        self.persist_user(user_id)?;
        Ok(())
    }

    pub fn add_permission(&mut self, user_id: &str, permission: String) -> Result<()> {
        self.user_permissions.get_mut(user_id)
            .ok_or_else(|| anyhow!("User not found"))?
            .insert(permission);
        self.persist_user(user_id)?;
        Ok(())
    }

    pub fn remove_permission(&mut self, user_id: &str, permission: &str) -> Result<()> {
        self.user_permissions.get_mut(user_id)
            .ok_or_else(|| anyhow!("User not found"))?
            .remove(permission);
        self.persist_user(user_id)?;
        Ok(())
    }

    /// Flush auth data to disk
    pub fn flush(&self) -> Result<()> {
        if let Some(ref store) = self.store {
            store.flush()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_auth_manager() {
        let mut auth_manager = AuthManager::new();

        auth_manager.add_user("user1", vec!["select".to_string(), "insert".to_string()]);
        assert!(auth_manager.is_authorized("user1", "select"));
        assert!(auth_manager.is_authorized("user1", "insert"));
        assert!(!auth_manager.is_authorized("user1", "delete"));

        auth_manager.update_permissions("user1", vec!["select".to_string(), "delete".to_string()]).unwrap();
        assert!(auth_manager.is_authorized("user1", "select"));
        assert!(!auth_manager.is_authorized("user1", "insert"));
        assert!(auth_manager.is_authorized("user1", "delete"));

        auth_manager.add_permission("user1", "update".to_string()).unwrap();
        assert!(auth_manager.is_authorized("user1", "update"));

        auth_manager.remove_permission("user1", "delete").unwrap();
        assert!(!auth_manager.is_authorized("user1", "delete"));

        auth_manager.remove_user("user1").unwrap();
        assert!(!auth_manager.is_authorized("user1", "select"));
    }

    #[test]
    fn test_auth_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let data_path = temp_dir.path();

        // Create manager with persistence and add a user
        {
            let mut manager = AuthManager::with_persistence(data_path).unwrap();
            manager.add_user("persistent_user", vec![
                "select".to_string(),
                "insert".to_string(),
                "delete".to_string(),
            ]);
            manager.flush().unwrap();
        }

        // Create new manager and verify user was loaded
        {
            let manager = AuthManager::with_persistence(data_path).unwrap();
            assert!(manager.is_authorized("persistent_user", "select"));
            assert!(manager.is_authorized("persistent_user", "insert"));
            assert!(manager.is_authorized("persistent_user", "delete"));
            assert!(!manager.is_authorized("persistent_user", "admin"));
        }
    }
}