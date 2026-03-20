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
