//! # SVG Icon Rendering
//!
//! # SVG 图标渲染
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! SVG icon rendering for the editor UI.
//! Uses `icondata_vs` (VS Code Codicons) and `resvg` to pre-render
//! SVG icons into egui textures at startup.
//!
//! 编辑器 UI 的 SVG 图标渲染。
//! 使用 `icondata_vs` (VS Code Codicons) 和 `resvg` 在启动时将 SVG 图标预渲染为 egui 纹理。

use std::collections::HashMap;

use bevy::prelude::*;
use icondata_core::IconData;

const ICON_SIZE: u32 = 16;

/// Pre-rendered icon textures cached for egui.
#[derive(Resource, Default)]
pub struct EditorIcons {
    textures: HashMap<&'static str, egui::TextureHandle>,
}

impl EditorIcons {
    /// Get a pre-rendered icon texture by name.
    pub fn get(&self, name: &str) -> Option<&egui::TextureHandle> {
        self.textures.get(name)
    }

    /// Show an icon as an egui image widget.
    pub fn show(&self, ui: &mut egui::Ui, name: &str) {
        if let Some(tex) = self.textures.get(name) {
            let size = egui::vec2(ICON_SIZE as f32, ICON_SIZE as f32);
            ui.image(egui::load::SizedTexture::new(tex.id(), size));
        }
    }

    /// Show an icon with custom tint color.
    pub fn show_tinted(&self, ui: &mut egui::Ui, name: &str, tint: egui::Color32) {
        if let Some(tex) = self.textures.get(name) {
            let size = egui::vec2(ICON_SIZE as f32, ICON_SIZE as f32);
            ui.add(egui::Image::new(egui::load::SizedTexture::new(tex.id(), size)).tint(tint));
        }
    }
}

/// Registry of icons to pre-render.
fn icon_registry() -> Vec<(&'static str, &'static IconData)> {
    use icondata_vs::*;
    vec![
        // File browser
        ("folder", VsFolder),
        ("folder_open", VsFolderOpened),
        ("file", VsFile),
        ("file_code", VsFileCode),
        ("file_media", VsFileMedia),
        // Asset types
        ("sequence", VsListOrdered),
        ("view", VsEye),
        ("rule", VsSymbolEvent),
        ("performance", VsFlame),
        ("config", VsGear),
        // Toolbar
        ("refresh", VsRefresh),
        ("search", VsSearch),
        ("filter", VsFilter),
        ("new_file", VsNewFile),
        ("new_folder", VsNewFolder),
    ]
}

/// Build an SVG document string from an `IconData`.
fn icon_to_svg(icon: &IconData, fill_color: &str) -> String {
    let vb = icon.view_box.unwrap_or("0 0 16 16");
    let w = icon.width.unwrap_or("16");
    let h = icon.height.unwrap_or("16");

    let mut attrs = String::new();
    if let Some(sl) = icon.stroke_linecap {
        attrs.push_str(&format!(r#" stroke-linecap="{sl}""#));
    }
    if let Some(sj) = icon.stroke_linejoin {
        attrs.push_str(&format!(r#" stroke-linejoin="{sj}""#));
    }
    if let Some(sw) = icon.stroke_width {
        attrs.push_str(&format!(r#" stroke-width="{sw}""#));
    }
    if let Some(s) = icon.stroke {
        attrs.push_str(&format!(r#" stroke="{s}""#));
    }
    if let Some(style) = icon.style {
        attrs.push_str(&format!(r#" style="{style}""#));
    }

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{vb}" width="{w}" height="{h}" fill="{fill_color}"{attrs}>{}</svg>"#,
        icon.data
    )
}

/// Render an SVG string to an RGBA pixel buffer using `resvg`.
fn render_svg_to_pixels(svg_str: &str, size: u32) -> Option<Vec<u8>> {
    let opts = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_str(svg_str, &opts).ok()?;

    let mut pixmap = resvg::tiny_skia::Pixmap::new(size, size)?;

    let tree_size = tree.size();
    let sx = size as f32 / tree_size.width();
    let sy = size as f32 / tree_size.height();
    let scale = sx.min(sy);

    let dx = (size as f32 - tree_size.width() * scale) / 2.0;
    let dy = (size as f32 - tree_size.height() * scale) / 2.0;

    let transform = resvg::tiny_skia::Transform::from_scale(scale, scale).post_translate(dx, dy);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    Some(pixmap.data().to_vec())
}

/// Initialize all editor icons. Call once when the egui context is available.
pub fn init_icons(ctx: &egui::Context) -> EditorIcons {
    let fill = "#C8C8C8";
    let mut icons = EditorIcons::default();

    for (name, icon_data) in icon_registry() {
        let svg = icon_to_svg(icon_data, fill);
        if let Some(pixels) = render_svg_to_pixels(&svg, ICON_SIZE) {
            let image = egui::ColorImage::from_rgba_unmultiplied(
                [ICON_SIZE as usize, ICON_SIZE as usize],
                &pixels,
            );
            let handle = ctx.load_texture(
                format!("editor_icon_{name}"),
                image,
                egui::TextureOptions::LINEAR,
            );
            icons.textures.insert(name, handle);
        } else {
            warn!("Failed to render icon: {name}");
        }
    }

    info!("Initialized {} editor icons", icons.textures.len());
    icons
}
