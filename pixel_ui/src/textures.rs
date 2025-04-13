use macroquad::prelude::*;
use std::collections::HashMap;
// src/atlas.rs
use ::utils::repo::NumericId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureId(pub u32);
impl NumericId for TextureId {
    fn value(&self) -> u32 {
        self.0
    }

    fn from_value(value: u32) -> Self {
        TextureId(value)
    }
}

pub struct TextureAtlas {
    pub texture: Texture2D,
    pub tile_size: f32,
    pub width_in_tiles: u32,
    regions: HashMap<TextureId, u32>, // Maps texture_id -> linear_index
}

impl TextureAtlas {
    pub fn new(texture: Texture2D, tile_size: f32, width_in_tiles: u32) -> Self {
        Self {
            texture,
            tile_size,
            width_in_tiles,
            regions: HashMap::new(),
        }
    }

    pub fn register_region(&mut self, texture_id: TextureId, linear_index: u32) {
        self.regions.insert(texture_id, linear_index);
    }

    pub fn draw(&self, texture_id: TextureId, screen_x: f32, screen_y: f32) {
        if let Some(&linear_index) = self.regions.get(&texture_id) {
            let col = linear_index % self.width_in_tiles;
            let row = linear_index / self.width_in_tiles;

            let src_x = col as f32 * self.tile_size;
            let src_y = row as f32 * self.tile_size;

            draw_texture_ex(
                &self.texture,
                screen_x,
                screen_y,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(src_x, src_y, self.tile_size, self.tile_size)),
                    ..Default::default()
                },
            );
        }
    }
}
pub struct TextureAtlasManager {
    atlases: HashMap<u32, TextureAtlas>,
    next_atlas_id: u32,
}

impl TextureAtlasManager {
    pub fn new() -> Self {
        Self {
            atlases: HashMap::new(),
            next_atlas_id: 1,
        }
    }

    pub async fn create_atlas(
        &mut self,
        path: &str,
        tile_size: f32,
        width_in_tiles: u32,
    ) -> Result<u32, String> {
        // Load texture synchronously
        let texture = load_texture_sync(path).await?;

        // Create atlas
        let atlas = TextureAtlas::new(texture, tile_size, width_in_tiles);
        let id = self.next_atlas_id;
        self.next_atlas_id += 1;

        // Store atlas
        self.atlases.insert(id, atlas);

        Ok(id)
    }
}

// Helper function for synchronous texture loading
async fn load_texture_sync(path: &str) -> Result<Texture2D, String> {
    match load_texture(path).await {
        Ok(texture) => Ok(texture),
        Err(e) => Err(format!("Failed to load texture: {}", e)),
    }
}
