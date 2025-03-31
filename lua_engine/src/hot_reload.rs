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

    // Scan and record all script files
    fn scan_scripts(&mut self) {
        if !self.scripts_dir.exists() {
            println!("Scripts directory does not exist: {:?}", self.scripts_dir);
            return;
        }

        if let Ok(entries) = fs::read_dir(&self.scripts_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
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
    }

    // Check for changes and reload modified scripts
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
                        if let Some(script_name) = path.file_stem().and_then(|s| s.to_str()) {
                            // Reload the script
                            if self.reload_script(path, script_name) {
                                println!("Reloaded script: {}", script_name);
                                reloaded.push(script_name.to_string());

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

    // Reload a single script file
    fn reload_script(&self, path: &Path, script_name: &str) -> bool {
        if let Ok(code) = fs::read_to_string(path) {
            let lua_lock = self.lua.lock().unwrap();

            // Execute the script and store its result
            if let Err(err) = lua_lock
                .load(&code)
                .set_name(script_name)
                .eval::<mlua::Value>()
            {
                println!("Error reloading {}: {}", script_name, err);
                return false;
            }

            return true;
        }

        false
    }
}
