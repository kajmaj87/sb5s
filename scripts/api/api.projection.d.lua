---@class api.projection
api.projection = {}

---@param name string The name of the projection
---@param handler function The function to handle the projection
--- Register the projection. The handler passed to this function will be called
--- after each event being sent by the engine. The handler will receive the event as
--- its first argument
function api.projection.register(name, handler)
end
