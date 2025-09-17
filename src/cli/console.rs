use std::io::{self, Write};
use crate::query::QueryExecutor;
use anyhow::Result;
#[cfg(feature = "vector")]
use usearch::{MetricKind, ScalarKind};

pub async fn run(query_executor: QueryExecutor) -> Result<()> {
    println!("Welcome to AI-First DB CLI");
    println!("Enter your queries or type 'exit' to quit");

    let mut user_id = String::new();
    print!("Enter your user ID: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut user_id)?;
    let user_id = user_id.trim();

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input = input.trim();

        if input.eq_ignore_ascii_case("exit") {
            break;
        }

        // Simple helper commands
        if input.starts_with(':') {
            let parts: Vec<&str> = input[1..].split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }
            match parts[0] {
                "ns" if parts.len() >= 2 => {
                    match parts[1] {
                        "list" => {
                            let nss = query_executor.list_namespaces();
                            println!("Namespaces: {:?}", nss);
                        }
                        "create" if parts.len() == 6 => {
                            let name = parts[2];
                            let dims: usize = parts[3].parse().unwrap_or(384);
                            #[cfg(feature = "vector")]
                            let metric = match parts[4].to_lowercase().as_str() {
                                "cosine" => MetricKind::Cos,
                                "euclidean" => MetricKind::L2sq,
                                _ => MetricKind::Cos,
                            };
                            #[cfg(feature = "vector")]
                            let scalar = match parts[5].to_lowercase().as_str() {
                                "f16" => ScalarKind::F16,
                                _ => ScalarKind::F32,
                            };
                            #[cfg(feature = "vector")]
                            let res = query_executor.create_namespace(name, dims, metric, scalar);
                            #[cfg(not(feature = "vector"))]
                            let res = query_executor.create_namespace_basic(name);
                            if let Err(e) = res {
                                eprintln!("Error: {}", e);
                            } else {
                                println!("Created namespace '{}'.", name);
                            }
                        }
                        _ => eprintln!("Usage: :ns list | :ns create <name> <dims> <cosine|euclidean> <f32|f16>"),
                    }
                }
                "put" if parts.len() >= 4 => {
                    let ns = parts[1];
                    let key = parts[2].as_bytes();
                    let value = parts[3..].join(" ");
                    if let Err(e) = query_executor.put(ns, key, value.as_bytes()) {
                        eprintln!("Error: {}", e);
                    } else {
                        println!("OK");
                    }
                }
                "get" if parts.len() == 3 => {
                    let ns = parts[1];
                    let key = parts[2].as_bytes();
                    match query_executor.get(ns, key) {
                        Ok(Some(v)) => println!("{}", String::from_utf8_lossy(&v)),
                        Ok(None) => println!("(nil)"),
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
                "del" if parts.len() == 3 => {
                    let ns = parts[1];
                    let key = parts[2].as_bytes();
                    if let Err(e) = query_executor.delete(ns, key) {
                        eprintln!("Error: {}", e);
                    } else {
                        println!("OK");
                    }
                }
                _ => eprintln!("Unknown command. Available: :ns, :put, :get, :del"),
            }
            continue;
        }

        match query_executor.execute(input, user_id).await {
            Ok(result) => println!("Result: {}", result),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    println!("Goodbye!");
    Ok(())
}
