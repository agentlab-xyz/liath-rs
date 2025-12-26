//! Integration tests for the Liath library

use tempfile::TempDir;

#[test]
fn test_library_structure() {
    // Verify that we can import the main types
    use liath::LiathResult;

    // Check that types exist and can be used
    let result: LiathResult<i32> = Ok(42);
    assert!(result.is_ok());
}

#[test]
fn test_config_default() {
    use liath::Config;

    let config = Config::default();
    assert_eq!(config.data_dir.to_str().unwrap(), "./data");
    assert!(config.luarocks_path.is_none());
}

#[test]
fn test_namespace_manager_kv_operations() {
    use liath::NamespaceManager;
    use usearch::{MetricKind, ScalarKind};

    let temp_dir = TempDir::new().unwrap();
    let manager = NamespaceManager::new(temp_dir.path().to_path_buf()).unwrap();

    // Create namespace
    manager.create_namespace("test", 128, MetricKind::Cos, ScalarKind::F32).unwrap();
    assert!(manager.namespace_exists("test"));

    // Get namespace and test KV operations
    let ns = manager.get_namespace("test").unwrap();
    ns.db.put(b"key1", b"value1").unwrap();

    let value = ns.db.get(b"key1").unwrap();
    assert_eq!(value, Some(b"value1".to_vec()));

    ns.db.delete(b"key1").unwrap();
    let value = ns.db.get(b"key1").unwrap();
    assert!(value.is_none());
}

#[test]
fn test_namespace_manager_vector_operations() {
    use liath::NamespaceManager;
    use usearch::{MetricKind, ScalarKind};

    let temp_dir = TempDir::new().unwrap();
    let manager = NamespaceManager::new(temp_dir.path().to_path_buf()).unwrap();

    // Create namespace with standard dimensions
    manager.create_namespace("vectors", 128, MetricKind::Cos, ScalarKind::F32).unwrap();

    let ns = manager.get_namespace("vectors").unwrap();

    // Reserve capacity before adding vectors
    ns.vector_db.reserve(10).unwrap();

    // Create test vectors
    let mut vec1 = vec![0.0f32; 128];
    vec1[0] = 1.0;
    let mut vec2 = vec![0.0f32; 128];
    vec2[1] = 1.0;

    // Add vectors
    ns.vector_db.add(1, &vec1).unwrap();
    ns.vector_db.add(2, &vec2).unwrap();

    // Verify size
    assert_eq!(ns.vector_db.size(), 2);
}

#[test]
fn test_namespace_persistence() {
    use liath::NamespaceManager;
    use usearch::{MetricKind, ScalarKind};

    let temp_dir = TempDir::new().unwrap();
    let data_path = temp_dir.path().to_path_buf();

    // Create namespace and add data
    {
        let manager = NamespaceManager::new(data_path.clone()).unwrap();
        manager.create_namespace("persistent", 128, MetricKind::Cos, ScalarKind::F32).unwrap();
        let ns = manager.get_namespace("persistent").unwrap();
        ns.db.put(b"test_key", b"test_value").unwrap();
        manager.save_all().unwrap();
    }

    // Reopen and verify
    {
        let manager = NamespaceManager::new(data_path).unwrap();
        assert!(manager.namespace_exists("persistent"));
        let ns = manager.get_namespace("persistent").unwrap();
        let value = ns.db.get(b"test_key").unwrap();
        assert_eq!(value, Some(b"test_value".to_vec()));
    }
}

#[test]
fn test_auth_manager_basic() {
    use liath::AuthManager;

    let mut auth = AuthManager::new();
    auth.add_user("alice", vec!["read".to_string(), "write".to_string()]);

    assert!(auth.is_authorized("alice", "read"));
    assert!(auth.is_authorized("alice", "write"));
    assert!(!auth.is_authorized("alice", "admin"));
    assert!(!auth.is_authorized("bob", "read"));
}

#[test]
fn test_auth_manager_persistence() {
    use liath::AuthManager;

    let temp_dir = TempDir::new().unwrap();
    let data_path = temp_dir.path();

    // Create and persist
    {
        let mut auth = AuthManager::with_persistence(data_path).unwrap();
        auth.add_user("alice", vec!["read".to_string(), "admin".to_string()]);
        auth.flush().unwrap();
    }

    // Reopen and verify
    {
        let auth = AuthManager::with_persistence(data_path).unwrap();
        assert!(auth.is_authorized("alice", "read"));
        assert!(auth.is_authorized("alice", "admin"));
        assert!(!auth.is_authorized("alice", "write"));
    }
}

#[test]
fn test_error_types() {
    use liath::{LiathError, LiathResult};

    let err: LiathError = LiathError::NamespaceNotFound("test".to_string());
    assert!(err.to_string().contains("test"));

    let result: LiathResult<()> = Err(LiathError::Unauthorized("not allowed".to_string()));
    assert!(result.is_err());
}

#[test]
fn test_fjall_wrapper_batch_operations() {
    use liath::FjallWrapper;

    let temp_dir = TempDir::new().unwrap();
    let wrapper = FjallWrapper::new(temp_dir.path()).unwrap();

    // Batch put
    let items: Vec<(&[u8], &[u8])> = vec![
        (b"key1", b"value1"),
        (b"key2", b"value2"),
        (b"key3", b"value3"),
    ];
    wrapper.batch_put(items).unwrap();

    // Verify all keys
    assert_eq!(wrapper.get(b"key1").unwrap(), Some(b"value1".to_vec()));
    assert_eq!(wrapper.get(b"key2").unwrap(), Some(b"value2".to_vec()));
    assert_eq!(wrapper.get(b"key3").unwrap(), Some(b"value3".to_vec()));
}

#[test]
fn test_agent_types() {
    use liath::agent::{Role, Message};

    let msg = Message {
        id: 1,
        role: Role::User,
        content: "Hello".to_string(),
        timestamp: 12345,
    };

    assert_eq!(msg.role.as_str(), "user");
    assert_eq!(msg.content, "Hello");

    let tool_role = Role::Tool("calculator".to_string());
    assert_eq!(tool_role.as_str(), "tool");
}

// ============================================================
// LUA API TESTS
// ============================================================

#[test]
fn test_lua_vm_basic() {
    use liath::LuaVM;
    use std::path::PathBuf;

    let vm = LuaVM::new(PathBuf::from("luarocks")).unwrap();

    // Basic Lua execution
    let result = vm.execute("local x = 1 + 1");
    assert!(result.is_ok());
}

#[test]
fn test_lua_stdlib_modules() {
    use liath::LuaVM;
    use std::path::PathBuf;

    let vm = LuaVM::new(PathBuf::from("luarocks")).unwrap();

    // Check liath modules exist
    let result = vm.execute(r#"
        assert(liath ~= nil, "liath module should exist")
        assert(liath.docs ~= nil, "liath.docs should exist")
        assert(liath.kv ~= nil, "liath.kv should exist")
        assert(liath.memory ~= nil, "liath.memory should exist")
        assert(liath.conversation ~= nil, "liath.conversation should exist")
        assert(liath.agent ~= nil, "liath.agent should exist")
        assert(liath.util ~= nil, "liath.util should exist")
        assert(liath.rag ~= nil, "liath.rag should exist")
    "#);

    assert!(result.is_ok(), "liath stdlib modules should exist: {:?}", result);
}

#[test]
fn test_lua_util_map() {
    use liath::LuaVM;
    use std::path::PathBuf;

    let vm = LuaVM::new(PathBuf::from("luarocks")).unwrap();

    let result = vm.execute(r#"
        local arr = {1, 2, 3}
        local doubled = liath.util.map(arr, function(n) return n * 2 end)
        assert(doubled[1] == 2, "First element should be 2")
        assert(doubled[2] == 4, "Second element should be 4")
        assert(doubled[3] == 6, "Third element should be 6")
    "#);

    assert!(result.is_ok(), "liath.util.map should work: {:?}", result);
}

#[test]
fn test_lua_util_filter() {
    use liath::LuaVM;
    use std::path::PathBuf;

    let vm = LuaVM::new(PathBuf::from("luarocks")).unwrap();

    let result = vm.execute(r#"
        local arr = {1, 2, 3, 4, 5}
        local evens = liath.util.filter(arr, function(n) return n % 2 == 0 end)
        assert(#evens == 2, "Should have 2 even numbers")
        assert(evens[1] == 2, "First even should be 2")
        assert(evens[2] == 4, "Second even should be 4")
    "#);

    assert!(result.is_ok(), "liath.util.filter should work: {:?}", result);
}

#[test]
fn test_lua_util_reduce() {
    use liath::LuaVM;
    use std::path::PathBuf;

    let vm = LuaVM::new(PathBuf::from("luarocks")).unwrap();

    let result = vm.execute(r#"
        local arr = {1, 2, 3, 4, 5}
        local sum = liath.util.reduce(arr, function(acc, n) return acc + n end, 0)
        assert(sum == 15, "Sum should be 15, got " .. sum)
    "#);

    assert!(result.is_ok(), "liath.util.reduce should work: {:?}", result);
}

#[test]
fn test_lua_util_inspect() {
    use liath::LuaVM;
    use std::path::PathBuf;

    let vm = LuaVM::new(PathBuf::from("luarocks")).unwrap();

    let result = vm.execute(r#"
        local t = {a = 1, b = "hello", c = {nested = true}}
        local s = liath.util.inspect(t)
        assert(type(s) == "string", "inspect should return a string")
        assert(#s > 0, "inspect should return non-empty string")
    "#);

    assert!(result.is_ok(), "liath.util.inspect should work: {:?}", result);
}

#[test]
fn test_lua_util_id() {
    use liath::LuaVM;
    use std::path::PathBuf;

    let vm = LuaVM::new(PathBuf::from("luarocks")).unwrap();

    let result = vm.execute(r#"
        local id = liath.util.id()
        assert(type(id) == "string", "id should return a string")
        assert(#id > 0, "id should return non-empty string")
    "#);

    assert!(result.is_ok(), "liath.util.id should work: {:?}", result);
}

#[test]
fn test_lua_util_now() {
    use liath::LuaVM;
    use std::path::PathBuf;

    let vm = LuaVM::new(PathBuf::from("luarocks")).unwrap();

    let result = vm.execute(r#"
        local ts = liath.util.now()
        assert(type(ts) == "number", "now should return a number")
        assert(ts > 0, "timestamp should be positive")
    "#);

    assert!(result.is_ok(), "liath.util.now should work: {:?}", result);
}

// ============================================================
// EMBEDDED LIATH TESTS
// ============================================================

#[test]
fn test_embedded_liath_creation() {
    use liath::{EmbeddedLiath, Config};

    let temp_dir = TempDir::new().unwrap();
    let config = Config {
        data_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let liath = EmbeddedLiath::new(config);
    assert!(liath.is_ok(), "EmbeddedLiath should be created successfully");
}

#[test]
fn test_embedded_liath_kv_operations() {
    use liath::{EmbeddedLiath, Config};
    use usearch::{MetricKind, ScalarKind};

    let temp_dir = TempDir::new().unwrap();
    let config = Config {
        data_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let liath = EmbeddedLiath::new(config).unwrap();

    // Create namespace
    liath.create_namespace("test_ns", 128, MetricKind::Cos, ScalarKind::F32).unwrap();
    assert!(liath.namespace_exists("test_ns"));

    // KV operations
    liath.put("test_ns", b"key1", b"value1").unwrap();
    let value = liath.get("test_ns", b"key1").unwrap();
    assert_eq!(value, Some(b"value1".to_vec()));

    liath.delete("test_ns", b"key1").unwrap();
    let value = liath.get("test_ns", b"key1").unwrap();
    assert!(value.is_none());
}

#[test]
fn test_embedded_liath_namespace_management() {
    use liath::{EmbeddedLiath, Config};
    use usearch::{MetricKind, ScalarKind};

    let temp_dir = TempDir::new().unwrap();
    let config = Config {
        data_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let mut liath = EmbeddedLiath::new(config).unwrap();

    // Create multiple namespaces
    liath.create_namespace("ns1", 128, MetricKind::Cos, ScalarKind::F32).unwrap();
    liath.create_namespace("ns2", 256, MetricKind::L2sq, ScalarKind::F32).unwrap();

    let namespaces = liath.list_namespaces();
    assert!(namespaces.contains(&"ns1".to_string()));
    assert!(namespaces.contains(&"ns2".to_string()));

    // Set and get current namespace
    liath.set_namespace("ns1");
    assert_eq!(liath.current_namespace(), "ns1");
}

#[test]
fn test_embedded_liath_vector_operations() {
    use liath::{EmbeddedLiath, Config};
    use usearch::{MetricKind, ScalarKind};

    let temp_dir = TempDir::new().unwrap();
    let config = Config {
        data_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let liath = EmbeddedLiath::new(config).unwrap();
    liath.create_namespace("vectors", 128, MetricKind::Cos, ScalarKind::F32).unwrap();

    // Add vectors
    let vec1: Vec<f32> = (0..128).map(|i| if i == 0 { 1.0 } else { 0.0 }).collect();
    let vec2: Vec<f32> = (0..128).map(|i| if i == 1 { 1.0 } else { 0.0 }).collect();

    liath.add_vector("vectors", 1, &vec1).unwrap();
    liath.add_vector("vectors", 2, &vec2).unwrap();

    // Search vectors
    let results = liath.search_vectors("vectors", &vec1, 2).unwrap();
    assert!(!results.is_empty());
    // The first result should be the closest (vec1 itself)
    assert_eq!(results[0].0, 1);
}

// ============================================================
// QUERY EXECUTOR TESTS
// ============================================================

#[tokio::test]
async fn test_query_executor_basic_lua() {
    use liath::{EmbeddedLiath, Config};

    let temp_dir = TempDir::new().unwrap();
    let config = Config {
        data_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let liath = EmbeddedLiath::new(config).unwrap();
    let executor = liath.query_executor();

    // Execute basic Lua
    let result = executor.execute("return 1 + 1", "test_user").await;
    assert!(result.is_ok(), "Basic Lua should work: {:?}", result);
    assert_eq!(result.unwrap(), "2");
}

#[tokio::test]
async fn test_query_executor_namespace_operations() {
    use liath::{EmbeddedLiath, Config};

    let temp_dir = TempDir::new().unwrap();
    let config = Config {
        data_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let liath = EmbeddedLiath::new(config).unwrap();
    let executor = liath.query_executor();

    // Create namespace via Lua
    let result = executor.execute(
        r#"create_namespace("lua_ns", 128, "cosine", "f32")"#,
        "test_user"
    ).await;
    assert!(result.is_ok(), "Namespace creation should work: {:?}", result);

    // Verify namespace exists
    assert!(executor.namespace_exists("lua_ns"));
}

#[tokio::test]
async fn test_query_executor_kv_via_lua() {
    use liath::{EmbeddedLiath, Config};
    use usearch::{MetricKind, ScalarKind};

    let temp_dir = TempDir::new().unwrap();
    let config = Config {
        data_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let liath = EmbeddedLiath::new(config).unwrap();
    liath.create_namespace("lua_kv", 128, MetricKind::Cos, ScalarKind::F32).unwrap();
    let executor = liath.query_executor();

    // Insert via Lua
    let result = executor.execute(
        r#"insert("lua_kv", "mykey", "myvalue")"#,
        "test_user"
    ).await;
    assert!(result.is_ok(), "Insert should work: {:?}", result);

    // Select via Lua
    let result = executor.execute(
        r#"return select("lua_kv", "mykey")"#,
        "test_user"
    ).await;
    assert!(result.is_ok(), "Select should work: {:?}", result);
    assert_eq!(result.unwrap(), "myvalue");
}

// ============================================================
// AGENT MODULE INTEGRATION TESTS
// ============================================================

#[test]
fn test_agent_memory_store_and_recall() {
    use liath::{EmbeddedLiath, Config};
    use liath::agent::Agent;
    use std::sync::Arc;

    let temp_dir = TempDir::new().unwrap();
    let config = Config {
        data_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let db = Arc::new(EmbeddedLiath::new(config).unwrap());
    let agent = Agent::new("test-agent", db.clone());

    let memory = agent.memory().unwrap();

    // Store a memory
    let id = memory.store("The capital of France is Paris", &["geography", "facts"]).unwrap();
    assert!(id > 0);

    // Recall by semantic search
    let results = memory.recall("What is the capital of France?", 3).unwrap();
    assert!(!results.is_empty(), "Should find at least one result");
    assert!(results[0].content.contains("Paris"), "Result should contain Paris");
}

#[test]
fn test_agent_memory_recall_by_tags() {
    use liath::{EmbeddedLiath, Config};
    use liath::agent::Agent;
    use std::sync::Arc;

    let temp_dir = TempDir::new().unwrap();
    let config = Config {
        data_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let db = Arc::new(EmbeddedLiath::new(config).unwrap());
    let agent = Agent::new("tag-test-agent", db.clone());

    let memory = agent.memory().unwrap();

    // Store memories with different tags
    memory.store("Fact about geography", &["geography"]).unwrap();
    memory.store("Fact about history", &["history"]).unwrap();
    memory.store("Fact about geography and history", &["geography", "history"]).unwrap();

    // Recall by tags
    let geography_results = memory.recall_by_tags(&["geography"], 10).unwrap();
    assert_eq!(geography_results.len(), 2, "Should find 2 geography entries");

    let history_results = memory.recall_by_tags(&["history"], 10).unwrap();
    assert_eq!(history_results.len(), 2, "Should find 2 history entries");

    let both_results = memory.recall_by_tags(&["geography", "history"], 10).unwrap();
    assert_eq!(both_results.len(), 1, "Should find 1 entry with both tags");
}

#[test]
fn test_agent_conversation() {
    use liath::{EmbeddedLiath, Config};
    use liath::agent::{Agent, Role};
    use std::sync::Arc;

    let temp_dir = TempDir::new().unwrap();
    let config = Config {
        data_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let db = Arc::new(EmbeddedLiath::new(config).unwrap());
    let agent = Agent::new("conv-test-agent", db.clone());

    // Create a conversation
    let conv = agent.conversation(None).unwrap();

    conv.add_message(Role::User, "Hello!").unwrap();
    conv.add_message(Role::Assistant, "Hi there! How can I help?").unwrap();
    conv.add_message(Role::User, "What's the weather like?").unwrap();

    assert_eq!(conv.message_count(), 3);

    let messages = conv.messages().unwrap();
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0].content, "Hello!");
    assert_eq!(messages[1].role, Role::Assistant);
}

#[test]
fn test_agent_tool_state() {
    use liath::{EmbeddedLiath, Config};
    use liath::agent::Agent;
    use std::sync::Arc;

    let temp_dir = TempDir::new().unwrap();
    let config = Config {
        data_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let db = Arc::new(EmbeddedLiath::new(config).unwrap());
    let agent = Agent::new("tool-test-agent", db.clone());

    let state = agent.tool_state("calculator").unwrap();

    // Set values
    state.set("last_result", &42i32).unwrap();
    state.set("history", &vec!["1+1", "2+2"]).unwrap();

    // Get values
    let result: Option<i32> = state.get("last_result").unwrap();
    assert_eq!(result, Some(42));

    let history: Option<Vec<String>> = state.get("history").unwrap();
    assert_eq!(history, Some(vec!["1+1".to_string(), "2+2".to_string()]));
}

#[test]
fn test_agent_persistence() {
    use liath::{EmbeddedLiath, Config};
    use liath::agent::Agent;
    use std::sync::Arc;

    let temp_dir = TempDir::new().unwrap();
    let data_path = temp_dir.path().to_path_buf();

    // Create agent and store memory
    {
        let config = Config {
            data_dir: data_path.clone(),
            ..Default::default()
        };
        let db = Arc::new(EmbeddedLiath::new(config).unwrap());
        let agent = Agent::new_with_description("persist-agent", "Test agent for persistence", db.clone());

        let memory = agent.memory().unwrap();
        memory.store("Important fact to remember", &["test"]).unwrap();

        agent.save().unwrap();
    }

    // Reload and verify
    {
        let config = Config {
            data_dir: data_path,
            ..Default::default()
        };
        let db = Arc::new(EmbeddedLiath::new(config).unwrap());

        // Check agent exists
        assert!(Agent::exists("persist-agent", &db).unwrap());

        let agent = Agent::load("persist-agent", db.clone()).unwrap().unwrap();
        let metadata = agent.metadata().unwrap().unwrap();
        assert_eq!(metadata.description, Some("Test agent for persistence".to_string()));
    }
}

#[test]
fn test_agent_list_and_delete() {
    use liath::{EmbeddedLiath, Config};
    use liath::agent::Agent;
    use std::sync::Arc;

    let temp_dir = TempDir::new().unwrap();
    let config = Config {
        data_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let db = Arc::new(EmbeddedLiath::new(config).unwrap());

    // Create multiple agents
    let _agent1 = Agent::new("agent-1", db.clone());
    let _agent2 = Agent::new("agent-2", db.clone());
    let _agent3 = Agent::new("agent-3", db.clone());

    // List agents
    let agents = Agent::list_agents(&db).unwrap();
    assert!(agents.len() >= 3, "Should have at least 3 agents");

    // Delete one
    Agent::delete("agent-2", &db).unwrap();

    // Verify deletion
    assert!(!Agent::exists("agent-2", &db).unwrap());

    let agents = Agent::list_agents(&db).unwrap();
    assert!(!agents.iter().any(|a| a.id == "agent-2"));
}

// ============================================================
// SEMANTIC SEARCH END-TO-END TESTS
// ============================================================

#[test]
fn test_semantic_search_with_content_mapping() {
    use liath::{EmbeddedLiath, Config};
    use usearch::{MetricKind, ScalarKind};

    let temp_dir = TempDir::new().unwrap();
    let config = Config {
        data_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let liath = EmbeddedLiath::new(config).unwrap();
    liath.create_namespace("docs", 384, MetricKind::Cos, ScalarKind::F32).unwrap();

    // Store documents with embeddings
    liath.store_with_embedding("docs", 1, b"doc1", "The quick brown fox jumps over the lazy dog").unwrap();
    liath.store_with_embedding("docs", 2, b"doc2", "A fast red fox leaps over a sleepy hound").unwrap();
    liath.store_with_embedding("docs", 3, b"doc3", "The weather is sunny today").unwrap();

    // Semantic search should return content
    let results = liath.semantic_search("docs", "fox jumping", 3).unwrap();

    assert!(!results.is_empty(), "Should find results");
    // The fox-related documents should rank higher
    let first_result = &results[0];
    assert!(
        first_result.1.contains("fox") || first_result.1.contains("Fox"),
        "First result should be about foxes: {}", first_result.1
    );
}