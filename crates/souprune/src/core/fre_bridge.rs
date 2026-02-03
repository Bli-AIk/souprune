//! FRE (Fact-Rule-Event) Bridge Module
//!
//! This module provides the bridge between game systems and the FRE rule engine.
//! It handles:
//! - Converting input actions to FRE events (ActionEvent)
//! - Processing FRE actions (PlaySound, SetLocalFact, CloseView, etc.)
//! - Managing ActiveView markers
//!
//! FRE（Fact-Rule-Event）桥接模块
//!
//! 此模块提供游戏系统和 FRE 规则引擎之间的桥接。
//! 它处理：
//! - 将输入动作转换为 FRE 事件（ActionEvent）
//! - 处理 FRE 动作（PlaySound、SetLocalFact、CloseView 等）
//! - 管理 ActiveView 标记

use bevy::prelude::*;
use bevy_fact_rule_event::{
    ActionHandlerRegistry, FactEvent, FactValue, LocalFactValue, RuleActionDef,
};
use leafwing_input_manager::action_state::ActionState;

use crate::core::audio;
use crate::core::input::{Action, ActionRegistry, ActionStateExt};
use crate::core::view::components::{ActiveView, ViewRoot};

/// System that converts input actions to FRE events.
///
/// This system checks all registered actions and emits FRE events
/// in the format `action:{action_name}:{kind}` where kind is one of:
/// - `just_pressed`
/// - `pressed`
/// - `just_released`
///
/// 将输入动作转换为 FRE 事件的系统。
///
/// 此系统检查所有已注册的动作，并以 `action:{action_name}:{kind}` 格式
/// 发出 FRE 事件，其中 kind 是以下之一：
/// - `just_pressed`
/// - `pressed`
/// - `just_released`
pub fn action_to_fre_event_system(
    registry: Res<ActionRegistry>,
    query: Query<&ActionState<Action>>,
    mut event_writer: MessageWriter<FactEvent>,
) {
    let Ok(action_state) = query.single() else {
        return;
    };

    // Emit events for each action state change
    for action_name in registry.all_actions() {
        // JustPressed events
        if action_state.action_just_pressed(&registry, action_name) {
            let event_id = format!("action:{}:just_pressed", action_name);
            debug!(
                "FRE Bridge: {} just_pressed, emitting {}",
                action_name, event_id
            );
            event_writer.write(FactEvent::new(event_id));
        }

        // Pressed (held) events
        if action_state.action_pressed(&registry, action_name) {
            let event_id = format!("action:{}:pressed", action_name);
            // Note: This fires every frame while held, may need throttling
            event_writer.write(FactEvent::new(event_id));
        }

        // JustReleased events
        if action_state.action_just_released(&registry, action_name) {
            let event_id = format!("action:{}:just_released", action_name);
            debug!(
                "FRE Bridge: {} just_released, emitting {}",
                action_name, event_id
            );
            event_writer.write(FactEvent::new(event_id));
        }
    }
}

/// Register FRE action handlers for View-related actions.
///
/// This function registers handlers for:
/// - PlaySound: Play audio using fuzzy search
/// - PlaySoundFullPath: Play audio using full path
/// - SetLocalFact: Set a fact on the active ViewRoot
/// - CloseView: Close the active View
/// - SwitchState: Switch game state
///
/// 为 View 相关动作注册 FRE 动作处理器。
///
/// 此函数注册以下处理器：
/// - PlaySound：使用模糊搜索播放音频
/// - PlaySoundFullPath：使用完整路径播放音频
/// - SetLocalFact：在活跃的 ViewRoot 上设置 fact
/// - CloseView：关闭活跃的 View
/// - SwitchState：切换游戏状态
pub fn register_view_action_handlers(registry: &mut ActionHandlerRegistry) {
    // Note: These handlers are minimal placeholders.
    // Actual implementation requires access to more resources
    // which need to be handled via Commands or events.

    registry.register("PlaySound", |action, _db, _commands| {
        if let RuleActionDef::PlaySound(sound_name) = action {
            info!("FRE Action: PlaySound({})", sound_name);
            // Actual audio playback is handled by a dedicated system
            // that processes PlaySoundRequest events
        }
    });

    registry.register("PlaySoundFullPath", |action, _db, _commands| {
        if let RuleActionDef::PlaySoundFullPath(path) = action {
            info!("FRE Action: PlaySoundFullPath({})", path);
        }
    });

    registry.register("SetLocalFact", |action, _db, _commands| {
        if let RuleActionDef::SetLocalFact(key, value) = action {
            info!("FRE Action: SetLocalFact({}, {:?})", key, value);
            // Actual fact setting is handled by process_fre_actions_system
        }
    });

    registry.register("CloseView", |_action, _db, _commands| {
        info!("FRE Action: CloseView");
    });

    registry.register("SwitchState", |action, _db, _commands| {
        if let RuleActionDef::SwitchState(state_name) = action {
            info!("FRE Action: SwitchState({})", state_name);
        }
    });
}

/// Message for requesting sound playback from FRE actions.
///
/// 从 FRE 动作请求音效播放的消息。
#[derive(Message, Debug, Clone)]
pub struct PlaySoundRequest {
    /// Sound name (for fuzzy search) or full path.
    pub sound: String,
    /// Whether this is a full path or fuzzy search name.
    pub is_full_path: bool,
}

/// Message for requesting state change from FRE actions.
///
/// 从 FRE 动作请求状态变更的消息。
#[derive(Message, Debug, Clone)]
pub struct SwitchStateRequest {
    /// Target state name.
    pub state_name: String,
}

/// Message for requesting local fact update from FRE actions.
///
/// 从 FRE 动作请求更新局部 fact 的消息。
#[derive(Message, Debug, Clone)]
pub struct SetLocalFactRequest {
    /// Fact key.
    pub key: String,
    /// Fact value (or expression).
    pub value: LocalFactValue,
}

/// Message for requesting View close from FRE actions.
///
/// 从 FRE 动作请求关闭 View 的消息。
#[derive(Message, Debug, Clone)]
pub struct CloseViewRequest;

/// System that processes SetLocalFact requests on the active ViewRoot.
///
/// 处理活跃 ViewRoot 上 SetLocalFact 请求的系统。
pub fn process_set_local_fact_system(
    mut events: MessageReader<SetLocalFactRequest>,
    mut active_view_query: Query<&mut ViewRoot, With<ActiveView>>,
) {
    for request in events.read() {
        if let Ok(mut view_root) = active_view_query.single_mut() {
            let value = match &request.value {
                LocalFactValue::Int(v) => FactValue::Int(*v),
                LocalFactValue::Float(v) => FactValue::Float(*v),
                LocalFactValue::Bool(v) => FactValue::Bool(*v),
                LocalFactValue::String(v) => FactValue::String(v.clone()),
                LocalFactValue::Expr(expr) => {
                    // Expression evaluation - for now, just try to parse as int
                    // TODO: Implement proper expression evaluator using core/view/ron_view/parsing.rs
                    if let Some(result) = evaluate_simple_expression(expr, &view_root.local_facts) {
                        result
                    } else {
                        warn!("FRE: Failed to evaluate expression '{}', using 0", expr);
                        FactValue::Int(0)
                    }
                }
            };

            debug!(
                "FRE: SetLocalFact on ActiveView: {} = {:?}",
                request.key, value
            );
            view_root.local_facts.set(request.key.as_str(), value);
        } else {
            warn!("FRE: SetLocalFact called but no ActiveView found");
        }
    }
}

/// Simple expression evaluator for basic arithmetic.
///
/// Supports:
/// - `$name` - local fact reference
/// - `$name + 1`, `$name - 1` - simple increment/decrement
///
/// 简单的表达式评估器，用于基本算术。
///
/// 支持：
/// - `$name` - 局部 fact 引用
/// - `$name + 1`、`$name - 1` - 简单增减
fn evaluate_simple_expression(
    expr: &str,
    facts: &bevy_fact_rule_event::FactDatabase,
) -> Option<FactValue> {
    let expr = expr.trim();

    // Handle simple variable reference: $name
    if expr.starts_with('$') && !expr.contains(' ') {
        let var_name = &expr[1..];
        return facts.get_by_str(var_name).cloned();
    }

    // Handle simple addition: $name + N
    if let Some(idx) = expr.find(" + ") {
        let left = expr[..idx].trim();
        let right = expr[idx + 3..].trim();

        if let Some(var_name) = left.strip_prefix('$') {
            if let Some(FactValue::Int(left_val)) = facts.get_by_str(var_name)
                && let Ok(right_val) = right.parse::<i64>()
            {
                return Some(FactValue::Int(left_val + right_val));
            }
        }
    }

    // Handle simple subtraction: $name - N
    if let Some(idx) = expr.find(" - ") {
        let left = expr[..idx].trim();
        let right = expr[idx + 3..].trim();

        if let Some(var_name) = left.strip_prefix('$') {
            if let Some(FactValue::Int(left_val)) = facts.get_by_str(var_name)
                && let Ok(right_val) = right.parse::<i64>()
            {
                return Some(FactValue::Int(left_val - right_val));
            }
        }
    }

    // Try parsing as literal integer
    if let Ok(v) = expr.parse::<i64>() {
        return Some(FactValue::Int(v));
    }

    None
}

/// System that processes PlaySound requests.
///
/// 处理 PlaySound 请求的系统。
#[cfg(all(feature = "bevy_kira_audio", not(feature = "firewheel")))]
pub fn process_play_sound_system(
    mut events: MessageReader<PlaySoundRequest>,
    audio: Res<bevy_kira_audio::Audio>,
    asset_server: Res<AssetServer>,
) {
    for request in events.read() {
        if request.is_full_path {
            audio::play_sound_full_path(&audio, &asset_server, &request.sound);
        } else {
            audio::play_sound(&audio, &asset_server, &request.sound);
        }
    }
}

/// System that processes PlaySound requests (Firewheel variant).
///
/// 处理 PlaySound 请求的系统（Firewheel 变体）。
#[cfg(feature = "firewheel")]
pub fn process_play_sound_system(
    mut events: MessageReader<PlaySoundRequest>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for request in events.read() {
        if request.is_full_path {
            audio::play_sound_full_path(&mut commands, &asset_server, &request.sound);
        } else {
            audio::play_sound(&mut commands, &asset_server, &request.sound);
        }
    }
}

/// Plugin for FRE-View bridge systems.
///
/// FRE-View 桥接系统的插件。
pub struct FREBridgePlugin;

impl Plugin for FREBridgePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlaySoundRequest>()
            .add_message::<SwitchStateRequest>()
            .add_message::<SetLocalFactRequest>()
            .add_message::<CloseViewRequest>()
            .add_systems(
                Update,
                (process_set_local_fact_system, process_play_sound_system),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fact_rule_event::FactDatabase;

    #[test]
    fn test_evaluate_simple_expression_variable() {
        let mut facts = FactDatabase::new();
        facts.set("selection", FactValue::Int(3));

        let result = evaluate_simple_expression("$selection", &facts);
        assert_eq!(result, Some(FactValue::Int(3)));
    }

    #[test]
    fn test_evaluate_simple_expression_addition() {
        let mut facts = FactDatabase::new();
        facts.set("selection", FactValue::Int(3));

        let result = evaluate_simple_expression("$selection + 1", &facts);
        assert_eq!(result, Some(FactValue::Int(4)));
    }

    #[test]
    fn test_evaluate_simple_expression_subtraction() {
        let mut facts = FactDatabase::new();
        facts.set("selection", FactValue::Int(3));

        let result = evaluate_simple_expression("$selection - 1", &facts);
        assert_eq!(result, Some(FactValue::Int(2)));
    }

    #[test]
    fn test_evaluate_simple_expression_literal() {
        let facts = FactDatabase::new();

        let result = evaluate_simple_expression("42", &facts);
        assert_eq!(result, Some(FactValue::Int(42)));
    }

    #[test]
    fn test_evaluate_simple_expression_missing_variable() {
        let facts = FactDatabase::new();

        let result = evaluate_simple_expression("$missing", &facts);
        assert_eq!(result, None);
    }
}
