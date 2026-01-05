//! Data structures for Alight Motion XML schema.
//!
//! This module provides strongly-typed representations of AM project files,
//! with robust handling of optional fields and defaults.

use serde::{Deserialize, Deserializer};

/// Root scene node containing project metadata and layers.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "scene")]
pub struct AmScene {
    /// Project title.
    #[serde(rename = "@title", default)]
    pub title: String,

    /// Canvas width in pixels.
    #[serde(rename = "@width", default = "default_canvas_size")]
    pub width: u32,

    /// Canvas height in pixels.
    #[serde(rename = "@height", default = "default_canvas_size")]
    pub height: u32,

    /// Export width in pixels.
    #[serde(rename = "@exportWidth", default = "default_canvas_size")]
    pub export_width: u32,

    /// Export height in pixels.
    #[serde(rename = "@exportHeight", default = "default_canvas_size")]
    pub export_height: u32,

    /// Frames per second.
    #[serde(rename = "@fps", default = "default_fps")]
    pub fps: u32,

    /// Total duration in milliseconds.
    #[serde(rename = "@totalTime", default)]
    pub total_time: u32,

    /// Background color in #AARRGGBB format.
    #[serde(rename = "@bgcolor", default = "default_bgcolor")]
    pub bgcolor: String,

    /// AM version number.
    #[serde(rename = "@amver", default)]
    pub amver: u32,

    /// Time remapping strategy.
    #[serde(rename = "@retime", default)]
    pub retime: String,

    /// Precompose mode.
    #[serde(rename = "@precompose", default)]
    pub precompose: String,

    /// Media resources.
    #[serde(rename = "media", default)]
    pub media: Vec<AmMedia>,

    /// Scene layers (shapes, nullobjs, embedScenes).
    #[serde(rename = "$value", default)]
    pub layers: Vec<AmLayer>,
}

fn default_canvas_size() -> u32 {
    1280
}

fn default_fps() -> u32 {
    60
}

fn default_bgcolor() -> String {
    "#ff000000".to_string()
}

/// Media resource definition.
#[derive(Debug, Clone, Deserialize)]
pub struct AmMedia {
    /// Resource URI (e.g., "amproj:filename.png").
    #[serde(rename = "@uri", default)]
    pub uri: String,

    /// Physical filename.
    #[serde(rename = "@filename", default)]
    pub filename: String,

    /// MIME type (e.g., "image/png").
    #[serde(rename = "@type", default)]
    pub media_type: String,

    /// Original width in pixels.
    #[serde(rename = "@width", default)]
    pub width: u32,

    /// Original height in pixels.
    #[serde(rename = "@height", default)]
    pub height: u32,

    /// File size in bytes.
    #[serde(rename = "@size", default)]
    pub size: u32,

    /// SHA1 signature.
    #[serde(rename = "@sig", default)]
    pub sig: String,
}

/// Layer types in the scene.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AmLayer {
    /// Visible shape layer.
    Shape(AmShape),
    /// Null/empty object for grouping.
    Nullobj(AmNullObj),
    /// Embedded sub-scene (pre-composition).
    EmbedScene(AmEmbedScene),
}

/// Common layer properties.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AmLayerBase {
    /// Unique identifier.
    #[serde(rename = "@id", default)]
    pub id: u64,

    /// Layer label/name.
    #[serde(rename = "@label", default)]
    pub label: String,

    /// In-point in milliseconds.
    #[serde(rename = "@startTime", default)]
    pub start_time: i32,

    /// Out-point in milliseconds.
    #[serde(rename = "@endTime", default)]
    pub end_time: i32,

    /// Parent layer ID (0 if no parent).
    #[serde(rename = "@parent", default)]
    pub parent: u64,

    /// Alternative out-point.
    #[serde(rename = "@outTime", default)]
    pub out_time: Option<i32>,
}

/// Shape layer (visible object).
#[derive(Debug, Clone, Deserialize)]
pub struct AmShape {
    /// Unique identifier.
    #[serde(rename = "@id", default)]
    pub id: u64,

    /// Layer label/name.
    #[serde(rename = "@label", default)]
    pub label: String,

    /// In-point in milliseconds.
    #[serde(rename = "@startTime", default)]
    pub start_time: i32,

    /// Out-point in milliseconds.
    #[serde(rename = "@endTime", default)]
    pub end_time: i32,

    /// Parent layer ID.
    #[serde(rename = "@parent", default)]
    pub parent: u64,

    /// Fill type ("color" or "media").
    #[serde(rename = "@fillType", default)]
    pub fill_type: String,

    /// Fill image URI (when fillType="media").
    #[serde(rename = "@fillImage", default)]
    pub fill_image: String,

    /// Shape type (e.g., ".rect", ".circle").
    #[serde(rename = "@s", default)]
    pub shape_type: String,

    /// Transform data.
    #[serde(default)]
    pub transform: AmTransform,

    /// Shape properties.
    #[serde(rename = "property", default)]
    pub properties: Vec<AmProperty>,

    /// Effects applied to this shape.
    #[serde(rename = "effect", default)]
    pub effects: Vec<AmEffect>,

    /// Fill color (when fillType="color").
    #[serde(rename = "fillColor", default)]
    pub fill_color: Option<AmFillColor>,
}

/// Null object (invisible parent controller).
#[derive(Debug, Clone, Deserialize)]
pub struct AmNullObj {
    /// Unique identifier.
    #[serde(rename = "@id", default)]
    pub id: u64,

    /// Layer label/name.
    #[serde(rename = "@label", default)]
    pub label: String,

    /// In-point in milliseconds.
    #[serde(rename = "@startTime", default)]
    pub start_time: i32,

    /// Out-point in milliseconds.
    #[serde(rename = "@endTime", default)]
    pub end_time: i32,

    /// Parent layer ID.
    #[serde(rename = "@parent", default)]
    pub parent: u64,

    /// Object type (e.g., "perspective").
    #[serde(rename = "@type", default)]
    pub obj_type: String,

    /// Transform data.
    #[serde(default)]
    pub transform: AmTransform,

    /// Effects applied to this object.
    #[serde(rename = "effect", default)]
    pub effects: Vec<AmEffect>,
}

/// Embedded sub-scene (pre-composition).
#[derive(Debug, Clone, Deserialize)]
pub struct AmEmbedScene {
    /// Unique identifier.
    #[serde(rename = "@id", default)]
    pub id: u64,

    /// Layer label/name.
    #[serde(rename = "@label", default)]
    pub label: String,

    /// In-point in milliseconds.
    #[serde(rename = "@startTime", default)]
    pub start_time: i32,

    /// Out-point in milliseconds.
    #[serde(rename = "@endTime", default)]
    pub end_time: i32,

    /// Parent layer ID.
    #[serde(rename = "@parent", default)]
    pub parent: u64,

    /// Fill type.
    #[serde(rename = "@fillType", default)]
    pub fill_type: String,

    /// Alternative out-point.
    #[serde(rename = "@outTime", default)]
    pub out_time: Option<i32>,

    /// Transform data.
    #[serde(default)]
    pub transform: AmTransform,

    /// Fill color.
    #[serde(rename = "fillColor", default)]
    pub fill_color: Option<AmFillColor>,

    /// Nested scene.
    pub scene: Box<AmScene>,
}

/// Fill color definition.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AmFillColor {
    /// Color value in #AARRGGBB format.
    #[serde(rename = "@value", default)]
    pub value: String,
}

/// Transform container with animated properties.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AmTransform {
    /// Lock aspect ratio flag.
    #[serde(rename = "@lockAspectRatio", default)]
    pub lock_aspect_ratio: bool,

    /// Location/position property.
    #[serde(default)]
    pub location: AmAnimatedVec3,

    /// Rotation property (Z-axis, degrees).
    #[serde(default)]
    pub rotation: AmAnimatedFloat,

    /// Scale property.
    #[serde(default)]
    pub scale: AmAnimatedVec2,

    /// Opacity property (0.0-1.0).
    #[serde(default)]
    pub opacity: AmAnimatedFloat,
}

/// Animated Vec3 property (x, y, z).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AmAnimatedVec3 {
    /// Static value (if not animated).
    #[serde(rename = "@value", default, deserialize_with = "deserialize_vec3_opt")]
    pub value: Option<[f32; 3]>,

    /// Keyframes (if animated).
    #[serde(rename = "kf", default)]
    pub keyframes: Vec<AmKeyframe>,
}

/// Animated Vec2 property (x, y).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AmAnimatedVec2 {
    /// Static value (if not animated).
    #[serde(rename = "@value", default, deserialize_with = "deserialize_vec2_opt")]
    pub value: Option<[f32; 2]>,

    /// Keyframes (if animated).
    #[serde(rename = "kf", default)]
    pub keyframes: Vec<AmKeyframe>,
}

/// Animated float property.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AmAnimatedFloat {
    /// Static value (if not animated).
    #[serde(rename = "@value", default)]
    pub value: Option<f32>,

    /// Keyframes (if animated).
    #[serde(rename = "kf", default)]
    pub keyframes: Vec<AmKeyframe>,
}

/// Keyframe definition.
#[derive(Debug, Clone, Deserialize)]
pub struct AmKeyframe {
    /// Normalized time (0.0-1.0).
    #[serde(rename = "@t", default)]
    pub time: f32,

    /// Value at this keyframe (string format varies by property type).
    #[serde(rename = "@v", default)]
    pub value: String,

    /// Easing function (e.g., "cubicBezier 0.0 0.0 0.58 1.0", "step 1.0 0.0").
    #[serde(rename = "@e", default)]
    pub easing: Option<String>,
}

/// Property definition (e.g., size).
#[derive(Debug, Clone, Deserialize)]
pub struct AmProperty {
    /// Property name.
    #[serde(rename = "@name", default)]
    pub name: String,

    /// Property type (e.g., "vec2", "float").
    #[serde(rename = "@type", default)]
    pub prop_type: String,

    /// Static value.
    #[serde(rename = "@value", default)]
    pub value: String,

    /// Keyframes (if animated).
    #[serde(rename = "kf", default)]
    pub keyframes: Vec<AmKeyframe>,
}

/// Effect definition.
#[derive(Debug, Clone, Deserialize)]
pub struct AmEffect {
    /// Effect type ID.
    #[serde(rename = "@id", default)]
    pub id: String,

    /// Whether applied locally.
    #[serde(rename = "@locallyApplied", default)]
    pub locally_applied: bool,

    /// Effect properties.
    #[serde(rename = "property", default)]
    pub properties: Vec<AmProperty>,
}

// Custom deserializers for vector types

fn deserialize_vec3_opt<'de, D>(deserializer: D) -> Result<Option<[f32; 3]>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(s) if !s.is_empty() => parse_vec3(&s).map(Some).map_err(serde::de::Error::custom),
        _ => Ok(None),
    }
}

fn deserialize_vec2_opt<'de, D>(deserializer: D) -> Result<Option<[f32; 2]>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(s) if !s.is_empty() => parse_vec2(&s).map(Some).map_err(serde::de::Error::custom),
        _ => Ok(None),
    }
}

/// Parse a comma-separated Vec3 string.
pub fn parse_vec3(s: &str) -> Result<[f32; 3], String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() >= 3 {
        Ok([
            parts[0].trim().parse().map_err(|e| format!("{}", e))?,
            parts[1].trim().parse().map_err(|e| format!("{}", e))?,
            parts[2].trim().parse().map_err(|e| format!("{}", e))?,
        ])
    } else if parts.len() == 2 {
        Ok([
            parts[0].trim().parse().map_err(|e| format!("{}", e))?,
            parts[1].trim().parse().map_err(|e| format!("{}", e))?,
            0.0,
        ])
    } else if parts.len() == 1 && !s.is_empty() {
        let v: f32 = parts[0].trim().parse().map_err(|e| format!("{}", e))?;
        Ok([v, v, v])
    } else {
        Err(format!("Invalid vec3 format: {}", s))
    }
}

/// Parse a comma-separated Vec2 string.
pub fn parse_vec2(s: &str) -> Result<[f32; 2], String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() >= 2 {
        Ok([
            parts[0].trim().parse().map_err(|e| format!("{}", e))?,
            parts[1].trim().parse().map_err(|e| format!("{}", e))?,
        ])
    } else if parts.len() == 1 && !s.is_empty() {
        let v: f32 = parts[0].trim().parse().map_err(|e| format!("{}", e))?;
        Ok([v, v])
    } else {
        Err(format!("Invalid vec2 format: {}", s))
    }
}

/// Parse color from #AARRGGBB format to [r, g, b, a] in 0.0-1.0 range.
pub fn parse_color(s: &str) -> Result<[f32; 4], String> {
    let s = s.trim_start_matches('#');
    if s.len() != 8 {
        return Err(format!("Invalid color format: #{}", s));
    }

    let a = u8::from_str_radix(&s[0..2], 16).map_err(|e| format!("{}", e))?;
    let r = u8::from_str_radix(&s[2..4], 16).map_err(|e| format!("{}", e))?;
    let g = u8::from_str_radix(&s[4..6], 16).map_err(|e| format!("{}", e))?;
    let b = u8::from_str_radix(&s[6..8], 16).map_err(|e| format!("{}", e))?;

    Ok([
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    ])
}

/// Easing function type.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Easing {
    /// Linear interpolation (default).
    #[default]
    Linear,
    /// Step function (instant transition).
    Step { x: f32, y: f32 },
    /// Cubic bezier curve with control points.
    CubicBezier { x1: f32, y1: f32, x2: f32, y2: f32 },
}

impl Easing {
    /// Parse easing string from AM format.
    pub fn parse(s: &str) -> Self {
        let s = s.trim();
        if s.is_empty() {
            return Easing::Linear;
        }

        let parts: Vec<&str> = s.split_whitespace().collect();
        match parts.first().copied() {
            Some("step") => {
                let x = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1.0);
                let y = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0.0);
                Easing::Step { x, y }
            }
            Some("cubicBezier") => {
                let x1 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0.0);
                let y1 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0.0);
                let x2 = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(1.0);
                let y2 = parts.get(4).and_then(|s| s.parse().ok()).unwrap_or(1.0);
                Easing::CubicBezier { x1, y1, x2, y2 }
            }
            _ => Easing::Linear,
        }
    }

    /// Evaluate the easing function at normalized time t (0.0-1.0).
    pub fn evaluate(&self, t: f32) -> f32 {
        match self {
            Easing::Linear => t,
            Easing::Step { .. } => {
                // Step function: hold previous value until t reaches 1.0
                if t < 1.0 { 0.0 } else { 1.0 }
            }
            Easing::CubicBezier { x1, y1, x2, y2 } => cubic_bezier_y_for_x(t, *x1, *y1, *x2, *y2),
        }
    }
}

/// Solve cubic bezier curve: find Y for given X.
/// Control points are (0,0), (x1,y1), (x2,y2), (1,1).
fn cubic_bezier_y_for_x(x: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    // Find t for given x using Newton's method
    let mut t = x;
    for _ in 0..8 {
        let x_t = bezier_component(t, x1, x2);
        let dx = x - x_t;
        if dx.abs() < 1e-6 {
            break;
        }
        let dx_dt = bezier_derivative(t, x1, x2);
        if dx_dt.abs() < 1e-6 {
            break;
        }
        t += dx / dx_dt;
        t = t.clamp(0.0, 1.0);
    }

    bezier_component(t, y1, y2)
}

/// Evaluate one component of a cubic bezier at parameter t.
/// B(t) = 3(1-t)²t*p1 + 3(1-t)t²*p2 + t³
fn bezier_component(t: f32, p1: f32, p2: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;

    3.0 * mt2 * t * p1 + 3.0 * mt * t2 * p2 + t3
}

/// Derivative of bezier component with respect to t.
fn bezier_derivative(t: f32, p1: f32, p2: f32) -> f32 {
    let t2 = t * t;
    let mt = 1.0 - t;

    3.0 * mt * mt * p1 + 6.0 * mt * t * (p2 - p1) + 3.0 * t2 * (1.0 - p2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_vec3() {
        assert_eq!(parse_vec3("1.0,2.0,3.0").unwrap(), [1.0, 2.0, 3.0]);
        assert_eq!(
            parse_vec3("640.0, 480.0, 0.0").unwrap(),
            [640.0, 480.0, 0.0]
        );
        assert_eq!(parse_vec3("-1.5,2.5,0").unwrap(), [-1.5, 2.5, 0.0]);
    }

    #[test]
    fn test_parse_vec2() {
        assert_eq!(parse_vec2("100.0,200.0").unwrap(), [100.0, 200.0]);
        assert_eq!(parse_vec2("1.5, 2.5").unwrap(), [1.5, 2.5]);
    }

    #[test]
    fn test_parse_color() {
        let color = parse_color("#ff000000").unwrap();
        assert_eq!(color, [0.0, 0.0, 0.0, 1.0]);

        let color = parse_color("#ffffffff").unwrap();
        assert_eq!(color, [1.0, 1.0, 1.0, 1.0]);

        let color = parse_color("#80ff0000").unwrap();
        assert!((color[0] - 1.0).abs() < 0.01);
        assert!((color[1] - 0.0).abs() < 0.01);
        assert!((color[2] - 0.0).abs() < 0.01);
        assert!((color[3] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_easing_parse() {
        assert_eq!(Easing::parse(""), Easing::Linear);
        assert_eq!(
            Easing::parse("step 1.0 0.0"),
            Easing::Step { x: 1.0, y: 0.0 }
        );
        assert_eq!(
            Easing::parse("cubicBezier 0.0 0.0 0.58 1.0"),
            Easing::CubicBezier {
                x1: 0.0,
                y1: 0.0,
                x2: 0.58,
                y2: 1.0
            }
        );
    }

    #[test]
    fn test_easing_linear() {
        let easing = Easing::Linear;
        assert!((easing.evaluate(0.0) - 0.0).abs() < 0.001);
        assert!((easing.evaluate(0.5) - 0.5).abs() < 0.001);
        assert!((easing.evaluate(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_easing_step() {
        let easing = Easing::Step { x: 1.0, y: 0.0 };
        assert!((easing.evaluate(0.0) - 0.0).abs() < 0.001);
        assert!((easing.evaluate(0.5) - 0.0).abs() < 0.001);
        assert!((easing.evaluate(0.99) - 0.0).abs() < 0.001);
        assert!((easing.evaluate(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_easing_cubic_bezier() {
        // ease-out curve
        let easing = Easing::CubicBezier {
            x1: 0.0,
            y1: 0.0,
            x2: 0.58,
            y2: 1.0,
        };

        assert!((easing.evaluate(0.0) - 0.0).abs() < 0.01);
        assert!((easing.evaluate(1.0) - 1.0).abs() < 0.01);

        // ease-out should be faster at start
        let mid = easing.evaluate(0.5);
        assert!(mid > 0.5, "ease-out at 0.5 should be > 0.5, got {}", mid);
    }
}
