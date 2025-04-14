---@class api.terrain Terrain type management functionality
api.terrain = {}

---Register a terrain type
---@param movement_cost number The movement cost associated with this terrain type
---@return number terrain_type_id The ID assigned to this terrain type
function api.terrain.register(movement_cost)
end

---Set a specific terrain type at location
---@param terrain_type_id number The ID of the terrain type to set
---@param x number The x-coordinate of the tile
---@param y number The y-coordinate of the tile
---@return number terrain_id The ID of the terrain set
function api.terrain.set(terrain_type_id, x, y)
end