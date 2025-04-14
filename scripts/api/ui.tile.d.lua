---@class ui.tile Tile-related functionality
ui.tile = {}

---Get the tile coordinates currently under the mouse cursor
---@return number x The x coordinate of the hovered tile
---@return number y The y coordinate of the hovered tile
function ui.tile.hovered()
end

---Get the tile ID at the specified coordinates
---@param x number The x coordinate to check
---@param y number  The y coordinate to check
---@return number integer|nil tileId The ID of the tile at the specified position, or nil if no tile exists
function ui.tile.at(x, y)
end
---Get the tile ID at the specified screen coordinates
---@param x number The x screen coordinate to check
---@param y number  The y screen coordinate to check
---@return number integer|nil tileId The ID of the tile at the specified position, or nil if no tile exists
function ui.tile.at_screen_pos(x, y)
end

---Convert tile coordinates to world coordinates. This does not take camera position into account
---@param x number The x coordinate of the tile
---@param y number The y coordinate of the tile
---@return number worldX The x world coordinate corresponding to the tile
---@return number worldY The y world coordinate corresponding to the tile
function ui.tile.to_world_pos(x, y)
end

---Draw a texture as map tile on grid coordinates
---@param texture_id number The ID of the texture to draw
---@param x number The x-coordinate to draw at (in grid coordinates)
---@param y number The y-coordinate to draw at (in grid coordinates)
---@return nil
---@error string Error message if texture not found or other drawing error
function ui.tile.draw(texture_id, x, y) end
