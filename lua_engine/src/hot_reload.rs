use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

pub struct SimpleHotReloader {
    scripts_dir: PathBuf,
    file_timestamps: HashMap<PathBuf, SystemTime>,
    lua: Arc<Mutex<mlua::Lua>>,
    check_interval: Duration,
    last_check: SystemTime,
}

impl SimpleHotReloader {
    pub fn new(lua: Arc<Mutex<mlua::Lua>>, scripts_dir: &Path) -> Self {
        // Create initial file timestamps map
        let mut reloader = Self {
            scripts_dir: scripts_dir.to_path_buf(),
            file_timestamps: HashMap::new(),
            lua,
            check_interval: Duration::from_millis(500),
            last_check: SystemTime::now(),
        };

        // Initial scan
        reloader.scan_scripts();

        reloader
    }

    pub fn check_for_changes(&mut self) -> Vec<String> {
        let now = SystemTime::now();

        // Only check periodically to avoid filesystem overhead
        if now
            .duration_since(self.last_check)
            .unwrap_or(Duration::ZERO)
            < self.check_interval
        {
            return Vec::new();
        }

        self.last_check = now;
        let mut reloaded = Vec::new();

        // Check each known script file
        for (path, last_mod_time) in self.file_timestamps.clone().iter() {
            if let Ok(metadata) = fs::metadata(path) {
                if let Ok(mod_time) = metadata.modified() {
                    // If file was modified since last check
                    if mod_time > *last_mod_time {
                        // Get relative path to convert to module name
                        if let Ok(rel_path) = path.strip_prefix(&self.scripts_dir) {
                            let module_name = Self::path_to_module_name(rel_path);

                            // Reload the script
                            if self.reload_script(path, &module_name) {
                                println!("Reloaded script: {}", module_name);
                                reloaded.push(module_name);

                                // Update timestamp
                                self.file_timestamps.insert(path.clone(), mod_time);
                            }
                        }
                    }
                }
            }
        }

        // Check for new files
        self.scan_scripts();

        reloaded
    }

    // Helper function to convert a path to a module name
    fn path_to_module_name(rel_path: &Path) -> String {
        let path_str = rel_path.to_string_lossy().replace("\\", "/");
        // Remove .lua extension if present
        let module_name = if path_str.ends_with(".lua") {
            &path_str[..path_str.len() - 4]
        } else {
            &path_str
        };

        module_name.to_string()
    }

    // Update scan_scripts to be recursive
    fn scan_scripts(&mut self) {
        if !self.scripts_dir.exists() {
            println!("Scripts directory does not exist: {:?}", self.scripts_dir);
            return;
        }

        // Use walkdir crate for recursive directory traversal
        for entry in walkdir::WalkDir::new(&self.scripts_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path().to_path_buf();

            // Only process .lua files
            if path.extension().map_or(false, |ext| ext == "lua") {
                // Record file's modification time
                if let Ok(metadata) = fs::metadata(&path) {
                    if let Ok(mod_time) = metadata.modified() {
                        self.file_timestamps.insert(path, mod_time);
                    }
                }
            }
        }
    }

    // Update reload_script to handle module paths
    fn reload_script(&self, path: &Path, module_name: &str) -> bool {
        if let Ok(_) = fs::read_to_string(path) {
            let lua_engine = self.lua.lock().unwrap();

            // Force Lua to reload the module
            let result = lua_engine
                .load(&format!(
                    r#"
            -- Remove from package.loaded to force reload
            package.loaded['{}'] = nil
            -- Re-require the module
            local status, result = pcall(require, '{}')
            if not status then
                print("Error reloading {}: " .. result)
                return false
            end
            return true
        "#,
                    module_name, module_name, module_name
                ))
                .exec();

            match result {
                Ok(()) => return true,
                Err(e) => println!("Error executing reload script: {:?}", e),
            }
        } else {
            println!("Could not read file: {}", path.display());
        }

        false
    }
}
