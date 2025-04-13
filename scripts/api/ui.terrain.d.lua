---@class ui.terrain Terrain type management functionality
ui.terrain = {}

---Register a terrain type
---@param name string Unique identifier for this terrain type (e.g., "grass", "water")
---@param properties? table Optional table with terrain properties like movement_cost, buildable, etc.
---@return number terrain_id The ID assigned to this terrain type
function ui.terrain.register(name, properties)
end

---Define a connection between terrain types and the texture to use
---@param terrain_id number The ID of the primary terrain type
---@param surroundings table Array with 4 terrain IDs in [west, east, north, south] order
---@param texture_id number The ID of the texture to use for this terrain configuration
---@return boolean success Whether the connection was successfully registered
function ui.terrain.connection(terrain_id, surroundings, texture_id)
end

---Register a terrain feature (like roads, forests, etc.)
---@param name string Unique identifier for this feature (e.g., "road", "forest")
---@param properties? table Optional table with feature properties like movement_modifier, etc.
---@return number feature_id The ID assigned to this feature
function ui.terrain.feature(name, properties)
end

---Define a connection between feature types and the texture to use
---@param feature_id number The ID of the primary feature
---@param surroundings table Array with 4 feature IDs in [west, east, north, south] order
---@param texture_id number The ID of the texture to use for this feature configuration
---@return boolean success Whether the connection was successfully registered
function ui.terrain.feature_connection(feature_id, surroundings, texture_id)
end

---Get the ID of a previously registered terrain type by name
---@param name string The name of the terrain type to look up
---@return number|nil terrain_id The ID of the terrain type, or nil if not found
function ui.terrain.get_id(name)
end

---Get the ID of a previously registered feature by name
---@param name string The name of the feature to look up
---@return number|nil feature_id The ID of the feature, or nil if not found
function ui.terrain.get_feature_id(name)
end