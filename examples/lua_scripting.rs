//! Lua scripting example
//!
//! This example demonstrates how to use Liath's Lua scripting interface
//! for querying and manipulating data.

use liath::{EmbeddedLiath, Config};
use usearch::{MetricKind, ScalarKind};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Liath Lua Scripting Example ===\n");

    // Create database with temporary storage
    let config = Config {
        data_dir: std::env::temp_dir().join("liath_lua_example"),
        ..Default::default()
    };

    let db = EmbeddedLiath::new(config)?;
    let executor = db.query_executor();
    println!("Database initialized.\n");

    // Create a namespace for our examples
    db.create_namespace("lua_test", 384, MetricKind::Cos, ScalarKind::F32)?;
    println!("Created 'lua_test' namespace.\n");

    // === Basic Lua Expressions ===
    println!("=== Basic Lua Expressions ===\n");

    let result = executor.execute("return 1 + 2 * 3", "admin").await?;
    println!("1 + 2 * 3 = {}", result);

    let result = executor.execute("return 'Hello, ' .. 'Liath!'", "admin").await?;
    println!("String concat: {}", result);

    let result = executor.execute("return math.sqrt(144)", "admin").await?;
    println!("sqrt(144) = {}", result);
    println!();

    // === Key-Value Operations ===
    println!("=== Key-Value Operations via Lua ===\n");

    // Insert values
    executor.execute(r#"insert("lua_test", "name", "Alice")"#, "admin").await?;
    executor.execute(r#"insert("lua_test", "age", "30")"#, "admin").await?;
    executor.execute(r#"insert("lua_test", "city", "New York")"#, "admin").await?;
    println!("Inserted: name=Alice, age=30, city=New York");

    // Select values
    let name = executor.execute(r#"return select("lua_test", "name")"#, "admin").await?;
    let age = executor.execute(r#"return select("lua_test", "age")"#, "admin").await?;
    println!("Retrieved: name={}, age={}", name, age);

    // Delete a value
    executor.execute(r#"delete("lua_test", "city")"#, "admin").await?;
    let city = executor.execute(r#"return select("lua_test", "city") or "(nil)")"#, "admin").await?;
    println!("After delete, city={}\n", city);

    // === Namespace Operations ===
    println!("=== Namespace Operations via Lua ===\n");

    // Create a new namespace
    executor.execute(
        r#"create_namespace("products", 384, "cosine", "f32")"#,
        "admin"
    ).await?;
    println!("Created 'products' namespace");

    // List namespaces
    let result = executor.execute(r#"
        local ns = list_namespaces()
        local result = ""
        for i, name in ipairs(ns) do
            result = result .. name
            if i < #ns then result = result .. ", " end
        end
        return result
    "#, "admin").await?;
    println!("Namespaces: {}\n", result);

    // === Document Storage and Search ===
    println!("=== Document Storage and Search ===\n");

    // Store documents with embeddings
    executor.execute(r#"store_document("products", "p1", "Fast gaming laptop with RTX 4080", 1)"#, "admin").await?;
    executor.execute(r#"store_document("products", "p2", "Wireless ergonomic keyboard", 2)"#, "admin").await?;
    executor.execute(r#"store_document("products", "p3", "High-resolution 4K monitor", 3)"#, "admin").await?;
    println!("Stored 3 product documents");

    // Semantic search
    let result = executor.execute(r#"
        local results = semantic_search("products", "computer for gaming", 2)
        local output = ""
        for i, r in ipairs(results) do
            output = output .. r.key .. ": " .. r.content
            if i < #results then output = output .. " | " end
        end
        return output
    "#, "admin").await?;
    println!("Search 'computer for gaming': {}\n", result);

    // === Using Liath Standard Library ===
    println!("=== Liath Standard Library ===\n");

    // Using liath.util functions
    let result = executor.execute(r#"
        local arr = {1, 2, 3, 4, 5}
        local doubled = liath.util.map(arr, function(x) return x * 2 end)
        local sum = liath.util.reduce(doubled, function(acc, x) return acc + x end, 0)
        return "Sum of doubled: " .. sum
    "#, "admin").await?;
    println!("{}", result);

    let result = executor.execute(r#"
        local arr = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10}
        local evens = liath.util.filter(arr, function(x) return x % 2 == 0 end)
        return "Evens: " .. table.concat(evens, ", ")
    "#, "admin").await?;
    println!("{}", result);

    // Generate a unique ID
    let result = executor.execute(r#"return "Generated ID: " .. liath.util.id()"#, "admin").await?;
    println!("{}", result);

    // Get current timestamp
    let result = executor.execute(r#"return "Current timestamp: " .. liath.util.now()"#, "admin").await?;
    println!("{}\n", result);

    // === Complex Lua Scripts ===
    println!("=== Complex Lua Scripts ===\n");

    let result = executor.execute(r#"
        -- Store some user data
        local users = {
            {id = "u1", name = "Alice", score = 85},
            {id = "u2", name = "Bob", score = 92},
            {id = "u3", name = "Charlie", score = 78}
        }

        for _, user in ipairs(users) do
            local key = "user:" .. user.id
            local value = user.name .. ":" .. user.score
            insert("lua_test", key, value)
        end

        -- Calculate average score
        local total = 0
        for _, user in ipairs(users) do
            total = total + user.score
        end
        local avg = total / #users

        return string.format("Stored %d users. Average score: %.2f", #users, avg)
    "#, "admin").await?;
    println!("{}", result);

    // Retrieve and process the data
    let result = executor.execute(r#"
        local user1 = select("lua_test", "user:u1")
        local user2 = select("lua_test", "user:u2")
        return "user:u1 = " .. (user1 or "nil") .. ", user:u2 = " .. (user2 or "nil")
    "#, "admin").await?;
    println!("{}", result);

    println!("\nLua scripting example completed!");
    Ok(())
}
