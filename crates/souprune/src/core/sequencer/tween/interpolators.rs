//! Defines custom interpolators and bookkeeping components used by sequence-driven tweens.
//!
//! 定义由序列驱动 tween 使用的自定义插值器与运行时标记组件。
//!
//! Bevy Tween already covers common transform cases, but some sequence chapter
//! targets need custom interpolation state such as `ViewBox` sizing or sprite
//! alpha. This file isolates those tween primitives and the marker used to keep
//! track of in-flight tweens that chapters may need to wait on.
//!
//! Bevy Tween 已经覆盖了常见的 transform 情况，但某些序列章节目标还需要
//! 自定义插值状态，例如 `ViewBox` 尺寸或 sprite alpha。这个文件把这些 tween
//! 原语，以及章节等待中的 tween 跟踪标记单独收拢起来。

use crate::core::view::components::ViewBox;
use bevy::prelude::*;
use bevy_tween::interpolate::Interpolator;

#[derive(Debug, Clone, PartialEq, Reflect)]
pub(crate) struct ViewBoxSizeInterpolator {
    pub start_width: f32,
    pub start_height: f32,
    pub end_width: f32,
    pub end_height: f32,
}

impl Interpolator for ViewBoxSizeInterpolator {
    type Item = ViewBox;

    fn interpolate(&self, item: &mut Self::Item, value: f32, _previous_value: f32) {
        item.width = self.start_width + (self.end_width - self.start_width) * value;
        item.height = self.start_height + (self.end_height - self.start_height) * value;
    }
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct SpriteAlphaInterpolator {
    pub start: f32,
    pub end: f32,
}

impl Interpolator for SpriteAlphaInterpolator {
    type Item = Sprite;

    fn interpolate(&self, item: &mut Self::Item, value: f32, _previous_value: f32) {
        let alpha = self.start + (self.end - self.start) * value;
        item.color = item.color.with_alpha(alpha);
    }
}

#[derive(Component)]
pub struct TweenInProgress {
    pub wait_for_completion: bool,
    pub animator_entity: Entity,
}
