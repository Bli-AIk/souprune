//! # bevy_alight_motion
//!
//! A Bevy plugin for loading and playing Alight Motion (AM) project files.
//!
//! ## Features
//!
//! - Load `.amproj` ZIP archives and standalone `.xml` project files
//! - Automatic keyframe animation with cubic-bezier and step easing
//! - Coordinate system conversion (AM top-left origin to Bevy center origin)
//! - Support for nested scenes (pre-compositions)
//! - Customizable playback control
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use bevy::prelude::*;
//! use bevy_alight_motion::prelude::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(AlightMotionPlugin)
//!         .add_systems(Startup, setup)
//!         .run();
//! }
//!
//! fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
//!     commands.spawn(Camera2d);
//!     load_am_project(&mut commands, &asset_server, "my_project.amproj");
//! }
//! ```
//!
//! ## Architecture
//!
//! The library is organized into three layers:
//!
//! 1. **Data Layer** (`schema`): Rust structs for XML deserialization
//! 2. **Resource Layer** (`loader`): Asset loading and ZIP extraction
//! 3. **Runtime Layer** (`plugin`, `animation`, `scene`): ECS components and systems

pub mod animation;
pub mod error;
pub mod loader;
pub mod plugin;
pub mod scene;
pub mod schema;

/// Prelude module for convenient imports.
pub mod prelude {
    pub use crate::animation::{AmAnimated, AmPlayback};
    pub use crate::error::AmError;
    pub use crate::loader::AmProject;
    pub use crate::plugin::{AlightMotionPlugin, load_am_project};
    pub use crate::scene::{
        AmLayerMarker, AmProjectBundle, AmProjectRoot, AmSceneConfig, am_to_bevy_coords,
    };
    pub use crate::schema::{
        AmAnimatedFloat, AmAnimatedVec2, AmAnimatedVec3, AmEmbedScene, AmKeyframe, AmLayer,
        AmMedia, AmNullObj, AmScene, AmShape, Easing,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_xml() {
        let xml = r##"<?xml version='1.0' encoding='UTF-8' ?>
        <scene title="Test Project" width="1280" height="960" fps="60" totalTime="2000" bgcolor="#ff000000">
            <media uri="amproj:test.png" filename="test.png" type="image/png" width="100" height="100" size="1234" />
            <shape id="123" label="Shape 1" startTime="0" endTime="1000" fillType="color" s=".rect">
                <transform lockAspectRatio="false">
                    <location value="640.0,480.0,0.0" />
                    <rotation value="45.0" />
                    <scale value="1.5,1.5" />
                    <opacity value="0.8" />
                </transform>
                <property name="size" type="vec2" value="100.0,100.0" />
            </shape>
            <nullobj id="456" label="Null 1" startTime="0" endTime="2000" type="perspective">
                <transform>
                    <location>
                        <kf t="0.0" v="0.0,0.0,0.0" />
                        <kf t="1.0" v="100.0,100.0,0.0" e="cubicBezier 0.0 0.0 0.58 1.0" />
                    </location>
                </transform>
            </nullobj>
        </scene>
        "##;

        let scene: schema::AmScene = quick_xml::de::from_str(xml).expect("Failed to parse XML");

        assert_eq!(scene.title, "Test Project");
        assert_eq!(scene.width, 1280);
        assert_eq!(scene.height, 960);
        assert_eq!(scene.fps, 60);
        assert_eq!(scene.total_time, 2000);
        assert_eq!(scene.media.len(), 1);
        assert_eq!(scene.layers.len(), 2);
    }

    #[test]
    fn test_keyframe_animation() {
        use animation::interpolate_float;
        use schema::{AmAnimatedFloat, AmKeyframe};

        // Test animation with easing
        let prop = AmAnimatedFloat {
            value: None,
            keyframes: vec![
                AmKeyframe {
                    time: 0.0,
                    value: "0.0".to_string(),
                    easing: Some("cubicBezier 0.42 0.0 0.58 1.0".to_string()),
                },
                AmKeyframe {
                    time: 1.0,
                    value: "100.0".to_string(),
                    easing: None,
                },
            ],
        };

        // Check boundaries
        let v0 = interpolate_float(&prop, 0.0).unwrap();
        assert!((v0 - 0.0).abs() < 0.1);

        let v1 = interpolate_float(&prop, 1.0).unwrap();
        assert!((v1 - 100.0).abs() < 0.1);

        // Middle should be affected by easing
        let v_mid = interpolate_float(&prop, 0.5).unwrap();
        assert!(v_mid > 0.0 && v_mid < 100.0);
    }

    #[test]
    fn test_coordinate_conversion() {
        use scene::{AmSceneConfig, am_to_bevy_coords};

        let config = AmSceneConfig::default();

        // Test center point
        let (x, y) = am_to_bevy_coords(640.0, 480.0, &config);
        assert!((x - 0.0).abs() < 0.01);
        assert!((y - 0.0).abs() < 0.01);
    }

    // Robustness tests for edge cases and missing fields

    #[test]
    fn test_parse_minimal_scene() {
        // Scene with only required structure, all optional fields missing
        let xml = r##"<?xml version='1.0' encoding='UTF-8' ?>
        <scene></scene>
        "##;

        let scene: schema::AmScene =
            quick_xml::de::from_str(xml).expect("Failed to parse minimal XML");
        assert_eq!(scene.title, "");
        assert_eq!(scene.width, 1280); // default
        assert_eq!(scene.height, 1280); // default
        assert_eq!(scene.fps, 60); // default
    }

    #[test]
    fn test_parse_shape_missing_transform() {
        // Shape without transform should use defaults
        let xml = r##"<?xml version='1.0' encoding='UTF-8' ?>
        <scene title="Test">
            <shape id="1" label="No Transform" startTime="0" endTime="1000" fillType="color" s=".rect">
            </shape>
        </scene>
        "##;

        let scene: schema::AmScene = quick_xml::de::from_str(xml).expect("Failed to parse XML");
        assert_eq!(scene.layers.len(), 1);
    }

    #[test]
    fn test_parse_empty_keyframes() {
        // Transform with empty keyframe list
        let xml = r##"<?xml version='1.0' encoding='UTF-8' ?>
        <scene title="Test">
            <shape id="1" label="Empty KF" startTime="0" endTime="1000" fillType="color" s=".rect">
                <transform>
                    <location />
                    <rotation />
                </transform>
            </shape>
        </scene>
        "##;

        let scene: schema::AmScene = quick_xml::de::from_str(xml).expect("Failed to parse XML");
        assert_eq!(scene.layers.len(), 1);
    }

    #[test]
    fn test_parse_partial_attributes() {
        // Scene with only some attributes set
        let xml = r##"<?xml version='1.0' encoding='UTF-8' ?>
        <scene title="Partial" width="800">
            <shape id="1">
                <transform />
            </shape>
        </scene>
        "##;

        let scene: schema::AmScene = quick_xml::de::from_str(xml).expect("Failed to parse XML");
        assert_eq!(scene.title, "Partial");
        assert_eq!(scene.width, 800);
        assert_eq!(scene.height, 1280); // default since not specified
    }

    #[test]
    fn test_easing_robustness() {
        use schema::Easing;

        // Unknown easing type should default to linear
        let e = Easing::parse("unknown 1 2 3");
        assert_eq!(e, Easing::Linear);

        // Incomplete cubicBezier should use defaults
        let e = Easing::parse("cubicBezier 0.5");
        match e {
            Easing::CubicBezier { x1, .. } => assert!((x1 - 0.5).abs() < 0.01),
            _ => panic!("Expected CubicBezier"),
        }

        // Empty easing
        let e = Easing::parse("");
        assert_eq!(e, Easing::Linear);

        // Whitespace only
        let e = Easing::parse("   ");
        assert_eq!(e, Easing::Linear);
    }

    #[test]
    fn test_animation_outside_bounds() {
        use animation::interpolate_float;
        use schema::{AmAnimatedFloat, AmKeyframe};

        // Keyframes that don't cover 0-1 range
        let prop = AmAnimatedFloat {
            value: None,
            keyframes: vec![
                AmKeyframe {
                    time: 0.2,
                    value: "50.0".to_string(),
                    easing: None,
                },
                AmKeyframe {
                    time: 0.8,
                    value: "100.0".to_string(),
                    easing: None,
                },
            ],
        };

        // Before first keyframe
        let v = interpolate_float(&prop, 0.0).unwrap();
        assert!(
            (v - 50.0).abs() < 0.1,
            "Before first kf should hold first value"
        );

        // After last keyframe
        let v = interpolate_float(&prop, 1.0).unwrap();
        assert!(
            (v - 100.0).abs() < 0.1,
            "After last kf should hold last value"
        );
    }
}
