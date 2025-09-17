use rlua::{Lua, Result as LuaResult, Context, Error as LuaError, RluaCompat};
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct LuaVM {
    lua: Lua,
    installed_packages: Arc<RwLock<HashMap<String, String>>>,
    luarocks_path: PathBuf,
}

impl LuaVM {
    pub fn new(luarocks_path: PathBuf) -> LuaResult<Self> {
        let lua = Lua::new();
        
        // Register custom functions
        lua.context(|ctx| {
            ctx.globals().set("print", ctx.create_function(|_, msg: String| {
                println!("{}", msg);
                Ok(())
            })?)?;
            
            Ok::<(), LuaError>(())
        })?;
        
        Ok(Self {
            lua,
            installed_packages: Arc::new(RwLock::new(HashMap::new())),
            luarocks_path,
        })
    }
    
    pub fn execute(&self, code: &str) -> LuaResult<()> {
        self.lua.context(|ctx| ctx.load(code).exec())
    }
    
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
