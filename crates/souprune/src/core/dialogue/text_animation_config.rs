//! Text animation configuration resource.
//!
//! 文本动画配置资源。
//!
//! Wraps [`TextAnimationConfigDef`] from the schema crate. Follows the same pattern as
//! [`VoiceConfig`] and [`AutoPauseConfig`] — a newtype resource populated at startup
//! from `narrative/dialogue.ron`.
//!
//! 包装来自 schema crate 的 [`TextAnimationConfigDef`]。遵循与 [`VoiceConfig`] 和
//! [`AutoPauseConfig`] 相同的模式 — 启动时从 `narrative/dialogue.ron` 填充的 newtype 资源。

use std::ops::{Deref, DerefMut};

use bevy::prelude::*;
use bevy_fact_rule_event::LayeredFactDatabase;
use souprune_schema::dialogue::{TextAnimationConfigDef, TextAnimationPresetDef};

use crate::core::fre_facts;

/// Resource wrapping the deserialized text animation config.
///
/// 包装反序列化后的文本动画配置的资源。
#[derive(Resource, Debug, Clone, Default)]
pub struct TextAnimationConfig(pub TextAnimationConfigDef);

impl Deref for TextAnimationConfig {
    type Target = TextAnimationConfigDef;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TextAnimationConfig {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TextAnimationConfig {
    /// Returns the preset for `name`, falling back to `default_preset`.
    ///
    /// 返回 `name` 对应的预设，回退到 `default_preset`。
    pub fn resolve_preset(&self, name: Option<&str>) -> Option<&TextAnimationPresetDef> {
        let key = name
            .filter(|n| !n.is_empty())
            .unwrap_or(&self.default_preset);
        self.presets.get(key)
    }

    /// Returns the effective preset name for a dialogue channel.
    ///
    /// 返回对话通道的实际预设名称。
    pub fn resolve_channel_preset_name(
        &self,
        facts: &LayeredFactDatabase,
        channel: &str,
    ) -> Option<String> {
        facts
            .get_string(&fre_facts::dialogue_channel_key(
                channel,
                fre_facts::DIALOGUE_TEXT_STYLE_FIELD,
            ))
            .filter(|name| !name.is_empty())
            .or_else(|| {
                facts
                    .get_string(fre_facts::DIALOGUE_TEXT_STYLE)
                    .filter(|name| !name.is_empty())
            })
            .map(str::to_string)
            .or_else(|| (!self.default_preset.is_empty()).then(|| self.default_preset.clone()))
    }

    /// Returns the effective preset for a dialogue channel.
    ///
    /// 返回对话通道的实际预设。
    pub fn resolve_channel_preset(
        &self,
        facts: &LayeredFactDatabase,
        channel: &str,
    ) -> Option<&TextAnimationPresetDef> {
        let preset_name = self.resolve_channel_preset_name(facts, channel);
        self.resolve_preset(preset_name.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use bevy_fact_rule_event::{FactValue, LayeredFactDatabase};
    use souprune_schema::dialogue::{
        TextAnimationConfigDef, TextAnimationPresetDef, TextDisplayDef,
    };

    use super::*;
    use crate::core::fre_facts;

    fn preset(display: TextDisplayDef) -> TextAnimationPresetDef {
        TextAnimationPresetDef {
            display,
            shake: None,
            wave: None,
        }
    }

    fn config() -> TextAnimationConfig {
        TextAnimationConfig(TextAnimationConfigDef {
            default_preset: "default".into(),
            presets: [
                ("default".into(), preset(TextDisplayDef::Normal)),
                ("global".into(), preset(TextDisplayDef::Normal)),
                ("channel".into(), preset(TextDisplayDef::Normal)),
            ]
            .into_iter()
            .collect(),
        })
    }

    #[test]
    fn channel_text_style_overrides_global_text_style() {
        let mut facts = LayeredFactDatabase::new();
        facts.set_global(
            fre_facts::DIALOGUE_TEXT_STYLE,
            FactValue::String("global".into()),
        );
        facts.set_local(
            fre_facts::dialogue_channel_key("main", fre_facts::DIALOGUE_TEXT_STYLE_FIELD),
            FactValue::String("channel".into()),
        );

        assert_eq!(
            config()
                .resolve_channel_preset_name(&facts, "main")
                .as_deref(),
            Some("channel")
        );
    }

    #[test]
    fn global_text_style_applies_when_channel_has_no_override() {
        let mut facts = LayeredFactDatabase::new();
        facts.set_global(
            fre_facts::DIALOGUE_TEXT_STYLE,
            FactValue::String("global".into()),
        );

        assert_eq!(
            config()
                .resolve_channel_preset_name(&facts, "main")
                .as_deref(),
            Some("global")
        );
    }

    #[test]
    fn empty_text_style_uses_default_preset() {
        let mut facts = LayeredFactDatabase::new();
        facts.set_global(
            fre_facts::DIALOGUE_TEXT_STYLE,
            FactValue::String(String::new()),
        );

        assert_eq!(
            config()
                .resolve_channel_preset_name(&facts, "main")
                .as_deref(),
            Some("default")
        );
    }

    #[test]
    fn resolves_channel_preset_from_fact_fallback_chain() {
        let mut facts = LayeredFactDatabase::new();
        facts.set_global(
            fre_facts::DIALOGUE_TEXT_STYLE,
            FactValue::String("global".into()),
        );

        let config = config();
        let resolved = config.resolve_channel_preset(&facts, "main").unwrap();

        assert!(matches!(resolved.display, TextDisplayDef::Normal));
    }
}
