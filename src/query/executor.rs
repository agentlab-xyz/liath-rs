use crate::core::NamespaceManager;
use crate::ai::EmbeddingWrapper;
use crate::lua::LuaVM;
use crate::file::FileStorage;
use crate::auth::AuthManager;
use anyhow::Result;
use tokio::sync::Semaphore;
use std::sync::{Arc, RwLock};
use tracing::instrument;
use rlua::{Context as LuaContext, Error as LuaError, Value as LuaValue};
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

    fn register_db_functions(&self, lua_ctx: &LuaContext, user_id: &str) -> Result<(), LuaError> {
        let namespace_manager = self.namespace_manager.clone();
        let embedding = self.embedding.clone();
        let file_storage = self.file_storage.clone();
        let auth_manager = self.auth_manager.clone();
        let embedding_semaphore = self.embedding_semaphore.clone();
        let lua_vm = self.lua_vm.clone();

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

        Ok(())
    }
}
