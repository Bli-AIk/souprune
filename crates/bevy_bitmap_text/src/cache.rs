use std::collections::HashMap;

use bevy::image::Image;
use bevy::math::{Rect, UVec2};
use bevy::prelude::*;
use etagere::{AllocId, BucketedAtlasAllocator, size2};
use fontdue::layout::GlyphRasterConfig;

use crate::font_id::FontId;

/// Key for looking up a rasterized glyph in the cache.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GlyphKey {
    pub font_id: FontId,
    pub character: char,
    pub size_px: u32,
}

/// Cached info for a single rasterized glyph.
#[derive(Debug, Clone)]
pub struct GlyphInfo {
    /// UV rectangle in the atlas (in pixels, NOT normalized).
    pub pixel_rect: Rect,
    /// Glyph metrics from fontdue.
    pub metrics: fontdue::Metrics,
    /// Allocation ID for etagere (used for deallocation).
    pub alloc_id: AllocId,
}

/// Configuration for the dynamic glyph cache.
#[derive(Debug, Clone)]
pub struct GlyphCacheConfig {
    pub atlas_width: u32,
    pub atlas_height: u32,
    /// Padding around each glyph in the atlas (prevents bleeding).
    pub glyph_padding: u32,
}

impl Default for GlyphCacheConfig {
    fn default() -> Self {
        Self {
            atlas_width: 2048,
            atlas_height: 2048,
            glyph_padding: 1,
        }
    }
}

/// Global dynamic glyph cache resource.
///
/// Rasterizes glyphs on demand using `fontdue` and packs them into a single
/// atlas texture using `etagere`.
#[derive(Resource)]
pub struct DynamicGlyphCache {
    fonts: HashMap<FontId, fontdue::Font>,
    glyph_map: HashMap<GlyphKey, GlyphInfo>,
    allocator: BucketedAtlasAllocator,
    pub config: GlyphCacheConfig,
    pub atlas_image: Handle<Image>,
    atlas_dirty: bool,
}

impl DynamicGlyphCache {
    pub fn new(config: GlyphCacheConfig, images: &mut Assets<Image>) -> Self {
        let image = Image::new_fill(
            bevy::render::render_resource::Extent3d {
                width: config.atlas_width,
                height: config.atlas_height,
                depth_or_array_layers: 1,
            },
            bevy::render::render_resource::TextureDimension::D2,
            &[0, 0, 0, 0],
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            Default::default(),
        );
        let handle = images.add(image);

        let allocator = BucketedAtlasAllocator::new(size2(
            config.atlas_width as i32,
            config.atlas_height as i32,
        ));

        Self {
            fonts: HashMap::new(),
            glyph_map: HashMap::new(),
            allocator,
            config,
            atlas_image: handle,
            atlas_dirty: false,
        }
    }

    /// Register a font from raw bytes.
    pub fn add_font(&mut self, id: FontId, data: &[u8]) -> Result<(), String> {
        let settings = fontdue::FontSettings {
            collection_index: 0,
            scale: 40.0,
        };
        let font = fontdue::Font::from_bytes(data, settings)
            .map_err(|e| format!("Failed to load font {:?}: {}", id, e))?;
        self.fonts.insert(id, font);
        Ok(())
    }

    /// Get a cached glyph, or rasterize and cache it on demand.
    /// Returns `None` if the font is not loaded or the atlas is full.
    pub fn get_or_insert(
        &mut self,
        key: &GlyphKey,
        images: &mut Assets<Image>,
    ) -> Option<&GlyphInfo> {
        if self.glyph_map.contains_key(key) {
            return self.glyph_map.get(key);
        }

        let font = self.fonts.get(&key.font_id)?;
        let (metrics, bitmap) = font.rasterize(key.character, key.size_px as f32);

        if metrics.width == 0 || metrics.height == 0 {
            let info = GlyphInfo {
                pixel_rect: Rect::new(0.0, 0.0, 0.0, 0.0),
                metrics,
                alloc_id: AllocId::deserialize(0),
            };
            self.glyph_map.insert(key.clone(), info);
            return self.glyph_map.get(key);
        }

        let padded_w = metrics.width as i32 + self.config.glyph_padding as i32 * 2;
        let padded_h = metrics.height as i32 + self.config.glyph_padding as i32 * 2;

        let alloc = self.allocator.allocate(size2(padded_w, padded_h))?;
        let rect = alloc.rectangle;
        let pad = self.config.glyph_padding as i32;
        let glyph_x = rect.min.x + pad;
        let glyph_y = rect.min.y + pad;

        if let Some(image) = images.get_mut(self.atlas_image.id()) {
            write_glyph_to_atlas(image, &bitmap, &metrics, glyph_x as usize, glyph_y as usize);
            self.atlas_dirty = true;
        }

        let pixel_rect = Rect::new(
            glyph_x as f32,
            glyph_y as f32,
            (glyph_x + metrics.width as i32) as f32,
            (glyph_y + metrics.height as i32) as f32,
        );

        let info = GlyphInfo {
            pixel_rect,
            metrics,
            alloc_id: alloc.id,
        };
        self.glyph_map.insert(key.clone(), info);
        self.glyph_map.get(key)
    }

    /// Clear the entire cache and reset the allocator.
    /// Call this on scene transitions to reclaim atlas space.
    pub fn clear(&mut self, images: &mut Assets<Image>) {
        self.glyph_map.clear();
        self.allocator.clear();
        if let Some(image) = images.get_mut(self.atlas_image.id())
            && let Some(ref mut data) = image.data
        {
            data.fill(0);
        }
    }

    pub fn has_font(&self, id: &FontId) -> bool {
        self.fonts.contains_key(id)
    }

    /// Read-only access to glyph cache (for systems that don't rasterize).
    pub fn glyph_map_get(&self, key: &GlyphKey) -> Option<&GlyphInfo> {
        self.glyph_map.get(key)
    }

    /// Get font metrics (width, height, x_offset, y_offset) for a character.
    /// Works without rasterization — just queries fontdue for metrics.
    pub fn glyph_metrics(
        &self,
        font_id: &FontId,
        character: char,
        size_px: u32,
    ) -> (u32, u32, f32, f32) {
        if let Some(font) = self.fonts.get(font_id) {
            let metrics = font.metrics(character, size_px as f32);
            (
                metrics.width as u32,
                metrics.height as u32,
                metrics.xmin as f32,
                metrics.ymin as f32,
            )
        } else {
            (0, 0, 0.0, 0.0)
        }
    }

    pub fn atlas_size(&self) -> UVec2 {
        UVec2::new(self.config.atlas_width, self.config.atlas_height)
    }

    /// Check if atlas was modified since last call to `acknowledge_dirty`.
    pub fn is_dirty(&self) -> bool {
        self.atlas_dirty
    }

    pub fn acknowledge_dirty(&mut self) {
        self.atlas_dirty = false;
    }

    /// Get horizontal advance for a character (in pixels at given size).
    pub fn horizontal_advance(&self, font_id: &FontId, character: char, size_px: u32) -> f32 {
        if let Some(font) = self.fonts.get(font_id) {
            let metrics = font.metrics(character, size_px as f32);
            metrics.advance_width
        } else {
            size_px as f32 * 0.5
        }
    }

    /// Get line metrics for a font at a given size.
    pub fn line_metrics(&self, font_id: &FontId, size_px: u32) -> Option<fontdue::LineMetrics> {
        self.fonts
            .get(font_id)?
            .horizontal_line_metrics(size_px as f32)
    }

    /// Get raster config for looking up glyphs in fontdue.
    pub fn raster_config(
        &self,
        font_id: &FontId,
        character: char,
        size_px: f32,
    ) -> Option<GlyphRasterConfig> {
        self.fonts.get(font_id).map(|font| GlyphRasterConfig {
            glyph_index: font.lookup_glyph_index(character),
            px: size_px,
            font_hash: font.file_hash(),
        })
    }

    /// Create a headless cache for testing (no Bevy Assets).
    #[cfg(test)]
    pub fn new_headless() -> Self {
        Self {
            fonts: HashMap::new(),
            glyph_map: HashMap::new(),
            allocator: BucketedAtlasAllocator::new(size2(256, 256)),
            config: GlyphCacheConfig::default(),
            atlas_image: Handle::default(),
            atlas_dirty: false,
        }
    }
}

/// Write a rasterized glyph bitmap into the atlas image at the given position.
fn write_glyph_to_atlas(
    image: &mut Image,
    bitmap: &[u8],
    metrics: &fontdue::Metrics,
    glyph_x: usize,
    glyph_y: usize,
) {
    let atlas_w = image.width() as usize;
    let pixel_size = 4; // RGBA8
    let Some(ref mut data) = image.data else {
        return;
    };

    for row in 0..metrics.height {
        for col in 0..metrics.width {
            let src_idx = row * metrics.width + col;
            let coverage = bitmap[src_idx];
            let dst_x = glyph_x + col;
            let dst_y = glyph_y + row;
            let dst_idx = (dst_y * atlas_w + dst_x) * pixel_size;

            if dst_idx + 3 < data.len() {
                data[dst_idx] = 255; // R
                data[dst_idx + 1] = 255; // G
                data[dst_idx + 2] = 255; // B
                data[dst_idx + 3] = coverage; // A
            }
        }
    }
}
