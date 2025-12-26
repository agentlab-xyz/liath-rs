use crate::core::NamespaceManager;
use crate::ai::EmbeddingWrapper;
use crate::lua::LuaVM;
use crate::file::FileStorage;
use crate::auth::AuthManager;
use anyhow::Result;
use tokio::sync::Semaphore;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::instrument;
use rlua::{Context as LuaContext, Error as LuaError, Value as LuaValue, Table as LuaTable};
#[cfg(feature = "vector")]
use usearch::{MetricKind, ScalarKind};
#[cfg(not(feature = "vector"))]
use crate::core::{MetricKind, ScalarKind};

#[derive(Clone)]
pub struct QueryExecutor {
    namespace_manager: Arc<RwLock<NamespaceManager>>,
    embedding: Arc<RwLock<EmbeddingWrapper>>,
    lua_vm: Arc<RwLock<LuaVM>>,
    file_storage: Arc<RwLock<FileStorage>>,
    auth_manager: Arc<RwLock<AuthManager>>,
    embedding_semaphore: Arc<Semaphore>,
}

impl QueryExecutor {
    pub fn new(
        namespace_manager: NamespaceManager,
        embedding: EmbeddingWrapper,
        lua_vm: LuaVM,
        file_storage: FileStorage,
        auth_manager: AuthManager,
        max_concurrent_embedding: usize,
    ) -> Self {
        Self {
            namespace_manager: Arc::new(RwLock::new(namespace_manager)),
            embedding: Arc::new(RwLock::new(embedding)),
            lua_vm: Arc::new(RwLock::new(lua_vm)),
            file_storage: Arc::new(RwLock::new(file_storage)),
            auth_manager: Arc::new(RwLock::new(auth_manager)),
            embedding_semaphore: Arc::new(Semaphore::new(max_concurrent_embedding)),
        }
    }

    #[instrument(skip(self, query))]
    pub async fn execute(&self, query: &str, user_id: &str) -> Result<String> {
        let res: String = self
            .lua_vm
            .read()
            .unwrap()
            .execute_with_context(|lua_ctx| {
                self.register_db_functions(&lua_ctx, user_id)
                    .map_err(|e| LuaError::RuntimeError(e.to_string()))?;
                let value: LuaValue = lua_ctx.load(query).eval()?;
                let out = match value {
                    LuaValue::String(s) => s.to_str()?.to_owned(),
                    LuaValue::Number(n) => n.to_string(),
                    LuaValue::Integer(i) => i.to_string(),
                    LuaValue::Boolean(b) => b.to_string(),
                    LuaValue::Nil => "nil".to_owned(),
                    _ => return Err(LuaError::RuntimeError("Unexpected Lua return type".to_string())),
                };
                Ok(out)
            })?;
        Ok(res)
    }

    // Public, typed helpers (Rust API)
    pub fn create_namespace(
        &self,
        name: &str,
        dimensions: usize,
        metric: MetricKind,
        scalar: ScalarKind,
    ) -> Result<()> {
        self.namespace_manager
            .read()
            .unwrap()
            .create_namespace(name, dimensions, metric, scalar)
    }
    #[cfg(not(feature = "vector"))]
    pub fn create_namespace_basic(&self, name: &str) -> anyhow::Result<()> {
        self.create_namespace(name, 128, MetricKind::Cos, ScalarKind::F32)
    }


    pub fn put(&self, namespace: &str, key: &[u8], value: &[u8]) -> Result<()> {
        let ns = self
            .namespace_manager
            .read()
            .unwrap()
            .get_namespace(namespace)?;
        ns.db.put(key, value)
    }

    pub fn get(&self, namespace: &str, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let ns = self
            .namespace_manager
            .read()
            .unwrap()
            .get_namespace(namespace)?;
        ns.db.get(key)
    }

    pub fn delete(&self, namespace: &str, key: &[u8]) -> Result<()> {
        let ns = self
            .namespace_manager
            .read()
            .unwrap()
            .get_namespace(namespace)?;
        ns.db.delete(key)
    }

    pub fn list_namespaces(&self) -> Vec<String> {
        self.namespace_manager.read().unwrap().list_namespaces()
    }

    pub fn generate_embedding(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>> {
        self.embedding.read().unwrap().generate(texts)
    }

    pub fn similarity_search(
        &self,
        namespace: &str,
        vector: &[f32],
        k: usize,
    ) -> Result<Vec<(u64, f32)>> {
        let ns = self
            .namespace_manager
            .read()
            .unwrap()
            .get_namespace(namespace)?;
        ns.vector_db.search(vector, k)
    }

    /// Add a vector to a namespace
    pub fn add_vector(&self, namespace: &str, id: u64, vector: &[f32]) -> Result<()> {
        let ns = self
            .namespace_manager
            .read()
            .unwrap()
            .get_namespace(namespace)?;
        ns.vector_db.add(id, vector)
    }

    /// Check if a namespace exists
    pub fn namespace_exists(&self, name: &str) -> bool {
        self.namespace_manager.read().unwrap().namespace_exists(name)
    }

    /// Delete a namespace
    pub fn delete_namespace(&self, name: &str) -> Result<()> {
        self.namespace_manager.write().unwrap().delete_namespace(name)
    }

    /// Save all data to disk
    pub fn save_all(&self) -> Result<()> {
        self.namespace_manager.read().unwrap().save_all()?;
        self.auth_manager.read().unwrap().flush()?;
        Ok(())
    }

    /// Save a specific namespace
    pub fn save_namespace(&self, name: &str) -> Result<()> {
        self.namespace_manager.read().unwrap().save_namespace(name)
    }

    fn register_db_functions(&self, lua_ctx: &LuaContext, user_id: &str) -> Result<(), LuaError> {
        // These are cloned as needed in closures below
        let namespace_manager = self.namespace_manager.clone();
        let auth_manager = self.auth_manager.clone();

        let user_id_str = user_id.to_string();

        // Namespace operations
        let user_id = user_id_str.clone();
        lua_ctx.globals().set("create_namespace", lua_ctx.create_function_mut(move |_, (name, dimensions, metric, scalar): (String, usize, String, String)| {
            if !auth_manager.read().unwrap().is_authorized(&user_id, "create_namespace") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            let metric = match metric.as_str() {
                "cosine" => MetricKind::Cos,
                "euclidean" => MetricKind::L2sq,
                _ => return Err(LuaError::RuntimeError("Invalid metric kind".to_string())),
            };
            let scalar = match scalar.as_str() {
                "f32" => ScalarKind::F32,
                "f16" => ScalarKind::F16,
                _ => return Err(LuaError::RuntimeError("Invalid scalar kind".to_string())),
            };
            namespace_manager.write().unwrap().create_namespace(&name, dimensions, metric, scalar)
                .map_err(|e| LuaError::RuntimeError(format!("Failed to create namespace: {}", e)))
        })?)?;

        let user_id = user_id_str.clone();
        let namespace_manager = self.namespace_manager.clone();
        let auth_manager = self.auth_manager.clone();
        lua_ctx.globals().set("delete_namespace", lua_ctx.create_function_mut(move |_, name: String| {
            if !auth_manager.read().unwrap().is_authorized(&user_id, "delete_namespace") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            namespace_manager.write().unwrap().delete_namespace(&name)
                .map_err(|e| LuaError::RuntimeError(format!("Failed to delete namespace: {}", e)))
        })?)?;

        let user_id = user_id_str.clone();
        let namespace_manager = self.namespace_manager.clone();
        let auth_manager = self.auth_manager.clone();
        lua_ctx.globals().set("list_namespaces", lua_ctx.create_function_mut(move |lua_ctx, ()| {
            if !auth_manager.read().unwrap().is_authorized(&user_id, "list_namespaces") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            let namespaces = namespace_manager.read().unwrap().list_namespaces();
            let lua_namespaces = lua_ctx.create_table()?;
            for (i, namespace) in namespaces.iter().enumerate() {
                lua_namespaces.set(i + 1, namespace.clone())?;
            }
            Ok(lua_namespaces)
        })?)?;

        // Database operations
        let user_id = user_id_str.clone();
        let namespace_manager = self.namespace_manager.clone();
        let auth_manager = self.auth_manager.clone();
        lua_ctx.globals().set("select", lua_ctx.create_function_mut(move |_, (namespace, key): (String, String)| {
            if !auth_manager.read().unwrap().is_authorized(&user_id, "select") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            let ns = namespace_manager.read().unwrap().get_namespace(&namespace)
                .map_err(|e| LuaError::RuntimeError(format!("Namespace error: {}", e)))?;
            let value = ns.db.get(key.as_bytes())
                .map_err(|e| LuaError::RuntimeError(format!("Failed to retrieve value: {}", e)))?;
            Ok(value.map(|v| String::from_utf8_lossy(&v).into_owned()))
        })?)?;

        let user_id = user_id_str.clone();
        let namespace_manager = self.namespace_manager.clone();
        let auth_manager = self.auth_manager.clone();
        lua_ctx.globals().set("insert", lua_ctx.create_function_mut(move |_, (namespace, key, value): (String, String, String)| {
            if !auth_manager.read().unwrap().is_authorized(&user_id, "insert") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            let ns = namespace_manager.read().unwrap().get_namespace(&namespace)
                .map_err(|e| LuaError::RuntimeError(format!("Namespace error: {}", e)))?;
            ns.db.put(key.as_bytes(), value.as_bytes())
                .map_err(|e| LuaError::RuntimeError(format!("Failed to insert value: {}", e)))?;
            Ok(())
        })?)?;

        let user_id = user_id_str.clone();
        let namespace_manager = self.namespace_manager.clone();
        let auth_manager = self.auth_manager.clone();
        lua_ctx.globals().set("update", lua_ctx.create_function_mut(move |_, (namespace, key, value): (String, String, String)| {
            if !auth_manager.read().unwrap().is_authorized(&user_id, "update") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            let ns = namespace_manager.read().unwrap().get_namespace(&namespace)
                .map_err(|e| LuaError::RuntimeError(format!("Namespace error: {}", e)))?;
            ns.db.put(key.as_bytes(), value.as_bytes())
                .map_err(|e| LuaError::RuntimeError(format!("Failed to update value: {}", e)))?;
            Ok(())
        })?)?;

        let user_id = user_id_str.clone();
        let namespace_manager = self.namespace_manager.clone();
        let auth_manager = self.auth_manager.clone();
        lua_ctx.globals().set("delete", lua_ctx.create_function_mut(move |_, (namespace, key): (String, String)| {
            if !auth_manager.read().unwrap().is_authorized(&user_id, "delete") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            let ns = namespace_manager.read().unwrap().get_namespace(&namespace)
                .map_err(|e| LuaError::RuntimeError(format!("Namespace error: {}", e)))?;
            ns.db.delete(key.as_bytes())
                .map_err(|e| LuaError::RuntimeError(format!("Failed to delete value: {}", e)))?;
            Ok(())
        })?)?;

        // Embedding operations
        let user_id = user_id_str.clone();
        let embedding = self.embedding.clone();
        let auth_manager = self.auth_manager.clone();
        let embedding_semaphore = self.embedding_semaphore.clone();
        lua_ctx.globals().set("generate_embedding", lua_ctx.create_function_mut(move |lua_ctx, texts: Vec<String>| {
            if !auth_manager.read().unwrap().is_authorized(&user_id, "generate_embedding") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            let _permit = embedding_semaphore.try_acquire()
                .map_err(|_| LuaError::RuntimeError("Failed to acquire embedding semaphore".to_string()))?;
            
            let embedding_results = embedding.read().unwrap().generate(texts.iter().map(|s| s.as_str()).collect())
                .map_err(|e| LuaError::RuntimeError(format!("Failed to generate embeddings: {}", e)))?;
            
            let lua_embeddings = lua_ctx.create_table()?;
            for (i, embedding) in embedding_results.iter().enumerate() {
                let lua_embedding = lua_ctx.create_table()?;
                for (j, value) in embedding.iter().enumerate() {
                    lua_embedding.set(j + 1, *value)?;
                }
                lua_embeddings.set(i + 1, lua_embedding)?;
            }
            Ok(lua_embeddings)
        })?)?;

        // File operations
        let user_id = user_id_str.clone();
        let file_storage = self.file_storage.clone();
        let auth_manager = self.auth_manager.clone();
        lua_ctx.globals().set("upload_file", lua_ctx.create_function_mut(move |_, (_file_name, content): (String, Vec<u8>)| {
            if !auth_manager.read().unwrap().is_authorized(&user_id, "upload_file") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            let file_id = file_storage.read().unwrap().store(&content)
                .map_err(|e| LuaError::RuntimeError(format!("Failed to store file: {}", e)))?;
            Ok(file_id)
        })?)?;

        let user_id = user_id_str.clone();
        let file_storage = self.file_storage.clone();
        let auth_manager = self.auth_manager.clone();
        lua_ctx.globals().set("retrieve_file", lua_ctx.create_function_mut(move |lua_ctx, file_id: String| {
            if !auth_manager.read().unwrap().is_authorized(&user_id, "retrieve_file") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            let content = file_storage.read().unwrap().retrieve(&file_id)
                .map_err(|e| LuaError::RuntimeError(format!("Failed to retrieve file: {}", e)))?;
            let lua_content = lua_ctx.create_string(&content)?;
            Ok(lua_content)
        })?)?;

        // Vector search operations
        let user_id = user_id_str.clone();
        let namespace_manager = self.namespace_manager.clone();
        let auth_manager = self.auth_manager.clone();
        lua_ctx.globals().set("similarity_search", lua_ctx.create_function_mut(move |lua_ctx, (namespace, vector, k): (String, Vec<f32>, usize)| {
            if !auth_manager.read().unwrap().is_authorized(&user_id, "similarity_search") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            let ns = namespace_manager.read().unwrap().get_namespace(&namespace)
                .map_err(|e| LuaError::RuntimeError(format!("Namespace error: {}", e)))?;
            let results = ns.vector_db.search(&vector, k)
                .map_err(|e| LuaError::RuntimeError(format!("Failed to perform similarity search: {}", e)))?;
            
            let lua_results = lua_ctx.create_table()?;
            for (i, (id, distance)) in results.into_iter().enumerate() {
                let result_table = lua_ctx.create_table()?;
                result_table.set("id", id)?;
                result_table.set("distance", distance)?;
                lua_results.set(i + 1, result_table)?;
            }
            Ok(lua_results)
        })?)?;

        // LuaRocks package management
        let user_id = user_id_str.clone();
        let lua_vm = self.lua_vm.clone();
        let auth_manager = self.auth_manager.clone();
        lua_ctx.globals().set("install_package", lua_ctx.create_function_mut(move |_, package_name: String| {
            if !auth_manager.read().unwrap().is_authorized(&user_id, "install_package") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            lua_vm.read().unwrap().install_package(&package_name)
                .map_err(|e| LuaError::RuntimeError(format!("Failed to install package: {}", e)))?;
            Ok(())
        })?)?;

        let user_id = user_id_str.clone();
        let lua_vm = self.lua_vm.clone();
        let auth_manager = self.auth_manager.clone();
        lua_ctx.globals().set("list_packages", lua_ctx.create_function_mut(move |lua_ctx, ()| {
            if !auth_manager.read().unwrap().is_authorized(&user_id, "list_packages") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            let packages = lua_vm.read().unwrap().list_installed_packages()
                .map_err(|e| LuaError::RuntimeError(format!("Failed to list packages: {}", e)))?;
            let lua_packages = lua_ctx.create_table()?;
            for (i, package) in packages.iter().enumerate() {
                lua_packages.set(i + 1, package.clone())?;
            }
            Ok(lua_packages)
        })?)?;

        // ============================================================
        // VECTOR OPERATIONS
        // ============================================================

        // add_vector(namespace, id, vector) - Add a vector to the index
        let user_id = user_id_str.clone();
        let namespace_manager = self.namespace_manager.clone();
        let auth_manager = self.auth_manager.clone();
        lua_ctx.globals().set("add_vector", lua_ctx.create_function_mut(move |_, (namespace, id, vector): (String, u64, Vec<f32>)| {
            if !auth_manager.read().unwrap().is_authorized(&user_id, "insert") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            let ns = namespace_manager.read().unwrap().get_namespace(&namespace)
                .map_err(|e| LuaError::RuntimeError(format!("Namespace error: {}", e)))?;
            ns.vector_db.add(id, &vector)
                .map_err(|e| LuaError::RuntimeError(format!("Failed to add vector: {}", e)))?;
            Ok(())
        })?)?;

        // store_document(namespace, id, key, text) - Store text with auto-embedding
        let user_id = user_id_str.clone();
        let namespace_manager = self.namespace_manager.clone();
        let embedding = self.embedding.clone();
        let auth_manager = self.auth_manager.clone();
        lua_ctx.globals().set("store_document", lua_ctx.create_function_mut(move |_, (namespace, id, key, text): (String, u64, String, String)| {
            if !auth_manager.read().unwrap().is_authorized(&user_id, "insert") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            let ns = namespace_manager.read().unwrap().get_namespace(&namespace)
                .map_err(|e| LuaError::RuntimeError(format!("Namespace error: {}", e)))?;

            // Generate embedding
            let embeddings = embedding.read().unwrap().generate(vec![text.as_str()])
                .map_err(|e| LuaError::RuntimeError(format!("Embedding error: {}", e)))?;
            let vector = embeddings.into_iter().next()
                .ok_or_else(|| LuaError::RuntimeError("Failed to generate embedding".to_string()))?;

            // Store text
            ns.db.put(key.as_bytes(), text.as_bytes())
                .map_err(|e| LuaError::RuntimeError(format!("Failed to store text: {}", e)))?;

            // Store vector
            ns.vector_db.add(id, &vector)
                .map_err(|e| LuaError::RuntimeError(format!("Failed to add vector: {}", e)))?;

            // Store ID -> key mapping for semantic search lookup
            let mapping_key = format!("_vidx:{}", id);
            ns.db.put(mapping_key.as_bytes(), key.as_bytes())
                .map_err(|e| LuaError::RuntimeError(format!("Failed to store mapping: {}", e)))?;

            Ok(id)
        })?)?;

        // semantic_search(namespace, query_text, k) - Search by text query
        let user_id = user_id_str.clone();
        let namespace_manager = self.namespace_manager.clone();
        let embedding = self.embedding.clone();
        let auth_manager = self.auth_manager.clone();
        lua_ctx.globals().set("semantic_search", lua_ctx.create_function_mut(move |lua_ctx, (namespace, query, k): (String, String, usize)| {
            if !auth_manager.read().unwrap().is_authorized(&user_id, "similarity_search") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            let ns = namespace_manager.read().unwrap().get_namespace(&namespace)
                .map_err(|e| LuaError::RuntimeError(format!("Namespace error: {}", e)))?;

            // Generate query embedding
            let embeddings = embedding.read().unwrap().generate(vec![query.as_str()])
                .map_err(|e| LuaError::RuntimeError(format!("Embedding error: {}", e)))?;
            let query_vector = embeddings.into_iter().next()
                .ok_or_else(|| LuaError::RuntimeError("Failed to generate embedding".to_string()))?;

            // Search
            let results = ns.vector_db.search(&query_vector, k)
                .map_err(|e| LuaError::RuntimeError(format!("Search error: {}", e)))?;

            let lua_results = lua_ctx.create_table()?;
            for (i, (id, distance)) in results.into_iter().enumerate() {
                let result_table = lua_ctx.create_table()?;
                result_table.set("id", id)?;
                result_table.set("distance", distance)?;

                // Look up content using ID -> key mapping
                let mapping_key = format!("_vidx:{}", id);
                if let Ok(Some(key)) = ns.db.get(mapping_key.as_bytes()) {
                    if let Ok(Some(content)) = ns.db.get(&key) {
                        result_table.set("content", String::from_utf8_lossy(&content).into_owned())?;
                        result_table.set("key", String::from_utf8_lossy(&key).into_owned())?;
                    }
                }

                lua_results.set(i + 1, result_table)?;
            }
            Ok(lua_results)
        })?)?;

        // ============================================================
        // JSON OPERATIONS
        // ============================================================

        // json_encode(table) - Encode Lua table to JSON string
        lua_ctx.globals().set("json_encode", lua_ctx.create_function(|_, value: LuaValue| {
            let json = lua_value_to_json(value)?;
            serde_json::to_string(&json)
                .map_err(|e| LuaError::RuntimeError(format!("JSON encode error: {}", e)))
        })?)?;

        // json_decode(string) - Decode JSON string to Lua table
        lua_ctx.globals().set("json_decode", lua_ctx.create_function(|lua_ctx, json_str: String| {
            let value: serde_json::Value = serde_json::from_str(&json_str)
                .map_err(|e| LuaError::RuntimeError(format!("JSON decode error: {}", e)))?;
            json_to_lua_value(lua_ctx, &value)
        })?)?;

        // insert_json(namespace, key, table) - Store Lua table as JSON
        let user_id = user_id_str.clone();
        let namespace_manager = self.namespace_manager.clone();
        let auth_manager = self.auth_manager.clone();
        lua_ctx.globals().set("insert_json", lua_ctx.create_function_mut(move |_, (namespace, key, value): (String, String, LuaValue)| {
            if !auth_manager.read().unwrap().is_authorized(&user_id, "insert") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            let ns = namespace_manager.read().unwrap().get_namespace(&namespace)
                .map_err(|e| LuaError::RuntimeError(format!("Namespace error: {}", e)))?;

            let json = lua_value_to_json(value)?;
            let json_str = serde_json::to_string(&json)
                .map_err(|e| LuaError::RuntimeError(format!("JSON encode error: {}", e)))?;

            ns.db.put(key.as_bytes(), json_str.as_bytes())
                .map_err(|e| LuaError::RuntimeError(format!("Failed to insert: {}", e)))?;
            Ok(())
        })?)?;

        // select_json(namespace, key) - Retrieve as Lua table
        let user_id = user_id_str.clone();
        let namespace_manager = self.namespace_manager.clone();
        let auth_manager = self.auth_manager.clone();
        lua_ctx.globals().set("select_json", lua_ctx.create_function_mut(move |lua_ctx, (namespace, key): (String, String)| {
            if !auth_manager.read().unwrap().is_authorized(&user_id, "select") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            let ns = namespace_manager.read().unwrap().get_namespace(&namespace)
                .map_err(|e| LuaError::RuntimeError(format!("Namespace error: {}", e)))?;

            let data = ns.db.get(key.as_bytes())
                .map_err(|e| LuaError::RuntimeError(format!("Failed to select: {}", e)))?;

            match data {
                Some(bytes) => {
                    let json_str = String::from_utf8_lossy(&bytes);
                    let value: serde_json::Value = serde_json::from_str(&json_str)
                        .map_err(|e| LuaError::RuntimeError(format!("JSON decode error: {}", e)))?;
                    json_to_lua_value(lua_ctx, &value)
                }
                None => Ok(LuaValue::Nil),
            }
        })?)?;

        // ============================================================
        // UTILITY FUNCTIONS
        // ============================================================

        // save() - Persist all data to disk
        let namespace_manager = self.namespace_manager.clone();
        let auth_manager_save = self.auth_manager.clone();
        lua_ctx.globals().set("save", lua_ctx.create_function_mut(move |_, ()| {
            namespace_manager.read().unwrap().save_all()
                .map_err(|e| LuaError::RuntimeError(format!("Save error: {}", e)))?;
            auth_manager_save.read().unwrap().flush()
                .map_err(|e| LuaError::RuntimeError(format!("Auth save error: {}", e)))?;
            Ok(())
        })?)?;

        // namespace_exists(name) - Check if namespace exists
        let namespace_manager = self.namespace_manager.clone();
        lua_ctx.globals().set("namespace_exists", lua_ctx.create_function_mut(move |_, name: String| {
            Ok(namespace_manager.read().unwrap().namespace_exists(&name))
        })?)?;

        // uuid() - Generate a UUID
        lua_ctx.globals().set("uuid", lua_ctx.create_function(|_, ()| {
            Ok(uuid::Uuid::new_v4().to_string())
        })?)?;

        // timestamp() - Current Unix timestamp
        lua_ctx.globals().set("timestamp", lua_ctx.create_function(|_, ()| {
            let ts = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            Ok(ts)
        })?)?;

        // sleep(ms) - Sleep for milliseconds (useful for rate limiting)
        lua_ctx.globals().set("sleep", lua_ctx.create_function(|_, ms: u64| {
            std::thread::sleep(std::time::Duration::from_millis(ms));
            Ok(())
        })?)?;

        // ============================================================
        // BATCH OPERATIONS
        // ============================================================

        // batch_insert(namespace, items) - Batch insert key-value pairs
        // items = { {key="k1", value="v1"}, {key="k2", value="v2"}, ... }
        let user_id = user_id_str.clone();
        let namespace_manager = self.namespace_manager.clone();
        let auth_manager = self.auth_manager.clone();
        lua_ctx.globals().set("batch_insert", lua_ctx.create_function_mut(move |_, (namespace, items): (String, LuaTable)| {
            if !auth_manager.read().unwrap().is_authorized(&user_id, "insert") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            let ns = namespace_manager.read().unwrap().get_namespace(&namespace)
                .map_err(|e| LuaError::RuntimeError(format!("Namespace error: {}", e)))?;

            let mut batch_items: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
            for pair in items.sequence_values::<LuaTable>() {
                let item = pair?;
                let key: String = item.get("key")?;
                let value: String = item.get("value")?;
                batch_items.push((key.into_bytes(), value.into_bytes()));
            }

            let refs: Vec<(&[u8], &[u8])> = batch_items.iter()
                .map(|(k, v)| (k.as_slice(), v.as_slice()))
                .collect();

            ns.db.batch_put(refs)
                .map_err(|e| LuaError::RuntimeError(format!("Batch insert error: {}", e)))?;

            Ok(batch_items.len())
        })?)?;

        // batch_select(namespace, keys) - Batch get values
        let user_id = user_id_str.clone();
        let namespace_manager = self.namespace_manager.clone();
        let auth_manager = self.auth_manager.clone();
        lua_ctx.globals().set("batch_select", lua_ctx.create_function_mut(move |lua_ctx, (namespace, keys): (String, Vec<String>)| {
            if !auth_manager.read().unwrap().is_authorized(&user_id, "select") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            let ns = namespace_manager.read().unwrap().get_namespace(&namespace)
                .map_err(|e| LuaError::RuntimeError(format!("Namespace error: {}", e)))?;

            let results = lua_ctx.create_table()?;
            for key in keys {
                let value = ns.db.get(key.as_bytes())
                    .map_err(|e| LuaError::RuntimeError(format!("Get error: {}", e)))?;
                match value {
                    Some(v) => results.set(key, String::from_utf8_lossy(&v).into_owned())?,
                    None => results.set(key, LuaValue::Nil)?,
                }
            }
            Ok(results)
        })?)?;

        // scan(namespace, prefix, limit) - Scan keys with prefix
        let user_id = user_id_str.clone();
        let namespace_manager = self.namespace_manager.clone();
        let auth_manager = self.auth_manager.clone();
        lua_ctx.globals().set("scan", lua_ctx.create_function_mut(move |lua_ctx, (namespace, prefix, limit): (String, String, Option<usize>)| {
            if !auth_manager.read().unwrap().is_authorized(&user_id, "select") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            let ns = namespace_manager.read().unwrap().get_namespace(&namespace)
                .map_err(|e| LuaError::RuntimeError(format!("Namespace error: {}", e)))?;

            let limit = limit.unwrap_or(100);
            let results = lua_ctx.create_table()?;
            let mut count = 0;

            for result in ns.db.iter() {
                if count >= limit {
                    break;
                }
                let (key, value) = result
                    .map_err(|e| LuaError::RuntimeError(format!("Scan error: {}", e)))?;
                let key_str = String::from_utf8_lossy(&key);
                if key_str.starts_with(&prefix) {
                    let entry = lua_ctx.create_table()?;
                    entry.set("key", key_str.into_owned())?;
                    entry.set("value", String::from_utf8_lossy(&value).into_owned())?;
                    results.set(count + 1, entry)?;
                    count += 1;
                }
            }
            Ok(results)
        })?)?;

        // ============================================================
        // AGENT MEMORY OPERATIONS
        // ============================================================

        // memory_store(namespace, content, tags) - Store content with embedding
        let user_id = user_id_str.clone();
        let namespace_manager = self.namespace_manager.clone();
        let embedding = self.embedding.clone();
        let auth_manager = self.auth_manager.clone();
        lua_ctx.globals().set("memory_store", lua_ctx.create_function_mut(move |_, (namespace, content, tags): (String, String, Option<Vec<String>>)| {
            if !auth_manager.read().unwrap().is_authorized(&user_id, "insert") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            let ns = namespace_manager.read().unwrap().get_namespace(&namespace)
                .map_err(|e| LuaError::RuntimeError(format!("Namespace error: {}", e)))?;

            // Generate ID
            let id = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;

            // Generate embedding
            let embeddings = embedding.read().unwrap().generate(vec![content.as_str()])
                .map_err(|e| LuaError::RuntimeError(format!("Embedding error: {}", e)))?;
            let vector = embeddings.into_iter().next()
                .ok_or_else(|| LuaError::RuntimeError("Failed to generate embedding".to_string()))?;

            // Store content
            let content_key = format!("mem:{}:content", id);
            ns.db.put(content_key.as_bytes(), content.as_bytes())
                .map_err(|e| LuaError::RuntimeError(format!("Store error: {}", e)))?;

            // Store metadata with tags
            let meta = serde_json::json!({
                "id": id,
                "tags": tags.unwrap_or_default(),
                "created_at": SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
            });
            let meta_key = format!("mem:{}:meta", id);
            ns.db.put(meta_key.as_bytes(), meta.to_string().as_bytes())
                .map_err(|e| LuaError::RuntimeError(format!("Store error: {}", e)))?;

            // Store vector
            ns.vector_db.add(id, &vector)
                .map_err(|e| LuaError::RuntimeError(format!("Vector error: {}", e)))?;

            Ok(id)
        })?)?;

        // memory_recall(namespace, query, k) - Recall similar memories
        let user_id = user_id_str.clone();
        let namespace_manager = self.namespace_manager.clone();
        let embedding = self.embedding.clone();
        let auth_manager = self.auth_manager.clone();
        lua_ctx.globals().set("memory_recall", lua_ctx.create_function_mut(move |lua_ctx, (namespace, query, k): (String, String, usize)| {
            if !auth_manager.read().unwrap().is_authorized(&user_id, "select") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            let ns = namespace_manager.read().unwrap().get_namespace(&namespace)
                .map_err(|e| LuaError::RuntimeError(format!("Namespace error: {}", e)))?;

            // Generate query embedding
            let embeddings = embedding.read().unwrap().generate(vec![query.as_str()])
                .map_err(|e| LuaError::RuntimeError(format!("Embedding error: {}", e)))?;
            let query_vector = embeddings.into_iter().next()
                .ok_or_else(|| LuaError::RuntimeError("Failed to generate embedding".to_string()))?;

            // Search
            let results = ns.vector_db.search(&query_vector, k)
                .map_err(|e| LuaError::RuntimeError(format!("Search error: {}", e)))?;

            let lua_results = lua_ctx.create_table()?;
            for (i, (id, distance)) in results.into_iter().enumerate() {
                let result = lua_ctx.create_table()?;
                result.set("id", id)?;
                result.set("distance", distance)?;

                // Get content
                let content_key = format!("mem:{}:content", id);
                if let Ok(Some(content)) = ns.db.get(content_key.as_bytes()) {
                    result.set("content", String::from_utf8_lossy(&content).into_owned())?;
                }

                // Get metadata
                let meta_key = format!("mem:{}:meta", id);
                if let Ok(Some(meta)) = ns.db.get(meta_key.as_bytes()) {
                    if let Ok(meta_json) = serde_json::from_slice::<serde_json::Value>(&meta) {
                        if let Some(tags) = meta_json.get("tags").and_then(|t| t.as_array()) {
                            let lua_tags = lua_ctx.create_table()?;
                            for (j, tag) in tags.iter().enumerate() {
                                if let Some(s) = tag.as_str() {
                                    lua_tags.set(j + 1, s)?;
                                }
                            }
                            result.set("tags", lua_tags)?;
                        }
                        if let Some(ts) = meta_json.get("created_at").and_then(|t| t.as_u64()) {
                            result.set("created_at", ts)?;
                        }
                    }
                }

                lua_results.set(i + 1, result)?;
            }
            Ok(lua_results)
        })?)?;

        Ok(())
    }
}

// ============================================================
// JSON CONVERSION HELPERS
// ============================================================

fn lua_value_to_json(value: LuaValue) -> Result<serde_json::Value, LuaError> {
    match value {
        LuaValue::Nil => Ok(serde_json::Value::Null),
        LuaValue::Boolean(b) => Ok(serde_json::Value::Bool(b)),
        LuaValue::Integer(i) => Ok(serde_json::Value::Number(i.into())),
        LuaValue::Number(n) => {
            serde_json::Number::from_f64(n)
                .map(serde_json::Value::Number)
                .ok_or_else(|| LuaError::RuntimeError("Invalid number for JSON".to_string()))
        }
        LuaValue::String(s) => Ok(serde_json::Value::String(s.to_str()?.to_string())),
        LuaValue::Table(t) => {
            // Check if it's an array (sequential integer keys starting from 1)
            let mut is_array = true;
            let mut max_idx = 0i64;
            for pair in t.clone().pairs::<LuaValue, LuaValue>() {
                let (k, _) = pair?;
                match k {
                    LuaValue::Integer(i) if i > 0 => {
                        if i > max_idx {
                            max_idx = i;
                        }
                    }
                    _ => {
                        is_array = false;
                        break;
                    }
                }
            }

            if is_array && max_idx > 0 {
                let mut arr = Vec::new();
                for i in 1..=max_idx {
                    let v: LuaValue = t.get(i)?;
                    arr.push(lua_value_to_json(v)?);
                }
                Ok(serde_json::Value::Array(arr))
            } else {
                let mut map = serde_json::Map::new();
                for pair in t.pairs::<String, LuaValue>() {
                    let (k, v) = pair?;
                    map.insert(k, lua_value_to_json(v)?);
                }
                Ok(serde_json::Value::Object(map))
            }
        }
        _ => Err(LuaError::RuntimeError("Cannot convert value to JSON".to_string())),
    }
}

fn json_to_lua_value<'lua>(lua_ctx: LuaContext<'lua>, value: &serde_json::Value) -> Result<LuaValue<'lua>, LuaError> {
    match value {
        serde_json::Value::Null => Ok(LuaValue::Nil),
        serde_json::Value::Bool(b) => Ok(LuaValue::Boolean(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(LuaValue::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(LuaValue::Number(f))
            } else {
                Err(LuaError::RuntimeError("Invalid JSON number".to_string()))
            }
        }
        serde_json::Value::String(s) => {
            let lua_str = lua_ctx.create_string(s)?;
            Ok(LuaValue::String(lua_str))
        }
        serde_json::Value::Array(arr) => {
            let table = lua_ctx.create_table()?;
            for (i, v) in arr.iter().enumerate() {
                table.set(i + 1, json_to_lua_value(lua_ctx, v)?)?;
            }
            Ok(LuaValue::Table(table))
        }
        serde_json::Value::Object(obj) => {
            let table = lua_ctx.create_table()?;
            for (k, v) in obj.iter() {
                table.set(k.clone(), json_to_lua_value(lua_ctx, v)?)?;
            }
            Ok(LuaValue::Table(table))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rlua::Lua;

    #[test]
    fn test_lua_value_to_json_nil() {
        let nil = LuaValue::Nil;
        let json = lua_value_to_json(nil).unwrap();
        assert_eq!(json, serde_json::Value::Null);
    }

    #[test]
    fn test_lua_value_to_json_bool() {
        let val = LuaValue::Boolean(true);
        let json = lua_value_to_json(val).unwrap();
        assert_eq!(json, serde_json::Value::Bool(true));
    }

    #[test]
    fn test_lua_value_to_json_integer() {
        let val = LuaValue::Integer(42);
        let json = lua_value_to_json(val).unwrap();
        assert_eq!(json, serde_json::json!(42));
    }

    #[test]
    fn test_lua_value_to_json_number() {
        let val = LuaValue::Number(3.14);
        let json = lua_value_to_json(val).unwrap();
        assert_eq!(json, serde_json::json!(3.14));
    }

    #[test]
    fn test_lua_value_to_json_string() {
        let lua = Lua::new();
        let s = lua.create_string("hello").unwrap();
        let val = LuaValue::String(s);
        let json = lua_value_to_json(val).unwrap();
        assert_eq!(json, serde_json::json!("hello"));
    }

    #[test]
    fn test_lua_value_to_json_array() {
        let lua = Lua::new();
        let table = lua.create_table().unwrap();
        table.set(1, 10).unwrap();
        table.set(2, 20).unwrap();
        table.set(3, 30).unwrap();

        let val = LuaValue::Table(table);
        let json = lua_value_to_json(val).unwrap();
        assert_eq!(json, serde_json::json!([10, 20, 30]));
    }

    #[test]
    fn test_lua_value_to_json_object() {
        let lua = Lua::new();
        let table = lua.create_table().unwrap();
        table.set("name", "Alice").unwrap();
        table.set("age", 30).unwrap();

        let val = LuaValue::Table(table);
        let json = lua_value_to_json(val).unwrap();

        assert_eq!(json["name"], "Alice");
        assert_eq!(json["age"], 30);
    }

    #[test]
    fn test_json_to_lua_value_null() {
        let lua = Lua::new();
        let json = serde_json::Value::Null;
        let lua_val = json_to_lua_value(&lua, &json).unwrap();
        assert!(matches!(lua_val, LuaValue::Nil));
    }

    #[test]
    fn test_json_to_lua_value_bool() {
        let lua = Lua::new();
        let json = serde_json::json!(true);
        let lua_val = json_to_lua_value(&lua, &json).unwrap();
        assert!(matches!(lua_val, LuaValue::Boolean(true)));
    }

    #[test]
    fn test_json_to_lua_value_integer() {
        let lua = Lua::new();
        let json = serde_json::json!(42);
        let lua_val = json_to_lua_value(&lua, &json).unwrap();
        assert!(matches!(lua_val, LuaValue::Integer(42)));
    }

    #[test]
    fn test_json_to_lua_value_string() {
        let lua = Lua::new();
        let json = serde_json::json!("hello");
        let lua_val = json_to_lua_value(&lua, &json).unwrap();
        if let LuaValue::String(s) = lua_val {
            assert_eq!(s.to_str().unwrap(), "hello");
        } else {
            panic!("Expected LuaValue::String");
        }
    }

    #[test]
    fn test_json_to_lua_value_array() {
        let lua = Lua::new();
        let json = serde_json::json!([1, 2, 3]);
        let lua_val = json_to_lua_value(&lua, &json).unwrap();
        if let LuaValue::Table(t) = lua_val {
            let v1: i64 = t.get(1).unwrap();
            let v2: i64 = t.get(2).unwrap();
            let v3: i64 = t.get(3).unwrap();
            assert_eq!(v1, 1);
            assert_eq!(v2, 2);
            assert_eq!(v3, 3);
        } else {
            panic!("Expected LuaValue::Table");
        }
    }

    #[test]
    fn test_json_to_lua_value_object() {
        let lua = Lua::new();
        let json = serde_json::json!({"name": "Bob", "age": 25});
        let lua_val = json_to_lua_value(&lua, &json).unwrap();
        if let LuaValue::Table(t) = lua_val {
            let name: String = t.get("name").unwrap();
            let age: i64 = t.get("age").unwrap();
            assert_eq!(name, "Bob");
            assert_eq!(age, 25);
        } else {
            panic!("Expected LuaValue::Table");
        }
    }

    #[test]
    fn test_json_roundtrip() {
        let lua = Lua::new();

        // Create a complex Lua table
        let table = lua.create_table().unwrap();
        table.set("string", "hello").unwrap();
        table.set("number", 42).unwrap();
        table.set("bool", true).unwrap();

        let nested = lua.create_table().unwrap();
        nested.set("inner", "value").unwrap();
        table.set("nested", nested).unwrap();

        // Convert to JSON
        let json = lua_value_to_json(LuaValue::Table(table)).unwrap();

        // Convert back to Lua
        let lua_val = json_to_lua_value(&lua, &json).unwrap();

        // Verify
        if let LuaValue::Table(t) = lua_val {
            let s: String = t.get("string").unwrap();
            let n: i64 = t.get("number").unwrap();
            let b: bool = t.get("bool").unwrap();
            assert_eq!(s, "hello");
            assert_eq!(n, 42);
            assert!(b);

            let nested_t: rlua::Table = t.get("nested").unwrap();
            let inner: String = nested_t.get("inner").unwrap();
            assert_eq!(inner, "value");
        } else {
            panic!("Expected LuaValue::Table");
        }
    }
}
