---@class api.person
--- Person entity management API
api.person = {}

---@param name string The name of the person
---@param x integer X coordinate for initial position
---@param y integer Y coordinate for initial position
---@return table|nil person The created person object if successful
---@return string|nil error Error message if the operation failed
--- Create a new person at the specified location
function api.person.create(name, x, y)
end

---@param person_id integer ID of the person to move
---@param x integer X coordinate of the new location
---@param y integer Y coordinate of the new location
---@return table|nil person The updated person object if successful
---@return string|nil error Error message if the operation failed
--- Move a person to a new location
function api.person.move_to(person_id, x, y)
end

---@param person_id integer ID of the person to retrieve
---@return table|nil person The person object if found
---@return string|nil error Error message if the operation failed
--- Get a person by ID
function api.person.get(person_id)
end

---@return table[]|nil persons Array of person objects if successful
---@return string|nil error Error message if the operation failed
--- Get all persons
function api.person.get_all()
end
