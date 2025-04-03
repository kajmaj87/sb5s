use crate::hot_reload::SimpleHotReloader;
use logic::domain::event::person_event::PersonEvent;
use logic::domain::event::*;
use logic::{CoreApi, Projection};
use mlua::{Function, IntoLuaMulti, Lua, MultiValue, Table, Value};
use std::path::Path;
use std::sync::{mpsc, Arc, Mutex, RwLock};

// Commands that can be sent to the Lua worker
pub enum LuaCommand {
    Execute {
        code: String,
        response_tx: mpsc::Sender<Result<String, String>>,
    },
    HotReload,
    Shutdown,
}

// Response from LuaEngine
pub enum LuaResponse {
    // Add response types if needed
}

pub struct LuaEngine {
    pub lua: Lua,
    command_rx: mpsc::Receiver<LuaCommand>,
    hot_reload: SimpleHotReloader,
}

pub struct LuaProjection {
    handler: Function,
    name: String,
}

pub struct LuaEvent {
    event: DomainEvent,
}

impl IntoLuaMulti for LuaEvent {
    fn into_lua_multi(self, lua: &Lua) -> mlua::Result<MultiValue> {
        match self.event {
            DomainEvent::Person(person_event) => {
                let table = lua.create_table()?;

                // Common field for all Person events
                table.set("type", "Person")?;

                // Handle different PersonEvent variants
                match person_event {
                    PersonEvent::PersonCreated {
                        person_id,
                        name,
                        location,
                    } => {
                        table.set("event_type", "PersonCreated")?;
                        table.set("person_id", person_id.0)?; // Assuming PersonId has a field '0'
                        table.set("name", name)?;
                        table.set("x", location.x)?;
                        table.set("y", location.y)?;
                    }
                    PersonEvent::PersonMoved {
                        person_id,
                        from_location,
                        to_location,
                    } => {
                        table.set("event_type", "PersonMoved")?;
                        table.set("person_id", person_id.0)?;

                        // Create nested tables for from/to locations
                        let from = lua.create_table()?;
                        from.set("x", from_location.x)?;
                        from.set("y", from_location.y)?;
                        table.set("from_location", from)?;

                        let to = lua.create_table()?;
                        to.set("x", to_location.x)?;
                        to.set("y", to_location.y)?;
                        table.set("to_location", to)?;
                    } // Add other PersonEvent variants here as needed
                }
                let table_value = Value::Table(table);
                let mut values = MultiValue::new();
                values.push_front(table_value);

                Ok(values)
            } // Add other DomainEvent variants here as needed
        }
    }
}

impl Projection for LuaProjection {
    fn apply(&mut self, event: &DomainEvent) {
        if let Err(e) = self.handler.call::<()>(LuaEvent {
            event: event.clone(),
        }) {
            println!(
                "Error calling Lua handler inside projection {}: {:?}",
                self.name, e
            );
        }
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl LuaEngine {
    // Creates a new LuaEngine that receives commands from a channel
    pub fn new(command_rx: mpsc::Receiver<LuaCommand>) -> Self {
        let lua = Lua::new();
        if let Err(e) = lua
            .load(
                r#"-- Add scripts directory to Lua's package path
        package.path = "./scripts/?.lua;" .. package.path
        require('bootstrap')"#,
            )
            .exec()
        {
            println!("Error during lua bootstrap: {:?}", e);
        }
        if let Err(e) = Self::init_help_system(&lua) {
            println!("Error during lua help system initialization: {:?}", e);
        }
        let globals = lua.globals();
        let hot_reload =
            SimpleHotReloader::new(Arc::new(Mutex::new(lua.clone())), Path::new("scripts"));

        // Initialize core API
        let core = Arc::new(RwLock::new(CoreApi::new()));

        // Create API tables
        let person_table = lua.create_table().unwrap();
        let location_table = lua.create_table().unwrap();
        let event_table = lua.create_table().unwrap();
        let projection_table = lua.create_table().unwrap();

        // Setup the APIs
        Self::setup_person_api(&lua, &person_table, Arc::clone(&core));
        Self::setup_location_api(&lua, &location_table, Arc::clone(&core));
        Self::setup_event_api(&lua, &event_table, Arc::clone(&core));
        Self::setup_projection_api(&lua, &projection_table, Arc::clone(&core));

        // Create main API table
        let api_table = lua.create_table().unwrap();
        api_table.set("person", person_table).unwrap();
        api_table.set("location", location_table).unwrap();
        api_table.set("event", event_table).unwrap();
        api_table.set("projection", projection_table).unwrap();

        // Set API as global
        globals.set("api", api_table).unwrap();

        Self {
            lua,
            command_rx,
            hot_reload,
        }
    }

    pub fn run_script(&mut self, script: &str) -> mlua::Result<()> {
        self.lua.load(script).exec()
    }

    pub fn hot_reload(&mut self) {
        self.hot_reload.check_for_changes();
    }

    // Process a single command - call this in a loop from your thread
    pub fn process_command(&mut self) -> bool {
        self.hot_reload();
        match self.command_rx.recv() {
            Ok(cmd) => {
                match cmd {
                    LuaCommand::Execute { code, response_tx } => {
                        let result = match self.lua.load(&code).eval::<Value>() {
                            Ok(value) => {
                                // Convert Lua value to string representation
                                let result = match value {
                                    Value::Nil => "nil".to_string(),
                                    Value::Boolean(b) => b.to_string(),
                                    Value::Integer(i) => i.to_string(),
                                    Value::Number(n) => n.to_string(),
                                    Value::String(s) => s.to_str().unwrap().to_string(),
                                    Value::Table(_) => "table".to_string(),
                                    Value::Function(_) => "[function]".to_string(),
                                    _ => "[value]".to_string(),
                                };
                                Ok(result)
                            }
                            Err(e) => Err(e.to_string()),
                        };
                        let _ = response_tx.send(result);
                    }
                    LuaCommand::Shutdown => return false,
                    LuaCommand::HotReload => {
                        self.hot_reload();
                    }
                }
                true
            }
            Err(_) => false, // Channel closed
        }
    }
    pub fn run(&mut self) {
        while self.process_command() {}
    }
    fn setup_person_api(lua: &Lua, table: &Table, core: Arc<RwLock<CoreApi>>) {
        // Expose api.person.create to Lua
        let core_clone = Arc::clone(&core);
        let create_person = lua
            .create_function(move |lua_ctx, (name, x, y): (String, i32, i32)| {
                match core_clone.read().unwrap().person().create(name, x, y) {
                    Ok(person) => {
                        // Convert Person to Lua table using the provided lua context
                        let person_table = lua_ctx.create_table()?;
                        person_table.set("id", person.id.0)?;
                        person_table.set("name", person.name)?;

                        let location_table = lua_ctx.create_table()?;
                        location_table.set("x", person.location.x)?;
                        location_table.set("y", person.location.y)?;

                        person_table.set("location", location_table)?;
                        Ok(person_table)
                    }
                    Err(e) => Err(mlua::Error::RuntimeError(e)),
                }
            })
            .unwrap();
        table.set("create", create_person).unwrap();

        // Expose api.person.move_to to Lua
        let core_clone = Arc::clone(&core);
        let move_person = lua
            .create_function(move |lua_ctx, (id, x, y): (u32, i32, i32)| {
                match core_clone.read().unwrap().person().move_to(id, x, y) {
                    Ok(person) => {
                        // Convert Person to Lua table using the provided lua context
                        let person_table = lua_ctx.create_table()?;
                        person_table.set("id", person.id.0)?;
                        person_table.set("name", person.name)?;

                        let location_table = lua_ctx.create_table()?;
                        location_table.set("x", person.location.x)?;
                        location_table.set("y", person.location.y)?;

                        person_table.set("location", location_table)?;
                        Ok(person_table)
                    }
                    Err(e) => Err(mlua::Error::RuntimeError(e)),
                }
            })
            .unwrap();
        table.set("move_to", move_person).unwrap();

        // Expose api.person.get to Lua
        let core_clone = Arc::clone(&core);
        let get_person = lua
            .create_function(move |lua_ctx, id: u32| {
                match core_clone.read().unwrap().person().get(id) {
                    Ok(person) => {
                        // Convert Person to Lua table using the provided lua context
                        let person_table = lua_ctx.create_table()?;
                        person_table.set("id", person.id.0)?;
                        person_table.set("name", person.name)?;

                        let location_table = lua_ctx.create_table()?;
                        location_table.set("x", person.location.x)?;
                        location_table.set("y", person.location.y)?;

                        person_table.set("location", location_table)?;
                        Ok(person_table)
                    }
                    Err(e) => Err(mlua::Error::RuntimeError(e)),
                }
            })
            .unwrap();
        table.set("get", get_person).unwrap();

        // Expose api.person.get_all to Lua
        let core_clone = Arc::clone(&core);
        let get_all_persons = lua
            .create_function(move |lua_ctx, ()| {
                match core_clone.read().unwrap().person().get_all() {
                    Ok(persons) => {
                        // Convert Vec<Person> to Lua table using the provided lua context
                        let persons_table = lua_ctx.create_table()?;

                        for (i, person) in persons.iter().enumerate() {
                            let person_table = lua_ctx.create_table()?;
                            person_table.set("id", person.id.0)?;
                            person_table.set("name", person.name.clone())?;

                            let location_table = lua_ctx.create_table()?;
                            location_table.set("x", person.location.x)?;
                            location_table.set("y", person.location.y)?;

                            person_table.set("location", location_table)?;
                            persons_table.set(i + 1, person_table)?;
                        }

                        Ok(persons_table)
                    }
                    Err(e) => Err(mlua::Error::RuntimeError(e)),
                }
            })
            .unwrap();
        table.set("get_all", get_all_persons).unwrap();
    }

    fn setup_location_api(lua: &Lua, table: &Table, core: Arc<RwLock<CoreApi>>) {
        // Expose api.location.get_people_at to Lua
        let core_clone = Arc::clone(&core);
        let get_people_at = lua
            .create_function(move |lua_ctx, (x, y): (i32, i32)| {
                let people_ids = core_clone.read().unwrap().location().get_people_at(x, y);

                // Convert Vec<u32> to Lua table using the provided lua context
                let people_table = lua_ctx.create_table()?;

                for (i, id) in people_ids.iter().enumerate() {
                    people_table.set(i + 1, *id)?;
                }

                Ok(people_table)
            })
            .unwrap();
        table.set("get_people_at", get_people_at).unwrap();

        // Expose api.location.get_occupied to Lua
        let core_clone = Arc::clone(&core);
        let get_occupied = lua
            .create_function(move |lua_ctx, ()| {
                let locations = core_clone.read().unwrap().location().get_occupied();

                // Convert Vec<(i32, i32)> to Lua table using the provided lua context
                let locations_table = lua_ctx.create_table()?;

                for (i, (x, y)) in locations.iter().enumerate() {
                    let location_table = lua_ctx.create_table()?;
                    location_table.set("x", *x)?;
                    location_table.set("y", *y)?;

                    locations_table.set(i + 1, location_table)?;
                }

                Ok(locations_table)
            })
            .unwrap();
        table.set("get_occupied", get_occupied).unwrap();

        // Expose api.location.most_crowded to Lua
        let core_clone = Arc::clone(&core);
        let most_crowded = lua
            .create_function(move |_, ()| {
                if let Some((x, y, count)) = core_clone.read().unwrap().location().most_crowded() {
                    Ok((Some(x), Some(y), Some(count)))
                } else {
                    Ok((None, None, None))
                }
            })
            .unwrap();
        table.set("most_crowded", most_crowded).unwrap();

        // Expose api.location.occupied_count to Lua
        let core_clone = Arc::clone(&core);
        let occupied_count = lua
            .create_function(move |_, ()| {
                let count = core_clone.read().unwrap().location().occupied_count();
                Ok(count)
            })
            .unwrap();
        table.set("occupied_count", occupied_count).unwrap();
    }

    fn setup_event_api(lua: &Lua, table: &Table, core: Arc<RwLock<CoreApi>>) {
        // Expose api.event.count to Lua
        let core_clone = Arc::clone(&core);
        let event_count = lua
            .create_function(move |_, ()| {
                let count = core_clone.read().unwrap().event().count();
                Ok(count)
            })
            .unwrap();
        table.set("count", event_count).unwrap();
    }

    fn setup_projection_api(lua: &Lua, table: &Table, core: Arc<RwLock<CoreApi>>) {
        let core_clone = Arc::clone(&core);
        let register_projection = lua
            .create_function(move |_, (name, handler): (String, Function)| {
                let projection = LuaProjection {
                    handler,
                    name: name.clone(),
                };
                core_clone
                    .write()
                    .unwrap()
                    .projection()
                    .register_projection(projection);
                Ok(())
            })
            .unwrap();
        table.set("register", register_projection).unwrap();
    }

    fn init_help_system(lua: &Lua) -> mlua::Result<()> {
        let help_script = std::fs::read_to_string("scripts/help.lua")?;
        lua.load(&help_script).exec()?;

        // Automatically find and load all .d.lua files from the scripts/api directory
        let mut doc_files = std::collections::HashMap::new();

        match std::fs::read_dir("scripts/api") {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file()
                        && path.extension().is_some_and(|ext| ext == "lua")
                        && path
                            .file_name()
                            .is_some_and(|name| name.to_string_lossy().contains(".d.lua"))
                    {
                        // Read the file content
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            // Get filename without extension as the key
                            if let Some(filename) = path.file_stem() {
                                let key = filename.to_string_lossy().to_string();
                                // Remove the ".d" suffix if present
                                let key = key.strip_suffix(".d").unwrap_or(&key).to_string();
                                doc_files.insert(key, content);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("Warning: Could not read scripts/api directory: {}", e);
            }
        }

        if doc_files.is_empty() {
            println!("Warning: No .d.lua files found in scripts/api directory");
        }

        doc_files.iter().for_each(|(key, _content)| {
            println!("Loaded help doc: {}", key);
        });

        // Pass file contents to Lua
        let globals = lua.globals();
        let init_docs: Function = globals.get("init_help_docs")?;
        let result = init_docs.call::<bool>(doc_files)?;
        println!("init_help_docs result: {}", result);
        Ok(())
    }
}
