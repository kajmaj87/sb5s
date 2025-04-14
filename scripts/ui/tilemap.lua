-- Register the main terrain texture atlas
local ATLAS_WIDTH = 16
local TILE_SIZE = 16
local TILE_SCALE = 2
local TILESET_ATLAS = state("tileset_atlas",
        function()
            return ui.texture.register_atlas("assets/tileset.png", TILE_SIZE, TILE_SCALE, ATLAS_WIDTH)
        end)

local texture_ids = state("texture_ids",
        function()
            -- use Aseprite to find the ids, open the file and choose tileset mode then hover and use the magnifier icon value from bottom
            local textures = {}
            for i = 0, ATLAS_WIDTH * ATLAS_WIDTH - 1 do
                local texture, transparent = ui.texture.region(TILESET_ATLAS, i)
                print("Texture ID: " .. texture .. " is fully transparent: " .. tostring(transparent))
                if not transparent then
                    textures[i] = texture
                end
            end
            return textures;
        end)
print(texture_ids)
print("Total textures: " .. #texture_ids)
for i = 0, 15 do
    for j = 0, 15 do
        local texture = texture_ids[i + j * ATLAS_WIDTH]
        if texture then
            ui.tile.draw(texture, i, j)
        end
    end
end
--local terrain = state("terrain",
--        function()
--            return {
--                grass = api.terrain.register(1.0),
--                water = api.terrain.register(1000.0)
--            }
--        end)
--print("State after loading: ", STATE)
--print("Terrain Grass ID: " .. terrain.grass)
--print("Terrain Water ID: " .. terrain.water)
--api.terrain.set(terrain.water, -1, -1)
--api.terrain.set(terrain.grass, -1, -2)
--api.terrain.set(terrain.grass, -1, -3)
--api.projection.register("TerrainDrawing", function(event)
--    if event.type ~= "TerrainCreated" then
--        return
--    end
--    print(event)
--    print(ui.texture)
--    --- texture_id = get from
--    ui.tile.draw(texture_ids.grass, event.x, event.y)
--end)
--
