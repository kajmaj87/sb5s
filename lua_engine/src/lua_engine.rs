use core::CoreApi;
use mlua::{Lua, Result as LuaResult, Table};
use std::sync::{Arc, RwLock};

pub struct LuaEngine {
    pub lua: Lua,
}

impl LuaEngine {
    pub fn new(core: Arc<RwLock<CoreApi>>) -> Self {
        let lua = Lua::new();
        let globals = lua.globals();

        // Create the API tables
        let person_table = lua.create_table().unwrap();
        let location_table = lua.create_table().unwrap();
        let event_table = lua.create_table().unwrap();

        // Set up the person API
        Self::setup_person_api(&lua, &person_table, Arc::clone(&core));

        // Set up the location API
        Self::setup_location_api(&lua, &location_table, Arc::clone(&core));

        // Set up the event API
        Self::setup_event_api(&lua, &event_table, Arc::clone(&core));

        // Create the main API table
        let api_table = lua.create_table().unwrap();
        api_table.set("person", person_table).unwrap();
        api_table.set("location", location_table).unwrap();
        api_table.set("event", event_table).unwrap();

        // Set the API table as a global
        globals.set("api", api_table).unwrap();

        Self { lua }
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

    pub fn run_script(&self, script: &str) -> LuaResult<()> {
        self.lua.load(script).exec()
    }
}
