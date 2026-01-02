# ToolState API

The ToolState API provides persistent key-value storage for agent tools.

## Overview

```rust
use liath::agent::{Agent, ToolState};

let agent = Agent::new("my-agent", db.clone());
let state = agent.tool_state("calculator")?;
```

## Creating ToolState

Access tool state through an Agent:

```rust
// Get state for a specific tool
let calc_state = agent.tool_state("calculator")?;
let browser_state = agent.tool_state("web_browser")?;
```

## Methods

### set

Store a value with JSON serialization.

```rust
fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<(), Error>
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `key` | `&str` | Storage key |
| `value` | `&T` | Value to store (must implement Serialize) |

**Example:**

```rust
// Primitives
state.set("count", &42u32)?;
state.set("enabled", &true)?;
state.set("ratio", &3.14f64)?;
state.set("name", &"Calculator")?;

// Collections
state.set("history", &vec!["1+1", "2*3", "sqrt(16)"])?;
state.set("scores", &HashMap::from([("math", 95), ("science", 87)]))?;

// Custom types
#[derive(Serialize)]
struct Config {
    precision: u32,
    mode: String,
}

state.set("config", &Config {
    precision: 10,
    mode: "scientific".into(),
})?;

// JSON values
state.set("metadata", &serde_json::json!({
    "version": "1.0",
    "features": ["basic", "advanced"]
}))?;
```

---

### get

Retrieve a value with JSON deserialization.

```rust
fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Error>
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `key` | `&str` | Storage key |

**Returns:** `Option<T>` - The value if found, None otherwise

**Example:**

```rust
// With type annotation
let count: Option<u32> = state.get("count")?;
let enabled: Option<bool> = state.get("enabled")?;

// With turbofish
let history = state.get::<Vec<String>>("history")?;

// Handle missing keys
match state.get::<u32>("nonexistent")? {
    Some(value) => println!("Found: {}", value),
    None => println!("Key not found"),
}

// With default
let count: u32 = state.get("count")?.unwrap_or(0);
```

---

### delete

Remove a key from storage.

```rust
fn delete(&self, key: &str) -> Result<(), Error>
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `key` | `&str` | Key to delete |

**Example:**

```rust
state.delete("temporary_data")?;
state.delete("cache")?;
```

---

### exists

Check if a key exists.

```rust
fn exists(&self, key: &str) -> Result<bool, Error>
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `key` | `&str` | Key to check |

**Returns:** `bool` - True if key exists

**Example:**

```rust
if !state.exists("initialized")? {
    // First-time initialization
    state.set("initialized", &true)?;
    state.set("version", &1u32)?;
}
```

---

### agent_id

Get the parent agent ID.

```rust
fn agent_id(&self) -> &str
```

---

### tool_name

Get the tool name.

```rust
fn tool_name(&self) -> &str
```

**Example:**

```rust
println!("Tool: {} for agent: {}", state.tool_name(), state.agent_id());
```

## Storage Details

### Namespace Structure

```
agent:{agent_id}:tools:{tool_name}:{key}
```

### Serialization

All values are JSON serialized:

```rust
// This...
state.set("config", &Config { precision: 10 })?;

// Stores as:
// key: "config"
// value: "{\"precision\":10}"
```

## Usage Patterns

### Initialization Pattern

```rust
pub struct Calculator {
    state: ToolState,
}

impl Calculator {
    pub fn new(agent: &Agent) -> Result<Self, Error> {
        let state = agent.tool_state("calculator")?;

        // Initialize on first use
        if !state.exists("initialized")? {
            state.set("initialized", &true)?;
            state.set("version", &1u32)?;
            state.set("operation_count", &0u64)?;
            state.set("history", &Vec::<String>::new())?;
            state.set("settings", &CalculatorSettings::default())?;
        }

        Ok(Self { state })
    }
}
```

### State Object Pattern

```rust
#[derive(Serialize, Deserialize, Default)]
struct BrowserState {
    current_url: Option<String>,
    tabs: Vec<String>,
    history: Vec<String>,
    bookmarks: Vec<String>,
}

impl WebBrowser {
    fn load_state(&self) -> Result<BrowserState, Error> {
        Ok(self.state.get("browser_state")?.unwrap_or_default())
    }

    fn save_state(&self, state: &BrowserState) -> Result<(), Error> {
        self.state.set("browser_state", state)
    }

    pub fn navigate(&self, url: &str) -> Result<(), Error> {
        let mut browser_state = self.load_state()?;

        if let Some(current) = &browser_state.current_url {
            browser_state.history.push(current.clone());
        }
        browser_state.current_url = Some(url.to_string());

        self.save_state(&browser_state)
    }
}
```

### Versioned State

```rust
#[derive(Serialize, Deserialize)]
struct VersionedState<T> {
    version: u32,
    data: T,
}

impl<T: Serialize + DeserializeOwned + Default> Tool {
    fn migrate_state(&self) -> Result<(), Error> {
        let current: VersionedState<OldData> = self.state
            .get("state")?
            .unwrap_or(VersionedState {
                version: 0,
                data: OldData::default(),
            });

        match current.version {
            0 => {
                // Migrate v0 -> v1
                let new_data = migrate_v0_to_v1(current.data);
                self.state.set("state", &VersionedState {
                    version: 1,
                    data: new_data,
                })?;
            }
            1 => {
                // Current version
            }
            _ => return Err("Unknown version".into()),
        }

        Ok(())
    }
}
```

### Bounded History

```rust
impl Tool {
    const MAX_HISTORY: usize = 100;

    fn add_to_history(&self, item: &str) -> Result<(), Error> {
        let mut history: Vec<String> = self.state
            .get("history")?
            .unwrap_or_default();

        history.push(item.to_string());

        // Keep bounded
        if history.len() > Self::MAX_HISTORY {
            history = history.split_off(history.len() - Self::MAX_HISTORY);
        }

        self.state.set("history", &history)
    }
}
```

### Session Tracking

```rust
impl Tool {
    pub fn start_session(&self) -> Result<String, Error> {
        let session_id = uuid::Uuid::new_v4().to_string();

        self.state.set("current_session", &session_id)?;
        self.state.set("session_start", &SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs())?;

        Ok(session_id)
    }

    pub fn end_session(&self) -> Result<(), Error> {
        self.state.delete("current_session")?;
        self.state.delete("session_start")?;
        Ok(())
    }

    pub fn is_session_active(&self) -> Result<bool, Error> {
        self.state.exists("current_session")
    }
}
```

## Error Handling

```rust
use liath::LiathError;

// Type mismatch
match state.get::<u32>("string_value") {
    Ok(Some(v)) => println!("Value: {}", v),
    Ok(None) => println!("Not found"),
    Err(LiathError::Serialization(e)) => {
        println!("Type mismatch: {}", e);
    }
    Err(e) => println!("Error: {}", e),
}

// Handle gracefully
fn get_or_default<T: DeserializeOwned + Default>(
    state: &ToolState,
    key: &str,
) -> T {
    state.get(key)
        .ok()
        .flatten()
        .unwrap_or_default()
}
```

## See Also

- [Agent API](agent-api.md) - Parent agent interface
- [Tool State Guide](../guides/tool-state.md) - Best practices
- [Building Agents Guide](../guides/building-agents.md) - Tool integration
