-- Register the main terrain texture atlas
local ATLAS_ID = ui.atlas.register(
        "assets/tileset.png",
        16,
        16
)

print("Atlas ID: " .. ATLAS_ID)

--local texture_ids = {}
---- use Aseprite to find the ids, open the file and choose tileset mode then hover and use the magnifier icon value from bottom
--texture_ids.grass = ui.atlas.region(ATLAS_ID, "grass", 17)
--texture_ids.grass_water_s = ui.atlas.region(ATLAS_ID, "grass_water_s", 68) -- grass tile with water to the south
--texture_ids.water = ui.atlas.region(ATLAS_ID, "water", 81)
--
---- Register terrain types with optional properties
--local TERRAIN_GRASS = ui.terrain.register("grass", {
--    movement_cost = 1.0,
--    buildable = true
--})
--
--local TERRAIN_WATER = ui.terrain.register("water", {
--    movement_cost = 1000.0,
--    buildable = false
--})
--
---- Define terrain connections for various configurations
---- Connect grass with water to the north
--ui.terrain.connection(
--        TERRAIN_GRASS,
---- n, e, s, w
--        { TERRAIN_GRASS, TERRAIN_GRASS, TERRAIN_WATER, TERRAIN_GRASS },
--        texture_ids.grass_water_s
--)
--
---- Export texture IDs for use in other scripts
--return {
--    atlas_id = ATLAS_ID,
--    textures = texture_ids,
--    terrain = {
--        GRASS = TERRAIN_GRASS,
--        WATER = TERRAIN_WATER
--    }
--}
