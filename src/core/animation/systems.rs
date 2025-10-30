use crate::core::animation::SpriteAnimationClip;
use bevy::prelude::{Query, Res, Time, With};
use bevy::sprite::Sprite;

pub(crate) fn update_walking_system(
    time: Res<Time>,
    mut query: Query<(&mut Sprite,), With<SpriteAnimationClip>>,
) {
    for (mut sprite,) in query.iter_mut() {
        // 更新 sprite 和动画状态
    }
}