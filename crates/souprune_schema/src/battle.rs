//! # battle.rs
//!
//! Battle-specific shared data schema types.
//!
//! 战斗相关共享数据 Schema 类型。

use serde::{Deserialize, Serialize};

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
    /// Optional text animation style preset override.
    ///
    /// 可选的文本动画风格预设覆盖。
    #[serde(default)]
    pub text_style: Option<String>,
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

    #[test]
    fn parses_battle_speech_bubble_text_style_as_explicit_override() {
        let ron = r#"(
            mortar_path: "battle/enemies/mad_dummy.mortar",
            mortar_node: "enemy_speech_manual_intro",
            text_style: Some("mad_dummy"),
        )"#;

        let request: BattleSpeechBubbleDef =
            ron::from_str(ron).expect("battle speech bubble should parse");

        assert_eq!(request.text_style.as_deref(), Some("mad_dummy"));
    }

    #[test]
    fn battle_speech_bubble_text_style_defaults_to_none() {
        let ron = r#"(
            mortar_path: "battle/enemies/mad_dummy.mortar",
            mortar_node: "enemy_speech_manual_intro",
        )"#;

        let request: BattleSpeechBubbleDef =
            ron::from_str(ron).expect("battle speech bubble should parse");

        assert_eq!(request.text_style, None);
    }
}
