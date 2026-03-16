use bevy::math::{Rect, Vec2};

use crate::cache::DynamicGlyphCache;
use crate::components::*;

/// Compute the layout for a text block, populating `TextBlockLayout`.
///
/// This is a pure function: it reads font metrics from the cache and
/// produces glyph positions. It does NOT rasterize or spawn entities.
pub fn compute_layout(
    block: &TextBlock,
    styling: &TextBlockStyling,
    cache: &DynamicGlyphCache,
) -> TextBlockLayout {
    let font_id = &styling.font;
    let size_px = styling.size_px;

    let line_metrics = cache.line_metrics(font_id, size_px);
    let line_height_px = if let Some(lm) = line_metrics {
        (lm.ascent - lm.descent + lm.line_gap) * styling.line_height
    } else {
        size_px as f32 * styling.line_height
    };

    let ascent = line_metrics
        .map(|lm| lm.ascent)
        .unwrap_or(size_px as f32 * 0.8);

    let mut glyphs = Vec::new();
    let mut cursor_x: f32 = 0.0;
    let mut cursor_y: f32 = 0.0;
    let mut char_index: usize = 0;
    let mut max_line_width: f32 = 0.0;
    let mut line_start_idx: usize = 0;

    // Track line info for alignment post-processing.
    let mut lines: Vec<LineInfo> = Vec::new();

    for segment in &block.segments {
        let color = segment.style.color.unwrap_or(styling.color);

        for ch in segment.text.chars() {
            if ch == '\n' {
                lines.push(LineInfo {
                    width: cursor_x,
                    start_glyph: line_start_idx,
                    end_glyph: glyphs.len(),
                });
                max_line_width = max_line_width.max(cursor_x);
                cursor_x = 0.0;
                cursor_y -= line_height_px;
                line_start_idx = glyphs.len();
                char_index += 1;
                continue;
            }

            let advance = cache.horizontal_advance(font_id, ch, size_px);

            // Auto line-wrapping.
            if let Some(max_w) = styling.max_width
                && cursor_x + advance > max_w
                && cursor_x > 0.0
            {
                lines.push(LineInfo {
                    width: cursor_x,
                    start_glyph: line_start_idx,
                    end_glyph: glyphs.len(),
                });
                max_line_width = max_line_width.max(cursor_x);
                cursor_x = 0.0;
                cursor_y -= line_height_px;
                line_start_idx = glyphs.len();
            }

            // Get glyph metrics from fontdue.
            let (glyph_w, glyph_h, x_offset, y_offset) = cache.glyph_metrics(font_id, ch, size_px);

            // Position: top-left corner of glyph bitmap (Y-up).
            // baseline = cursor_y - ascent (baseline is below line top)
            // glyph top (Y-up) = baseline + ymin + height
            let glyph_pos = Vec2::new(
                cursor_x + x_offset,
                cursor_y - ascent + y_offset + glyph_h as f32,
            );

            glyphs.push(LayoutGlyph {
                char_index,
                character: ch,
                position: glyph_pos,
                size: Vec2::new(glyph_w as f32, glyph_h as f32),
                uv_rect: Rect::default(),
                color,
                entity: None,
            });

            cursor_x += advance + styling.char_spacing;
            if ch == ' ' {
                cursor_x += styling.word_spacing;
            }
            char_index += 1;
        }
    }

    // Record the last line.
    lines.push(LineInfo {
        width: cursor_x,
        start_glyph: line_start_idx,
        end_glyph: glyphs.len(),
    });
    max_line_width = max_line_width.max(cursor_x);

    let total_height = -cursor_y + line_height_px;

    // Apply horizontal alignment.
    for line in &lines {
        let offset_x = (max_line_width - line.width) * styling.align.factor();
        for glyph in &mut glyphs[line.start_glyph..line.end_glyph] {
            glyph.position.x += offset_x;
        }
    }

    // Apply anchor offset.
    // The anchor name describes the text flow direction, not the anchor point position:
    //   BOTTOM_RIGHT (0.5, -0.5) → text starts at entity and extends right & down → shift (0, 0)
    //   CENTER       (0.0,  0.0) → text centered on entity → shift (-W/2, H/2)
    //   TOP_LEFT    (-0.5,  0.5) → text ends at entity, extends left & up → shift (-W, H)
    let dimension = Vec2::new(max_line_width, total_height);
    let shift = Vec2::new(
        dimension.x * (styling.anchor.0.x - 0.5),
        dimension.y * (0.5 + styling.anchor.0.y),
    );

    for glyph in &mut glyphs {
        glyph.position += shift;
    }

    TextBlockLayout { glyphs, dimension }
}

struct LineInfo {
    width: f32,
    start_glyph: usize,
    end_glyph: usize,
}
