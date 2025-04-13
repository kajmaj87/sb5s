----@class ui.atlas Texture atlas management functionality
ui.atlas = {}

---Register a texture atlas from an image file
---@param path string Path to the texture atlas image file
---@param tile_size number Size of each tile in pixels
---@param width_in_tiles number Number of tiles horizontally in the atlas
---@return number atlas_id The ID of the registered atlas
function ui.atlas.register(path, tile_size, width_in_tiles)
end

---Register a texture region within an atlas
---@param atlas_id number The ID of the atlas to register this region in
---@param name string Unique identifier for this texture
---@param linear_index number Linear index of the tile in the atlas (0-based)
---                           For example, with 16 tiles per row, (0,0) is 0, (5,0) is 5, and (0,1) is 16
---@return number texture_id The ID assigned to this texture
function ui.atlas.region(atlas_id, name, linear_index)
end

---Get the ID of a previously registered texture by name
---@param name string The name of the texture to look up
---@return number|nil texture_id The ID of the texture, or nil if not found
function ui.atlas.get_texture_id(name)
end