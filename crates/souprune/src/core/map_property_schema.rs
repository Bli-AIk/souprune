//! # map_property_schema.rs
//!
//! # 地图属性 Schema 定义
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module defines the schema for Tiled map properties used by the engine.
//! It provides centralized property key definitions, validation, and documentation.
//!
//! 本模块定义了引擎使用的 Tiled 地图属性 Schema。
//! 它提供了集中的属性键定义、验证和文档。
//!
//! ## Map Property Reference
//!
//! ## 地图属性参考
//!
//! | Property Key | Type | Required | Description |
//! |--------------|------|----------|-------------|
//! | `backpack_ui` | String | No | Path to view layout RON file for UI (fallback, states.ron takes priority) |
//! | `bgm` | String | No | Path to background music file |
//! | `rules_file` | String | No | Path to FRE rules file for this map |
//!
//! ## Object Property Reference
//!
//! ## 对象属性参考
//!
//! | Property Key | Type | Required | Description |
//! |--------------|------|----------|-------------|
//! | `collision` | String | No | Collision type ("solid", "semi-solid", etc.) |
//! | `trigger` | String | No | Trigger zone type |
//! | `trigger_id` | String | No | Unique identifier for the trigger |

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::tiled;
use std::collections::HashMap;

/// Map property keys used by the engine.
///
/// 引擎使用的地图属性键。
pub mod keys {
    /// Path to view layout RON file for UI.
    /// Note: `states.ron` view_layout takes priority over this property.
    ///
    /// UI 视图布局 RON 文件路径。
    /// 注意：`states.ron` 的 view_layout 优先于此属性。
    pub const BACKPACK_UI: &str = "backpack_ui";

    /// Path to background music file.
    ///
    /// 背景音乐文件路径。
    pub const BGM: &str = "bgm";

    /// Path to FRE rules file for this map.
    ///
    /// 此地图的 FRE 规则文件路径。
    pub const RULES_FILE: &str = "rules_file";
}

/// Object property keys used by the engine.
///
/// 引擎使用的对象属性键。
pub mod object_keys {
    /// Collision type for the object.
    ///
    /// 对象的碰撞类型。
    pub const COLLISION: &str = "collision";

    /// Trigger zone type.
    ///
    /// 触发区域类型。
    pub const TRIGGER: &str = "trigger";
}

/// Property definition for validation purposes.
///
/// 用于验证的属性定义。
#[derive(Debug, Clone)]
pub struct PropertyDef {
    /// The property key name.
    pub key: &'static str,
    /// Human-readable description.
    pub description: &'static str,
    /// Whether this property is required.
    pub required: bool,
    /// Default value if the property is missing (None means no default).
    pub default: Option<&'static str>,
}

/// All map-level properties.
///
/// 所有地图级别的属性。
pub static MAP_PROPERTIES: &[PropertyDef] = &[
    PropertyDef {
        key: keys::BACKPACK_UI,
        description: "Path to view layout RON file for UI (states.ron view_layout takes priority)",
        required: false,
        default: None,
    },
    PropertyDef {
        key: keys::BGM,
        description: "Path to background music file",
        required: false,
        default: None,
    },
    PropertyDef {
        key: keys::RULES_FILE,
        description: "Path to FRE rules file for this map",
        required: false,
        default: None,
    },
];

/// All object-level properties.
///
/// 所有对象级别的属性。
pub static OBJECT_PROPERTIES: &[PropertyDef] = &[
    PropertyDef {
        key: object_keys::COLLISION,
        description: "Collision type for the object (solid, semi-solid, etc.)",
        required: false,
        default: None,
    },
    PropertyDef {
        key: object_keys::TRIGGER,
        description: "Trigger zone type",
        required: false,
        default: None,
    },
];

/// Validate map properties and log warnings for unknown properties.
///
/// 验证地图属性并对未知属性记录警告。
pub fn validate_map_properties(properties: &tiled::Properties) {
    let known_keys: std::collections::HashSet<&str> =
        MAP_PROPERTIES.iter().map(|p| p.key).collect();

    for (key, _value) in properties.iter() {
        if !known_keys.contains(key.as_str()) {
            debug!(
                "Unknown map property '{}' - this may be intentional for custom mod functionality",
                key
            );
        }
    }

    // Check for required properties (currently none are required)
    for prop_def in MAP_PROPERTIES {
        if prop_def.required && !properties.contains_key(prop_def.key) {
            warn!(
                "Required map property '{}' is missing: {}",
                prop_def.key, prop_def.description
            );
        }
    }
}

/// Get a string property from the map, with optional validation logging.
///
/// 从地图获取字符串属性，带有可选的验证日志。
pub fn get_string_property<'a>(properties: &'a tiled::Properties, key: &str) -> Option<&'a str> {
    properties.get(key).and_then(|v| {
        if let tiled::PropertyValue::StringValue(s) = v {
            Some(s.as_str())
        } else {
            warn!(
                "Map property '{}' has unexpected type, expected String",
                key
            );
            None
        }
    })
}

/// Get a bool property from the map.
///
/// 从地图获取布尔属性。
pub fn get_bool_property(properties: &tiled::Properties, key: &str) -> Option<bool> {
    properties.get(key).and_then(|v| {
        if let tiled::PropertyValue::BoolValue(b) = v {
            Some(*b)
        } else {
            warn!("Map property '{}' has unexpected type, expected Bool", key);
            None
        }
    })
}

/// Get a bool property from an object's properties HashMap.
///
/// 从对象的属性 HashMap 获取布尔属性。
pub fn get_object_bool_property(
    properties: &std::collections::HashMap<String, tiled::PropertyValue>,
    key: &str,
) -> Option<bool> {
    properties.get(key).and_then(|v| {
        if let tiled::PropertyValue::BoolValue(b) = v {
            Some(*b)
        } else {
            debug!(
                "Object property '{}' has unexpected type, expected Bool",
                key
            );
            None
        }
    })
}

/// Get a string property from an object's properties HashMap.
///
/// 从对象的属性 HashMap 获取字符串属性。
pub fn get_object_string_property<'a>(
    properties: &'a std::collections::HashMap<String, tiled::PropertyValue>,
    key: &str,
) -> Option<&'a str> {
    properties.get(key).and_then(|v| {
        if let tiled::PropertyValue::StringValue(s) = v {
            Some(s.as_str())
        } else {
            debug!(
                "Object property '{}' has unexpected type, expected String",
                key
            );
            None
        }
    })
}

/// Get a string property with a default fallback.
///
/// 获取字符串属性，带有默认值回退。
pub fn get_string_property_or_default<'a>(
    properties: &'a tiled::Properties,
    key: &str,
    default: &'a str,
) -> &'a str {
    get_string_property(properties, key).unwrap_or(default)
}

/// Validate object properties and log warnings for unknown properties.
///
/// 验证对象属性并对未知属性记录警告。
pub fn validate_object_properties(properties: &HashMap<String, tiled::PropertyValue>) {
    let known_keys: std::collections::HashSet<&str> =
        OBJECT_PROPERTIES.iter().map(|p| p.key).collect();

    for (key, _value) in properties.iter() {
        if !known_keys.contains(key.as_str()) {
            debug!(
                "Unknown object property '{}' - this may be intentional for custom mod functionality",
                key
            );
        }
    }

    // Check for required properties (currently none are required)
    for prop_def in OBJECT_PROPERTIES {
        if prop_def.required && !properties.contains_key(prop_def.key) {
            warn!(
                "Required object property '{}' is missing: {}",
                prop_def.key, prop_def.description
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_keys_are_unique() {
        let map_keys: Vec<&str> = MAP_PROPERTIES.iter().map(|p| p.key).collect();
        let mut unique_keys = map_keys.clone();
        unique_keys.sort();
        unique_keys.dedup();
        assert_eq!(
            map_keys.len(),
            unique_keys.len(),
            "Duplicate map property keys found"
        );

        let obj_keys: Vec<&str> = OBJECT_PROPERTIES.iter().map(|p| p.key).collect();
        let mut unique_obj_keys = obj_keys.clone();
        unique_obj_keys.sort();
        unique_obj_keys.dedup();
        assert_eq!(
            obj_keys.len(),
            unique_obj_keys.len(),
            "Duplicate object property keys found"
        );
    }
}
