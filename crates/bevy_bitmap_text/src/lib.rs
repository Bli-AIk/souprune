//! # bevy_bitmap_text
//!
//! Glyph-as-Entity dynamic atlas text rendering backend for Bevy.
//!
//! Each character is spawned as an individual entity with a `Sprite`,
//! enabling per-character ECS-driven animation (shake, wave, color changes).
//!
//! ## Key Types
//!
//! - [`TextBlock`] — The main component, replaces `Text3d`.
//! - [`TextBlockStyling`] — Font, size, color, alignment configuration.
//! - [`DynamicGlyphCache`] — Global resource managing the glyph atlas.
//! - [`GlyphEntity`] — Marker component on individual character entities.
//! - [`parse_text_to_segments`] — Parses `{#RRGGBB:content}` color tags.

pub mod cache;
pub mod components;
pub mod font_id;
pub mod layout;
pub mod parse;
pub mod systems;

pub use cache::{DynamicGlyphCache, GlyphCacheConfig, GlyphInfo, GlyphKey};
pub use components::{
    GlyphBaseOffset, GlyphEntity, GlyphReveal, LayoutGlyph, SegmentStyle, ShakeEffect, TextAlign,
    TextAnchor, TextBlock, TextBlockLayout, TextBlockStyling, TextSegment, WaveEffect,
};
pub use font_id::FontId;
pub use parse::parse_text_to_segments;
pub use systems::BitmapTextSet;

use bevy::app::{App, Plugin, PostUpdate};
use bevy::ecs::schedule::IntoScheduleConfigs;

/// Resource specifying directories to scan for font files at startup.
#[derive(bevy::prelude::Resource, Default)]
pub struct FontDirectories {
    pub directories: Vec<String>,
}

/// Plugin that registers all bitmap text systems.
#[derive(Default)]
pub struct BitmapTextPlugin {
    pub atlas_config: GlyphCacheConfig,
}

impl Plugin for BitmapTextPlugin {
    fn build(&self, app: &mut App) {
        use systems::*;

        // Initialize the glyph cache resource.
        let mut images = app
            .world_mut()
            .resource_mut::<bevy::asset::Assets<bevy::image::Image>>();
        let cache = DynamicGlyphCache::new(self.atlas_config.clone(), &mut images);
        app.insert_resource(cache);

        // Register system set.
        app.configure_sets(
            PostUpdate,
            BitmapTextSet.before(bevy::transform::TransformSystems::Propagate),
        );

        app.add_systems(
            PostUpdate,
            (
                rasterize_glyphs_system,
                layout_text_system.after(rasterize_glyphs_system),
                sync_glyph_entities_system.after(layout_text_system),
                glyph_reveal_system.after(sync_glyph_entities_system),
                show_text_when_ready_system.after(glyph_reveal_system),
            )
                .in_set(BitmapTextSet),
        );

        // Animation systems run in Update (frame-rate dependent).
        app.add_systems(bevy::app::Update, (text_shake_system, text_wave_system));

        // Load fonts from disk at startup.
        app.add_systems(bevy::app::Startup, load_fonts_from_directories);
    }
}

/// Load fonts from directories specified in `FontDirectories` resource.
pub fn load_fonts_from_directories(
    dirs: Option<bevy::prelude::Res<FontDirectories>>,
    mut cache: bevy::prelude::ResMut<DynamicGlyphCache>,
) {
    let Some(dirs) = dirs else {
        return;
    };
    for dir in &dirs.directories {
        let Ok(entries) = std::fs::read_dir(dir) else {
            log::warn!("Font directory not found: {}", dir);
            continue;
        };

        for entry in entries.flatten() {
            load_single_font(&entry.path(), &mut cache);
        }
    }
}

fn load_single_font(path: &std::path::Path, cache: &mut DynamicGlyphCache) {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if ext != "ttf" && ext != "otf" {
        return;
    }

    let data = match std::fs::read(path) {
        Ok(d) => d,
        Err(e) => {
            log::warn!("Failed to read font file {:?}: {}", path, e);
            return;
        }
    };

    let name = path
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    match cache.add_font(FontId::from_name(&name), &data) {
        Ok(()) => log::info!("Loaded font: {} from {:?}", name, path),
        Err(e) => log::warn!("Failed to load font {:?}: {}", path, e),
    }
}
