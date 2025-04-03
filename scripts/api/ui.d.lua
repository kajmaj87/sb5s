---@class ui UI namespace for Rust engine bindings
---@field tile ui.tile Tile-related functionality
---@field input ui.input Input handling functionality
ui = {}

---Create a label UI component at the specified position
---@param x number X coordinate on screen
---@param y number Y coordinate on screen
---@param handler function Function that returns the label text string
---@return nil
function ui.label(x, y, handler)
end

---Get the current frames per second
---@return number fps Current FPS value
function ui.fps()
end