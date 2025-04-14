use macroquad::prelude::*;
use std::collections::HashMap;
use ::utils::repo::NumericId;
use utils_derive::NumericId;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, NumericId)]
pub struct TextureId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, NumericId)]
pub struct AtlasId(pub u32);

pub struct TextureAtlas {
    pub texture: Texture2D,
    pub tile_size: f32,
    pub scale: f32,
    pub width_in_tiles: u32,
    regions: HashMap<TextureId, (f32, f32)>, // Maps texture_id -> position in atlas
}

impl TextureAtlas {
    pub fn new(texture: Texture2D, tile_size: f32, scale: f32, width_in_tiles: u32) -> Self {
        Self {
            texture,
            tile_size,
            scale,
            width_in_tiles,
            regions: HashMap::new(),
        }
    }

    fn register_region(&mut self, texture_id: TextureId, linear_index: u32) {
        let col = linear_index % self.width_in_tiles;
        let row = linear_index / self.width_in_tiles;

        let src_x = col as f32 * self.tile_size;
        let src_y = row as f32 * self.tile_size;
        self.regions.insert(texture_id, (src_x, src_y));
    }
    fn is_transparent(&self, texture_id: &TextureId) -> bool {
        if let Some((src_x, src_y)) = self.regions.get(&texture_id) {
            // Get image data from texture
            let image_data = self.texture.get_texture_data();

            // Convert to pixel coordinates
            let x = *src_x as u32;
            let y = *src_y as u32;
            let width = self.tile_size as u32;
            let height = self.tile_size as u32;

            // Check each pixel in the region
            for py in y..y + height {
                for px in x..x + width {
                    let pixel = image_data.get_pixel(px, py);
                    // If any pixel has non-zero alpha, the region is not fully transparent
                    if pixel.a > 0 as f32 {
                        return false;
                    }
                }
            }

            // All pixels were transparent
            return true;
        }
        // Texture ID not found, consider it transparent
        true
    }

    pub fn draw(&self, texture_id: &TextureId, screen_x: f32, screen_y: f32, color: Color) {
        if let Some((src_x, src_y)) = self.regions.get(&texture_id) {
            draw_texture_ex(
                &self.texture,
                screen_x,
                screen_y,
                color,
                DrawTextureParams {
                    source: Some(Rect::new(*src_x, *src_y, self.tile_size, self.tile_size)),
                    dest_size: Some(vec2(
                        self.tile_size * self.scale,
                        self.tile_size * self.scale,
                    )),
                    ..Default::default()
                },
            );
        }
    }
}
pub struct TextureAtlasManager {
    atlases: HashMap<AtlasId, TextureAtlas>,
    texture_atlas_map: HashMap<TextureId, AtlasId>,
    next_atlas_id: AtlasId,
    next_texture_id: TextureId,
}

impl TextureAtlasManager {
    pub fn new() -> Self {
        Self {
            atlases: HashMap::new(),
            texture_atlas_map: HashMap::new(),
            next_atlas_id: AtlasId(0),
            next_texture_id: TextureId(0),
        }
    }

    pub async fn create_atlas(
        &mut self,
        path: &str,
        tile_size: f32,
        scale: f32,
        width_in_tiles: u32,
    ) -> Result<AtlasId, String> {
        let texture = (match load_texture(path).await {
            Ok(texture) => Ok(texture),
            Err(e) => Err(format!("Failed to load texture: {}", e)),
        })?;
        texture.set_filter(FilterMode::Nearest);

        // Create atlas
        let atlas = TextureAtlas::new(texture, tile_size, scale, width_in_tiles);
        let id = self.next_atlas_id;
        self.next_atlas_id = self.next_atlas_id.next();

        // Store atlas
        self.atlases.insert(id, atlas);

        Ok(id)
    }
    pub fn register_texture_in_atlas(
        &mut self,
        atlas_id: AtlasId,
        linear_index: u32,
    ) -> Result<(TextureId, bool), String> {
        if let Some(atlas) = self.atlases.get_mut(&atlas_id) {
            // Create new texture ID
            let texture_id = self.next_texture_id;
            self.next_texture_id = texture_id.next();
            // Register in atlas
            atlas.register_region(texture_id, linear_index);

            // Store mapping for O(1) lookup
            self.texture_atlas_map.insert(texture_id, atlas_id);

            Ok((texture_id, atlas.is_transparent(&texture_id)))
        } else {
            Err(format!("Atlas with ID {:?} not found", atlas_id))
        }
    }

    pub fn draw_texture(
        &self,
        texture_id: &TextureId,
        x: f32,
        y: f32,
        color: Color,
    ) -> Result<(), String> {
        if let Some(&atlas_id) = self.texture_atlas_map.get(texture_id) {
            if let Some(atlas) = self.atlases.get(&atlas_id) {
                atlas.draw(texture_id, x, y, color);
                return Ok(());
            }
        }
        Err(format!(
            "Texture ID {:?} not found in any atlas",
            texture_id
        ))
    }
}
