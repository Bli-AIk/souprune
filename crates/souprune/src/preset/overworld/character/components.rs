//! # components.rs
//!
//! # components.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module defines components for character states and animation transitions.
//!
//! 本模块定义角色状态和动画转换的组件。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It includes components for idle, walking, and running states.
//!
//! 包括空闲、行走和奔跑状态的组件。

use crate::core::mode::ModeScoped;
use crate::core::animation::components::SpriteAnimationClip;
use crate::core::basic_components::{Direction, Facing, Speed};
use crate::core::character_asset::CharacterAnimator;
use bevy::prelude::*;

#[derive(Component)]
pub struct PlayerControlled;

#[derive(Clone, Component)]
#[component(storage = "SparseSet")]
pub(crate) struct StateIdle;

#[derive(Clone, Component)]
#[component(storage = "SparseSet")]
pub(crate) struct StateWalking;

#[derive(Clone, Component)]
#[component(storage = "SparseSet")]
pub(crate) struct StateRunning;

/// Bundle for all Overworld characters (player and NPCs).
///
/// 所有 Overworld 角色（玩家和 NPC）的组件包。
#[derive(Bundle)]
pub(crate) struct CharacterBundle {
    pub mode_scoped: ModeScoped,
    pub facing: Facing,
    pub speed: Speed,
    pub sprite: Sprite,
    pub anim_clip: SpriteAnimationClip,
    pub animator: CharacterAnimator,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl CharacterBundle {
    pub fn new(
        spawn_pos: Vec2,
        facing: Direction,
        speed: f32,
        anim_clip: SpriteAnimationClip,
        animator: CharacterAnimator,
    ) -> Self {
        Self {
            mode_scoped: ModeScoped("overworld".to_string()),
            facing: Facing { value: facing },
            speed: Speed { value: speed },
            sprite: Sprite::default(),
            anim_clip,
            animator,
            transform: Transform::from_translation(spawn_pos.extend(0.0)),
            global_transform: GlobalTransform::default(),
        }
    }
}
