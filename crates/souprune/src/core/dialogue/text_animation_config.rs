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
use souprune_schema::dialogue::{TextAnimationConfigDef, TextAnimationPresetDef};

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
}
