# Tool State

This guide covers managing persistent state for agent tools in Liath. Tool state allows your tools to remember information across invocations.

## Overview

Tool state provides:

- **Persistence**: State survives process restarts
- **Isolation**: Each tool has its own state space
- **Type Safety**: JSON serialization with type checking
- **Simplicity**: Key-value interface

## Basic Usage

### Creating Tool State

```rust
use liath::agent::Agent;

let agent = Agent::new("my-agent", db.clone());

// Get state for a specific tool
let state = agent.tool_state("calculator")?;
```

### Storing Values

```rust
// Primitives
state.set("last_result", &42.5f64)?;
state.set("operation_count", &100u32)?;
state.set("enabled", &true)?;
state.set("name", &"Calculator")?;

// Collections
state.set("history", &vec!["1+1", "2*3", "sqrt(16)"])?;

// Complex types
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Settings {
    precision: u32,
    mode: String,
    scientific_notation: bool,
}

state.set("settings", &Settings {
    precision: 10,
    mode: "standard".to_string(),
    scientific_notation: false,
})?;

// JSON values
state.set("config", &serde_json::json!({
    "theme": "dark",
    "font_size": 14
}))?;
```

### Retrieving Values

```rust
// With type annotation
let result: Option<f64> = state.get("last_result")?;
let count: Option<u32> = state.get("operation_count")?;
let history: Option<Vec<String>> = state.get("history")?;

// With turbofish
let settings = state.get::<Settings>("settings")?;

// Handle missing keys
match state.get::<f64>("nonexistent")? {
    Some(value) => println!("Found: {}", value),
    None => println!("Key not found"),
}
```

### Managing State

```rust
// Check existence
if state.exists("initialized")? {
    println!("Tool already initialized");
}

// Delete key
state.delete("old_key")?;

// Get tool info
println!("Agent: {}", state.agent_id());
println!("Tool: {}", state.tool_name());
```

## Tool Implementation Patterns

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
            state.set("operation_count", &0u32)?;
            state.set("history", &Vec::<String>::new())?;
            state.set("settings", &CalculatorSettings::default())?;
        }

        Ok(Self { state })
    }

    pub fn calculate(&self, expression: &str) -> Result<f64, Error> {
        // Parse and evaluate (simplified)
        let result = eval_expr(expression)?;

        // Update state
        let mut count: u32 = self.state.get("operation_count")?.unwrap_or(0);
        count += 1;
        self.state.set("operation_count", &count)?;

        let mut history: Vec<String> = self.state.get("history")?.unwrap_or_default();
        history.push(format!("{} = {}", expression, result));
        self.state.set("history", &history)?;

        self.state.set("last_result", &result)?;

        Ok(result)
    }
}
```

### Session State Pattern

```rust
pub struct WebBrowser {
    state: ToolState,
}

#[derive(Serialize, Deserialize, Default)]
struct BrowserSession {
    current_url: Option<String>,
    tabs: Vec<String>,
    history: Vec<String>,
    bookmarks: Vec<String>,
}

impl WebBrowser {
    pub fn new(agent: &Agent) -> Result<Self, Error> {
        let state = agent.tool_state("web_browser")?;

        if !state.exists("session")? {
            state.set("session", &BrowserSession::default())?;
        }

        Ok(Self { state })
    }

    fn get_session(&self) -> Result<BrowserSession, Error> {
        Ok(self.state.get("session")?.unwrap_or_default())
    }

    fn save_session(&self, session: &BrowserSession) -> Result<(), Error> {
        self.state.set("session", session)
    }

    pub fn navigate(&self, url: &str) -> Result<String, Error> {
        let mut session = self.get_session()?;

        session.history.push(session.current_url.clone().unwrap_or_default());
        session.current_url = Some(url.to_string());

        self.save_session(&session)?;

        // Fetch page content...
        Ok(format!("Navigated to {}", url))
    }

    pub fn open_tab(&self, url: &str) -> Result<(), Error> {
        let mut session = self.get_session()?;
        session.tabs.push(url.to_string());
        self.save_session(&session)
    }
}
```

### Configuration Pattern

```rust
pub struct CodeExecutor {
    state: ToolState,
}

#[derive(Serialize, Deserialize)]
struct ExecutorConfig {
    timeout_ms: u64,
    max_output_lines: usize,
    allowed_languages: Vec<String>,
    sandbox_enabled: bool,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 5000,
            max_output_lines: 100,
            allowed_languages: vec!["python".into(), "javascript".into()],
            sandbox_enabled: true,
        }
    }
}

impl CodeExecutor {
    pub fn new(agent: &Agent) -> Result<Self, Error> {
        let state = agent.tool_state("code_executor")?;

        if !state.exists("config")? {
            state.set("config", &ExecutorConfig::default())?;
        }

        Ok(Self { state })
    }

    pub fn configure(&self, key: &str, value: serde_json::Value) -> Result<(), Error> {
        let mut config: ExecutorConfig = self.state.get("config")?.unwrap_or_default();

        match key {
            "timeout_ms" => config.timeout_ms = value.as_u64().unwrap_or(5000),
            "max_output_lines" => config.max_output_lines = value.as_u64().unwrap_or(100) as usize,
            "sandbox_enabled" => config.sandbox_enabled = value.as_bool().unwrap_or(true),
            _ => return Err("Unknown config key".into()),
        }

        self.state.set("config", &config)
    }

    pub fn execute(&self, language: &str, code: &str) -> Result<String, Error> {
        let config: ExecutorConfig = self.state.get("config")?.unwrap_or_default();

        if !config.allowed_languages.contains(&language.to_string()) {
            return Err(format!("Language {} not allowed", language).into());
        }

        // Execute with timeout...
        Ok("Execution result".to_string())
    }
}
```

## Via Lua

### Basic Operations

```lua
-- Store tool state
local function tool_put(tool_name, key, value)
    local ns = "tools:" .. tool_name
    put(ns, key, json.encode(value))
end

local function tool_get(tool_name, key)
    local ns = "tools:" .. tool_name
    local raw = get(ns, key)
    return raw and json.decode(raw) or nil
end

-- Usage
tool_put("calculator", "last_result", 42.5)
local result = tool_get("calculator", "last_result")
```

### Tool Implementation

```lua
local Calculator = {}

function Calculator:new(agent_id)
    local tool = {
        agent_id = agent_id,
        ns = "agent:" .. agent_id .. ":tools:calculator"
    }
    setmetatable(tool, {__index = Calculator})

    -- Initialize
    if not get(tool.ns, "initialized") then
        put(tool.ns, "initialized", "true")
        put(tool.ns, "count", "0")
        put(tool.ns, "history", json.encode({}))
    end

    return tool
end

function Calculator:calculate(expr)
    -- Update count
    local count = tonumber(get(self.ns, "count") or "0")
    put(self.ns, "count", tostring(count + 1))

    -- Evaluate (simplified)
    local result = load("return " .. expr)()

    -- Update history
    local history = json.decode(get(self.ns, "history") or "[]")
    table.insert(history, {expr = expr, result = result})
    put(self.ns, "history", json.encode(history))

    put(self.ns, "last_result", tostring(result))

    return result
end

function Calculator:get_history()
    return json.decode(get(self.ns, "history") or "[]")
end

-- Usage
local calc = Calculator:new("agent-1")
local result = calc:calculate("2 + 2")
local history = calc:get_history()
```

## Best Practices

### 1. Use Typed State Objects

```rust
// Define explicit types for state
#[derive(Serialize, Deserialize, Default)]
struct MyToolState {
    version: u32,
    last_used: Option<u64>,
    settings: Settings,
    cache: HashMap<String, String>,
}

// Load/save entire state object
fn get_state(state: &ToolState) -> Result<MyToolState, Error> {
    Ok(state.get("state")?.unwrap_or_default())
}

fn save_state(state: &ToolState, data: &MyToolState) -> Result<(), Error> {
    state.set("state", data)
}
```

### 2. Handle Missing State Gracefully

```rust
// Always provide defaults
let count: u32 = state.get("count")?.unwrap_or(0);
let items: Vec<String> = state.get("items")?.unwrap_or_default();

// Or use initialization pattern
if !state.exists("initialized")? {
    initialize_defaults(&state)?;
}
```

### 3. Version State Schema

```rust
#[derive(Serialize, Deserialize)]
struct VersionedState {
    version: u32,
    data: StateData,
}

fn migrate_state(state: &ToolState) -> Result<(), Error> {
    let current: VersionedState = state.get("state")?.unwrap_or_default();

    match current.version {
        0 => {
            // Migrate from v0 to v1
            let migrated = migrate_v0_to_v1(current.data);
            state.set("state", &VersionedState { version: 1, data: migrated })?;
        }
        1 => {
            // Current version, no migration needed
        }
        _ => return Err("Unknown state version".into()),
    }

    Ok(())
}
```

### 4. Limit State Size

```rust
// Keep state bounded
fn add_to_history(state: &ToolState, item: &str) -> Result<(), Error> {
    const MAX_HISTORY: usize = 100;

    let mut history: Vec<String> = state.get("history")?.unwrap_or_default();
    history.push(item.to_string());

    // Trim if too large
    if history.len() > MAX_HISTORY {
        history = history.split_off(history.len() - MAX_HISTORY);
    }

    state.set("history", &history)
}
```

### 5. Clean Up Stale State

```rust
fn cleanup_tool_state(state: &ToolState, max_age_days: u64) -> Result<(), Error> {
    let last_used: Option<u64> = state.get("last_used")?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();

    if let Some(last) = last_used {
        if now - last > max_age_days * 24 * 3600 {
            // Clear old state
            state.delete("cache")?;
            state.delete("temp_data")?;
        }
    }

    state.set("last_used", &now)?;
    Ok(())
}
```

## Next Steps

- [Building AI Agents](building-agents.md) - Full agent with tools
- [Memory Patterns](memory-patterns.md) - Persistent memory
- [API Reference](../api/tool-state.md) - Complete ToolState API
