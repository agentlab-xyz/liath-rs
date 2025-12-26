use rlua::{Lua, Result as LuaResult, Context, Error as LuaError, RluaCompat};
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Embedded Liath standard library
const LIATH_STDLIB: &str = include_str!("../../lua/liath.lua");

pub struct LuaVM {
    lua: Lua,
    installed_packages: Arc<RwLock<HashMap<String, String>>>,
    #[allow(dead_code)]
    luarocks_path: PathBuf,
}

impl LuaVM {
    #[allow(deprecated)]
    pub fn new(luarocks_path: PathBuf) -> LuaResult<Self> {
        let lua = Lua::new();

        // Register print function and initialize standard library
        lua.context(|ctx| {
            // Print function
            ctx.globals().set("print", ctx.create_function(|_, msg: String| {
                println!("{}", msg);
                Ok(())
            })?)?;

            // UUID function
            ctx.globals().set("uuid", ctx.create_function(|_, ()| {
                Ok(Uuid::new_v4().to_string())
            })?)?;

            // Timestamp function (milliseconds since epoch)
            ctx.globals().set("timestamp", ctx.create_function(|_, ()| {
                let duration = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default();
                Ok(duration.as_millis() as i64)
            })?)?;

            // Sleep function (milliseconds)
            ctx.globals().set("sleep", ctx.create_function(|_, ms: u64| {
                std::thread::sleep(std::time::Duration::from_millis(ms));
                Ok(())
            })?)?;

            // Load and register the liath standard library
            let liath_module: rlua::Value = ctx.load(LIATH_STDLIB).eval()?;
            ctx.globals().set("liath", liath_module)?;

            Ok::<(), LuaError>(())
        })?;

        Ok(Self {
            lua,
            installed_packages: Arc::new(RwLock::new(HashMap::new())),
            luarocks_path,
        })
    }

    pub fn execute(&self, code: &str) -> LuaResult<()> {
        #[allow(deprecated)]
        self.lua.context(|ctx| ctx.load(code).exec())
    }

    #[allow(deprecated)]
    pub fn execute_with_context<F, R>(&self, f: F) -> LuaResult<R>
    where
        F: FnOnce(Context) -> LuaResult<R>,
    {
        self.lua.context(f)
    }
    
    pub fn install_package(&self, package_name: &str) -> LuaResult<()> {
        // Mock package installation
        self.installed_packages.write().unwrap().insert(
            package_name.to_string(), 
            format!("Mock installation of {}", package_name)
        );
        println!("Installed package: {}", package_name);
        Ok(())
    }
    
    pub fn list_installed_packages(&self) -> LuaResult<Vec<String>> {
        let packages = self.installed_packages.read().unwrap();
        Ok(packages.keys().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua_vm_creation() {
        let vm = LuaVM::new(PathBuf::from("luarocks")).unwrap();
        // Just verify it creates without error
        assert!(vm.execute("local x = 1 + 1").is_ok());
    }

    #[test]
    fn test_lua_stdlib_loaded() {
        let vm = LuaVM::new(PathBuf::from("luarocks")).unwrap();

        // Check that liath module is available
        let result = vm.execute_with_context(|ctx| {
            let globals = ctx.globals();
            let liath: rlua::Table = globals.get("liath")?;

            // Check for key modules
            let _docs: rlua::Table = liath.get("docs")?;
            let _kv: rlua::Table = liath.get("kv")?;
            let _memory: rlua::Table = liath.get("memory")?;
            let _conversation: rlua::Table = liath.get("conversation")?;
            let _agent: rlua::Table = liath.get("agent")?;
            let _util: rlua::Table = liath.get("util")?;
            let _rag: rlua::Table = liath.get("rag")?;

            Ok(())
        });

        assert!(result.is_ok(), "liath stdlib should be loaded");
    }

    #[test]
    fn test_lua_util_functions() {
        let vm = LuaVM::new(PathBuf::from("luarocks")).unwrap();

        // Test liath.util.map
        let result = vm.execute_with_context(|ctx| {
            let code = r#"
                local arr = {1, 2, 3}
                local doubled = liath.util.map(arr, function(n) return n * 2 end)
                return doubled[1] + doubled[2] + doubled[3]
            "#;
            let result: i64 = ctx.load(code).eval()?;
            Ok(result)
        });

        assert_eq!(result.unwrap(), 12); // 2 + 4 + 6 = 12
    }

    #[test]
    fn test_lua_util_filter() {
        let vm = LuaVM::new(PathBuf::from("luarocks")).unwrap();

        let result = vm.execute_with_context(|ctx| {
            let code = r#"
                local arr = {1, 2, 3, 4, 5}
                local evens = liath.util.filter(arr, function(n) return n % 2 == 0 end)
                return #evens
            "#;
            let result: i64 = ctx.load(code).eval()?;
            Ok(result)
        });

        assert_eq!(result.unwrap(), 2); // {2, 4}
    }

    #[test]
    fn test_lua_util_reduce() {
        let vm = LuaVM::new(PathBuf::from("luarocks")).unwrap();

        let result = vm.execute_with_context(|ctx| {
            let code = r#"
                local arr = {1, 2, 3, 4, 5}
                return liath.util.reduce(arr, function(acc, n) return acc + n end, 0)
            "#;
            let result: i64 = ctx.load(code).eval()?;
            Ok(result)
        });

        assert_eq!(result.unwrap(), 15); // 1+2+3+4+5
    }

    #[test]
    fn test_lua_util_inspect() {
        let vm = LuaVM::new(PathBuf::from("luarocks")).unwrap();

        let result = vm.execute_with_context(|ctx| {
            let code = r#"
                local t = {a = 1, b = "hello"}
                local s = liath.util.inspect(t)
                return type(s) == "string"
            "#;
            let result: bool = ctx.load(code).eval()?;
            Ok(result)
        });

        assert!(result.unwrap());
    }

    #[test]
    fn test_lua_print() {
        let vm = LuaVM::new(PathBuf::from("luarocks")).unwrap();

        // Print should work without error
        let result = vm.execute(r#"print("Hello from Lua!")"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_lua_package_management() {
        let vm = LuaVM::new(PathBuf::from("luarocks")).unwrap();

        // Install a mock package
        vm.install_package("test-package").unwrap();

        let packages = vm.list_installed_packages().unwrap();
        assert!(packages.contains(&"test-package".to_string()));
    }
}
