use crate::docs;
use logic::CoreApi;
use mlua::{Function, Lua, Result as LuaResult, Table, Value};
use std::collections::HashMap;
use std::sync::{mpsc, Arc, RwLock};

// Commands that can be sent to the Lua worker
#[derive(Debug)]
pub enum LuaCommand {
    RunScript {
        script: String,
        response_tx: mpsc::Sender<Result<(), String>>,
    },
    Execute {
        code: String,
        response_tx: mpsc::Sender<Result<String, String>>,
    },
    RegisterCallback {
        script: String,
        response_tx: mpsc::Sender<u32>,
    },
    ExecuteCallback {
        id: u32,
        response_tx: mpsc::Sender<Result<(), String>>,
    },
    Shutdown,
}

// Response from LuaEngine
pub enum LuaResponse {
    // Add response types if needed
}

pub struct LuaEngine {
    lua: Lua,
    callbacks: HashMap<u32, Function>,
    next_callback_id: u32,
    command_rx: mpsc::Receiver<LuaCommand>,
}

impl LuaEngine {
    // Creates a new LuaEngine that receives commands from a channel
    pub fn new(command_rx: mpsc::Receiver<LuaCommand>) -> Self {
        let lua = Lua::new();
        let globals = lua.globals();

        // Initialize core API
        let core = Arc::new(RwLock::new(CoreApi::new()));

        // Create API tables
        let person_table = lua.create_table().unwrap();
        let location_table = lua.create_table().unwrap();
        let event_table = lua.create_table().unwrap();

        // Setup the APIs
        Self::setup_person_api(&lua, &person_table, Arc::clone(&core));
        Self::setup_location_api(&lua, &location_table, Arc::clone(&core));
        Self::setup_event_api(&lua, &event_table, Arc::clone(&core));

        // Create main API table
        let api_table = lua.create_table().unwrap();
        api_table.set("person", person_table).unwrap();
        api_table.set("location", location_table).unwrap();
        api_table.set("event", event_table).unwrap();

        // Set API as global
        globals.set("api", api_table).unwrap();

        // Setup documentation
        Self::setup_documentation(&lua);

        Self {
            lua,
            callbacks: HashMap::new(),
            next_callback_id: 1,
            command_rx,
        }
    }

    // Process a single command - call this in a loop from your thread
    pub fn process_command(&mut self) -> bool {
        match self.command_rx.recv() {
            Ok(cmd) => {
                match cmd {
                    LuaCommand::RunScript {
                        script,
                        response_tx,
                    } => {
                        let result = match self.lua.load(&script).exec() {
                            Ok(()) => Ok(()),
                            Err(e) => Err(e.to_string()), // Convert LuaError to String
                        };
                        let _ = response_tx.send(result);
                    }
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
                    // LuaCommand::RegisterCallback { script, name, response_tx } => {
                    //     // Load the function from string
                    //     let func = match self.load(&script).eval::<Function>() {
                    //         Ok(f) => f,
                    //         Err(e) => {
                    //             let _ = response_tx.send(0); // Error code
                    //         }
                    //     };
                    //
                    //     // Store in a HashMap with ID
                    //     let id = next_callback_id;
                    //     next_callback_id += 1;
                    //     callbacks.insert(id, (name, func));
                    //
                    //     let _ = response_tx.send(id);
                    // }
                    LuaCommand::ExecuteCallback { id, response_tx } => {
                        let result = if let Some(callback) = self.callbacks.get(&id) {
                            match callback.call(()) {
                                Ok(()) => Ok(()),
                                Err(e) => Err(e.to_string()), // Convert LuaError to String
                            }
                        } else {
                            Err(format!("Callback with id {} not found", id))
                        };
                        let _ = response_tx.send(result);
                    }
                    LuaCommand::Shutdown => return false,
                    _ => {}
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
            .create_function(move |lua_ctx, ()| {
                if let Some((x, y, count)) = core_clone.read().unwrap().location().most_crowded() {
                    // Convert to Lua table using the provided lua context
                    let result_table = lua_ctx.create_table()?;
                    result_table.set("x", x)?;
                    result_table.set("y", y)?;
                    result_table.set("count", count)?;

                    Ok(Some(result_table))
                } else {
                    Ok(None)
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

    fn setup_documentation(lua: &Lua) {
        // Create the docs table
        let docs_table = lua.create_table().unwrap();
        let globals = lua.globals();
        globals.set("docs", docs_table.clone()).unwrap();

        // Get the API documentation from the generated code
        let api_docs = docs::get_api_docs();

        // Convert API docs to Lua tables
        for (module_name, module_docs) in api_docs {
            let module_table = lua.create_table().unwrap();
            docs_table
                .set(module_name.clone(), module_table.clone())
                .unwrap();

            for (method_name, method_doc) in module_docs.methods {
                let method_table = lua.create_table().unwrap();
                method_table
                    .set("description", method_doc.description)
                    .unwrap();

                // Set parameters
                let params_table = lua.create_table().unwrap();
                for (i, param) in method_doc.params.iter().enumerate() {
                    let param_table = lua.create_table().unwrap();
                    param_table.set("name", param.name.clone()).unwrap();
                    param_table.set("type", param.type_name.clone()).unwrap();
                    param_table
                        .set("description", param.description.clone())
                        .unwrap();
                    params_table.set(i + 1, &param_table).unwrap();
                    params_table.set(param.name.clone(), param_table).unwrap();
                }

                method_table.set("params", params_table).unwrap();
                method_table.set("returns", method_doc.returns).unwrap();

                module_table.set(method_name, method_table).unwrap();
            }
        }

        // Add help function
        let help_fn = lua.create_function(|ctx, topic: Option<String>| {
            let docs: Table = ctx.globals().get("docs")?;

            match topic {
                None => {
                    // Level 1: List all modules
                    let mut result = String::from("Available modules:\n");

                    for pair in docs.pairs::<String, Table>() {
                        let (module, _) = pair?;
                        result.push_str(&format!("  {}\n", module));
                    }

                    result.push_str("\nUse help(\"module\") to see available methods.");
                    Ok(result)
                }
                Some(topic) => {
                    // Check if this is a module name or a method name
                    let parts: Vec<&str> = topic.split('.').collect();

                    if parts.len() == 1 {
                        // Level 2: List all methods in a module
                        let module = parts[0];
                        let module_docs: LuaResult<Table> = docs.get(module);

                        if let Ok(module_table) = module_docs {
                            let mut result = format!("Methods in {} module:\n", module);

                            for pair in module_table.pairs::<String, Table>() {
                                let (method, _) = pair?;
                                result.push_str(&format!("  {}.{}\n", module, method));
                            }

                            result.push_str("\nUse help(\"module.method\") to see method details.");
                            Ok(result)
                        } else {
                            Ok(format!("Module '{}' not found. Use help() to see available modules.", module))
                        }
                    } else if parts.len() == 2 {
                        // Level 3: Show details of a specific method
                        let module = parts[0];
                        let method = parts[1];

                        // Get the module table
                        let module_docs: LuaResult<Table> = docs.get(module);
                        if let Ok(module_table) = module_docs {
                            // Get the method documentation
                            let method_docs: LuaResult<Table> = module_table.get(method);
                            if let Ok(doc) = method_docs {
                                // Format and return documentation
                                let desc: String = doc.get("description")?;
                                let params: Table = doc.get("params")?;
                                let returns: String = doc.get("returns")?;

                                let mut result = format!("--- {}\n\n", desc);
                                result.push_str("Parameters:\n");

                                // List parameters
                                let param_count: i32 = params.len()?;
                                for i in 1..=param_count {
                                    let param: Table = params.get(i)?;
                                    let name: String = param.get("name")?;
                                    let type_name: String = param.get("type")?;
                                    let param_desc: String = param.get("description").unwrap_or_default();

                                    result.push_str(&format!("  {} ({})", name, type_name));
                                    if !param_desc.is_empty() {
                                        result.push_str(&format!(" - {}", param_desc));
                                    }
                                    result.push('\n');
                                }

                                result.push_str(&format!("\nReturns: {}", returns));
                                Ok(result)
                            } else {
                                Ok(format!("Method '{}.{}' not found. Use help('{}') to see available methods.",
                                           module, method, module))
                            }
                        } else {
                            Ok(format!("Module '{}' not found. Use help() to see available modules.", module))
                        }
                    } else {
                        Ok(format!("Invalid topic format: '{}'. Use help(), help(\"module\"), or help(\"module.method\").", topic))
                    }
                }
            }
        }).unwrap();

        globals.set("help", help_fn).unwrap();
    }
}
