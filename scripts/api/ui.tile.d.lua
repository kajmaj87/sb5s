---@class ui.tile Tile-related functionality
ui.tile = {}

---Get the tile coordinates currently under the mouse cursor
---@return number x The x coordinate of the hovered tile
---@return number y The y coordinate of the hovered tile
function ui.tile.hovered()
end

---Get the tile ID at the specified coordinates
---@param x integer The x coordinate to check
---@param y integer The y coordinate to check
---@return integer|nil tileId The ID of the tile at the specified position, or nil if no tile exists
function ui.tile.at(x, y)
end