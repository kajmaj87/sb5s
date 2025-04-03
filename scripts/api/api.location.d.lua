---@class api.location Location-based API for querying position information
api.location = {}

---@param x integer X coordinate
---@param y integer Y coordinate
---@return integer[] person_ids List of person IDs at the location
--- Get all people at a specific location
function api.location.get_people_at(x, y)
end

---@return table[] locations Array of location pairs (x,y)
--- Get all occupied locations
function api.location.get_occupied()
end

---@return table|nil location Table containing (x, y, count) for the most crowded location, or nil if no locations are occupied
--- Get the most crowded location
function api.location.most_crowded()
end

---@return integer count
--- Get the number of occupied locations
function api.location.occupied_count()
end
