----@class ui.texture Texture management functionality
ui.texture = {}

---Register a texture atlas from an image file
---@param path string Path to the texture atlas image file
---@param tile_size number Size of each tile in pixels
---@param scale number Scale factor for the texture (1.0 = 100%)
---@param width_in_tiles number Number of tiles horizontally in the atlas
---@return number atlas_id The ID of the registered atlas
function ui.texture.register_atlas(path, tile_size, scale, width_in_tiles)
end

---Register a texture region within an atlas
---@param atlas_id number The ID of the atlas to register this region in
---@param linear_index number Linear index of the tile in the atlas (0-based)
---                           For example, with 16 tiles per row, (0,0) is 0, (5,0) is 5, and (0,1) is 16
---@return number texture_id The ID assigned to this texture
---@return boolean is_fully_transparent Whether the texture is fully transparent
function ui.texture.region(atlas_id, linear_index)
end
