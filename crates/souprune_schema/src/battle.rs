//! # battle.rs
//!
//! BattlePlayerConfig schema types for `.battle_player.ron` files.
//! Mirrors `souprune::app_state::battle::player_config_schema` without Bevy dependency.
//!
//! `.battle_player.ron` 文件的战斗玩家配置 Schema 类型。

use crate::bevy_types::BevyColor;
use serde::{Deserialize, Serialize};

/// Collider shape for battle entities.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum BattleColliderShape {
    Circle { radius: f32 },
    Box { half_size: (f32, f32) },
}

/// Collider configuration with debug visualization offset.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ColliderConfig {
    pub shape: BattleColliderShape,
    pub debug_z_offset: f32,
}

/// Invincibility configuration for battle damage behavior.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BattleInvincibilityConfig {
    #[serde(default = "default_invincibility_duration")]
    pub duration: f32,
    #[serde(default = "default_flash_interval")]
    pub flash_interval: f32,
    #[serde(default = "default_normal_color")]
    pub normal_color: BevyColor,
    #[serde(default = "default_flash_color")]
    pub flash_color: BevyColor,
    #[serde(default)]
    pub damage_sound: Option<String>,
}

impl Default for BattleInvincibilityConfig {
    fn default() -> Self {
        Self {
            duration: default_invincibility_duration(),
            flash_interval: default_flash_interval(),
            normal_color: default_normal_color(),
            flash_color: default_flash_color(),
            damage_sound: None,
        }
    }
}

/// Battle enemy speech bubble request data.
///
/// 战斗敌人对话气泡请求数据。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct BattleSpeechBubbleDef {
    /// Dialogue channel used by the bubble.
    ///
    /// 气泡使用的对话通道。
    #[serde(default = "default_enemy_speech_channel")]
    pub channel: String,
    /// Mortar script path to start.
    ///
    /// 要启动的 Mortar 脚本路径。
    pub mortar_path: String,
    /// Mortar node to start.
    ///
    /// 要启动的 Mortar 节点。
    pub mortar_node: String,
    /// Visual frame profile.
    ///
    /// 视觉气泡框配置。
    #[serde(default)]
    pub frame: BattleSpeechBubbleFrame,
    /// How the bubble advances after it starts.
    ///
    /// 气泡启动后的推进方式。
    #[serde(default)]
    pub advance: BattleSpeechBubbleAdvance,
    /// Whether the bubble hides after the dialogue finishes.
    ///
    /// 对话结束后是否隐藏气泡。
    #[serde(default = "default_true")]
    pub hide_on_finish: bool,
    /// Optional typewriter voice path override.
    ///
    /// 可选的打字机语音路径覆盖。
    #[serde(default)]
    pub voice: Option<String>,
    /// Optional typewriter speed override.
    ///
    /// 可选的打字机速度覆盖。
    #[serde(default)]
    pub typewriter_speed: Option<f32>,
}

/// Battle speech bubble visual frame.
///
/// 战斗对话气泡视觉框。
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
pub enum BattleSpeechBubbleFrame {
    /// Wide Mad Dummy-style bubble frame.
    ///
    /// Mad Dummy 风格的宽气泡框。
    #[default]
    MadDummyWide,
}

/// Battle speech bubble advance behavior.
///
/// 战斗对话气泡推进行为。
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Default)]
pub enum BattleSpeechBubbleAdvance {
    /// Player confirmation advances the Mortar dialogue.
    ///
    /// 由玩家确认键推进 Mortar 对话。
    #[default]
    Manual,
    /// A timer hides the bubble without focus.
    ///
    /// 由计时器隐藏无焦点气泡。
    Timed {
        /// Duration in seconds.
        ///
        /// 持续秒数。
        duration: f32,
    },
}

/// Battle player configuration — top-level `.battle_player.ron` schema.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BattlePlayerConfig {
    pub sprite_path: String,
    pub color: BevyColor,
    pub physics_collider: ColliderConfig,
    pub damage_trigger: ColliderConfig,
    pub z_position: f32,
    pub default_mode_id: String,
    pub speed: f32,
    pub focus_speed_ratio: f32,
    #[serde(default = "default_box_id")]
    pub default_box: String,
    #[serde(default)]
    pub invincibility: BattleInvincibilityConfig,
}

// ============================================================================
// Default helpers
// ============================================================================

fn default_invincibility_duration() -> f32 {
    1.0
}

fn default_flash_interval() -> f32 {
    0.25
}

fn default_normal_color() -> BevyColor {
    BevyColor::Srgba(crate::bevy_types::SrgbaColor {
        red: 1.0,
        green: 0.0,
        blue: 0.0,
        alpha: 1.0,
    })
}

fn default_flash_color() -> BevyColor {
    BevyColor::Srgba(crate::bevy_types::SrgbaColor {
        red: 0.5,
        green: 0.0,
        blue: 0.0,
        alpha: 1.0,
    })
}

fn default_box_id() -> String {
    "main".to_string()
}

fn default_enemy_speech_channel() -> String {
    "battle_enemy_speech".to_string()
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_battle_player_config_with_default_box() {
        let ron = r#"(
            sprite_path: "assets/textures/common/view/heart.png",
            color: Srgba((red: 1.0, green: 0.0, blue: 0.0, alpha: 1.0)),
            physics_collider: (
                shape: Circle(radius: 8.0),
                debug_z_offset: 10.0,
            ),
            damage_trigger: (
                shape: Box(half_size: (2.0, 2.0)),
                debug_z_offset: 12.0,
            ),
            z_position: 10.0,
            default_mode_id: "soul_red",
            speed: 150.0,
            focus_speed_ratio: 0.5,
        )"#;

        let config: BattlePlayerConfig = ron::from_str(ron).expect("battle player config");

        assert_eq!(config.default_box, "main");
        match config.damage_trigger.shape {
            BattleColliderShape::Box { half_size } => assert_eq!(half_size, (2.0, 2.0)),
            other => panic!("unexpected collider parsed: {other:?}"),
        }
    }

    #[test]
    fn parses_battle_speech_bubble_with_manual_advance() {
        let ron = r#"(
            channel: "battle_enemy_speech",
            mortar_path: "battle/enemies/mad_dummy.mortar",
            mortar_node: "enemy_speech_manual_intro",
            frame: MadDummyWide,
            advance: Manual,
            hide_on_finish: true,
        )"#;

        let request: BattleSpeechBubbleDef =
            ron::from_str(ron).expect("battle speech bubble should parse");

        assert_eq!(request.channel, "battle_enemy_speech");
        assert_eq!(request.mortar_node, "enemy_speech_manual_intro");
        assert_eq!(request.frame, BattleSpeechBubbleFrame::MadDummyWide);
        assert_eq!(request.advance, BattleSpeechBubbleAdvance::Manual);
        assert!(request.hide_on_finish);
    }

    #[test]
    fn parses_battle_speech_bubble_with_timed_advance() {
        let ron = r#"(
            channel: "battle_enemy_speech",
            mortar_path: "battle/enemies/mad_dummy.mortar",
            mortar_node: "enemy_speech_timed_wave",
            frame: MadDummyWide,
            advance: Timed(duration: 2.0),
        )"#;

        let request: BattleSpeechBubbleDef =
            ron::from_str(ron).expect("timed battle speech bubble should parse");

        assert_eq!(
            request.advance,
            BattleSpeechBubbleAdvance::Timed { duration: 2.0 }
        );
        assert!(request.hide_on_finish);
    }
}
