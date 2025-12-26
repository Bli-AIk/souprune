//! # resources.rs
//!
//! # resources.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Defines `PlayerInputSettings`, a resource that manages input mappings (keyboard, gamepad) for player actions, supporting multiple control schemes.
//!
//! 定义 `PlayerInputSettings`，该资源管理玩家动作的输入映射（键盘、手柄），支持多种控制方案。

use super::actions::Action;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Resource)]
pub(crate) struct PlayerInputSettings {
    maps: Vec<InputMap<Action>>,
}

impl PlayerInputSettings {
    #[allow(dead_code)]
    pub fn get_map(&self, index: usize) -> Option<&InputMap<Action>> {
        self.maps.get(index)
    }

    pub fn get_merged_map(&self) -> InputMap<Action> {
        let mut merged = InputMap::default();

        for map in &self.maps {
            merged.merge(map);
        }

        merged
    }
}

impl Default for PlayerInputSettings {
    fn default() -> Self {
        use Action::*;
        use KeyCode::*;
        let mut map_key_default = InputMap::default();

        map_key_default.insert(Up, ArrowUp);
        map_key_default.insert(Down, ArrowDown);
        map_key_default.insert(Left, ArrowLeft);
        map_key_default.insert(Right, ArrowRight);
        map_key_default.insert(Confirm, KeyZ);
        map_key_default.insert(Cancel, KeyX);
        map_key_default.insert(Menu, KeyC);

        let mut map_key_alternate_0 = InputMap::default();

        map_key_alternate_0.insert(Up, KeyW);
        map_key_alternate_0.insert(Down, KeyS);
        map_key_alternate_0.insert(Left, KeyA);
        map_key_alternate_0.insert(Right, KeyD);
        map_key_alternate_0.insert(Confirm, Enter);
        map_key_alternate_0.insert(Cancel, ShiftLeft);
        map_key_alternate_0.insert(Menu, ControlLeft);

        let mut map_key_alternate_1 = InputMap::default();

        map_key_alternate_1.insert(Cancel, ShiftRight);
        map_key_alternate_1.insert(Menu, ControlRight);

        let mut map_gamepad_default = InputMap::default();
        use GamepadButton::*;
        map_gamepad_default.insert(Up, DPadUp);
        map_gamepad_default.insert(Down, DPadDown);
        map_gamepad_default.insert(Left, DPadLeft);
        map_gamepad_default.insert(Right, DPadRight);
        map_gamepad_default.insert(Confirm, South);
        map_gamepad_default.insert(Cancel, East);
        map_gamepad_default.insert(Menu, North);

        Self {
            maps: vec![
                map_key_default,
                map_key_alternate_0,
                map_key_alternate_1,
                map_gamepad_default,
            ],
        }
    }
}
