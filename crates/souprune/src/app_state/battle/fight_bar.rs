//! Fight bar sweep and Z-key interaction.
//!
//! 攻击条滑动与 Z 键交互。
//!
//! Drives the attack bar across the target during FIGHT mode.
//! Each frame the bar advances at a constant speed matching the
//! Undertale original (11 px/frame at 30 fps ≈ 330 px/s).
//! When the player presses Confirm (Z) the bar freezes and
//! `fight:confirmed` is set; if the bar reaches the far edge
//! without input it counts as a miss.
//! After a hit the bar colors flash (black↔white swap at ~6 Hz)
//! until the collapse phase begins.
//!
//! 在 FIGHT 模式下驱动攻击条从左向右滑过目标。
//! 每帧以与 Undertale 原作一致的恒定速度推进攻击条
//! （30 fps 下 11 px/帧 ≈ 330 px/s）。
//! 玩家按下确认键（Z）时攻击条冻结并设置 `fight:confirmed`；
//! 若攻击条到达右端而无输入，则视为 Miss。
//! 命中后攻击条颜色闪烁（黑白色互换，约 6 Hz）直到收尾阶段开始。

use bevy::prelude::*;
use bevy_alight_motion::sdf_material::SdfMaterial;
use bevy_fact_rule_event::LayeredFactDatabase;
use bevy_kira_audio::Audio;
use leafwing_input_manager::action_state::ActionState;
use std::collections::VecDeque;

use crate::core::battle_runtime::BattleInputManager;
use crate::core::input::{Action, ActionRegistry, ActionStateExt};
use crate::core::view::components::ViewElement;
use crate::core::view::sdf_shape::ViewSdfShape;

/// Attack-bar sweep speed in pixels per second (UT: 11 px/frame × 30 fps).
const BAR_SPEED: f32 = 330.0;

/// Right-edge X coordinate where the bar stops (DumbTarget right boundary).
const BAR_RIGHT_EDGE: f32 = 272.0;

/// Seconds between color swaps when bar is flashing (~6 Hz cycle).
const FLASH_INTERVAL: f32 = 0.083;

/// Tracks the flash animation timer and locked bar position.
///
/// 闪烁动画计时器和锁定的攻击条位置。
///
/// `locked_x` stores the bar's X position while the sweep or flash
/// is active. A `PostUpdate` system re-applies this position every
/// frame so the view reconciliation system cannot reset it.
#[derive(Resource, Default)]
pub struct FightBarFlashTimer {
    elapsed: f32,
    active: bool,
    /// Bar X position to enforce after reconciliation.
    pub(crate) locked_x: Option<f32>,
    /// `Some(true)` = hit, `Some(false)` = miss. Consumed by fact sync.
    pending_completion: Option<bool>,
}

/// Moves the attack bar every frame and handles Confirm input.
///
/// 每帧移动攻击条并处理确认键输入。
pub fn fight_bar_sweep_system(
    fact_db: Res<LayeredFactDatabase>,
    action_query: Query<&ActionState<Action>, With<BattleInputManager>>,
    registry: Res<ActionRegistry>,
    time: Res<Time>,
    audio: Res<Audio>,
    asset_server: Res<AssetServer>,
    view_elements: Query<(Entity, &ViewElement)>,
    mut transforms: Query<&mut Transform>,
    mut flash_timer: ResMut<FightBarFlashTimer>,
) {
    if !fact_db.get_bool("fight:bar_active").unwrap_or(false) {
        return;
    }

    let Ok(action_state) = action_query.single() else {
        return;
    };

    let Some((bar_entity, _)) = view_elements
        .iter()
        .find(|(_, elem)| elem.local_name == "AttackBar")
    else {
        return;
    };

    let Ok(mut transform) = transforms.get_mut(bar_entity) else {
        return;
    };

    // Restore position if reconciliation reset it.
    if let Some(lx) = flash_timer.locked_x {
        transform.translation.x = lx;
    }

    // Advance bar position.
    transform.translation.x += BAR_SPEED * time.delta_secs();
    flash_timer.locked_x = Some(transform.translation.x);

    if transform.translation.x >= BAR_RIGHT_EDGE {
        transform.translation.x = BAR_RIGHT_EDGE;
        flash_timer.locked_x = Some(BAR_RIGHT_EDGE);
        flash_timer.pending_completion = Some(false);
        return;
    }

    if action_state.action_just_pressed(&registry, "Confirm") {
        flash_timer.active = true;
        flash_timer.elapsed = 0.0;
        flash_timer.pending_completion = Some(true);
        crate::core::audio::play_sound(&audio, &asset_server, "slice");
    }
}

/// Sets fight bar facts when the bar reaches the end or Z is pressed.
/// Runs in `PostUpdate` so the facts are set AFTER reconciliation,
/// preventing the reconciliation from resetting the bar's transform.
///
/// 攻击条到达终点或按下 Z 时设置 fact。
/// 在 `PostUpdate` 中运行，在 reconciliation 之后设置 fact，
/// 防止 reconciliation 重置攻击条的 transform。
pub fn fight_bar_fact_sync_system(
    mut fact_db: ResMut<LayeredFactDatabase>,
    mut flash_timer: ResMut<FightBarFlashTimer>,
) {
    let Some(confirmed) = flash_timer.pending_completion.take() else {
        return;
    };

    fact_db.set("fight:bar_active", false);
    fact_db.set("fight:bar_done", true);
    fact_db.set("fight:confirmed", confirmed);
}

/// Re-applies the locked bar position after reconciliation.
/// Runs in `PostUpdate` to override any position reset.
///
/// 在 reconciliation 之后重新应用锁定的攻击条位置。
/// 在 `PostUpdate` 中运行以覆盖任何位置重置。
pub fn fight_bar_position_restore_system(
    flash_timer: Res<FightBarFlashTimer>,
    view_elements: Query<(Entity, &ViewElement)>,
    mut transforms: Query<&mut Transform>,
) {
    let Some(locked_x) = flash_timer.locked_x else {
        return;
    };

    let Some((bar_entity, _)) = view_elements
        .iter()
        .find(|(_, elem)| elem.local_name == "AttackBar")
    else {
        return;
    };

    let Ok(mut transform) = transforms.get_mut(bar_entity) else {
        return;
    };

    transform.translation.x = locked_x;
}

/// Swaps the attack bar SDF colors (black↔white) at a fixed rate after a hit.
///
/// 命中后以固定频率交换攻击条 SDF 颜色（黑↔白）。
pub fn fight_bar_flash_system(
    time: Res<Time>,
    fact_db: Res<LayeredFactDatabase>,
    view_elements: Query<(Entity, &ViewElement)>,
    children_query: Query<&Children>,
    sdf_shapes: Query<&MeshMaterial2d<SdfMaterial>, With<ViewSdfShape>>,
    mut sdf_materials: ResMut<Assets<SdfMaterial>>,
    mut flash_timer: ResMut<FightBarFlashTimer>,
) {
    if !flash_timer.active {
        return;
    }

    // Stop flashing when fight target is hidden (collapse phase finished).
    if !fact_db.get_bool("fight_target_visible").unwrap_or(false) {
        flash_timer.active = false;
        flash_timer.locked_x = None;
        return;
    }

    let Some((bar_entity, _)) = view_elements
        .iter()
        .find(|(_, elem)| elem.local_name == "AttackBar")
    else {
        return;
    };

    flash_timer.elapsed += time.delta_secs();

    let cycle = (flash_timer.elapsed / FLASH_INTERVAL) as u32;
    let inverted = !cycle.is_multiple_of(2);

    // BFS to collect SDF shape children in spawn order (outer first, then inner).
    let Ok(children) = children_query.get(bar_entity) else {
        return;
    };
    let mut queue = VecDeque::from_iter(children.iter());
    let mut sdf_entities = Vec::new();
    while let Some(child) = queue.pop_front() {
        if sdf_shapes.contains(child) {
            sdf_entities.push(child);
        }
        if let Ok(grandchildren) = children_query.get(child) {
            queue.extend(grandchildren.iter());
        }
    }

    // Attack bar structure: [0]=outer(black), [1]=inner(white).
    // On flash: swap their colors.
    let colors = if inverted {
        [Vec3::ONE, Vec3::ZERO]
    } else {
        [Vec3::ZERO, Vec3::ONE]
    };

    for (i, entity) in sdf_entities.iter().enumerate() {
        let Some(color) = colors.get(i) else {
            break;
        };
        let Ok(mat_handle) = sdf_shapes.get(*entity) else {
            continue;
        };
        let Some(material) = sdf_materials.get_mut(&mat_handle.0) else {
            continue;
        };
        let alpha = material.uniform_data.color.w;
        material.uniform_data.color = color.extend(alpha);
    }
}
