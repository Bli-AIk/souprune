//! # motion.rs
//!
//! # motion.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Updates live bullets by calling into the loaded WASM danmaku behaviors. It builds
//! the per-frame bullet context, accumulates position and rotation deltas from every active
//! behavior, and writes the resulting transform, scale, and opacity back to the ECS entities.
//!
//! 负责通过已加载的 WASM 弹幕行为更新存活中的子弹。它会为每帧构建子弹上下文，
//! 汇总所有活跃行为给出的位移和旋转增量，再把结果写回实体的变换、缩放和透明度。

use super::*;

fn build_bullet_ctx(
    state: &BulletMotionState,
    dt: f32,
    player_pos: Vec2,
    props: &HashMap<String, f32>,
) -> souprune_api::BulletContext {
    souprune_api::BulletContext {
        elapsed: state.elapsed,
        delta_time: dt,
        spawn_pos: souprune_api::Vec2::new(state.spawn_center.x, state.spawn_center.y),
        offset: souprune_api::Vec2::new(state.initial_offset.x, state.initial_offset.y),
        initial_angle: state.initial_angle,
        initial_radius: state.initial_radius,
        player_pos: souprune_api::Vec2::new(player_pos.x, player_pos.y),
        props: props
            .iter()
            .map(|(name, value)| souprune_api::Prop {
                name: name.clone(),
                value: *value,
            })
            .collect(),
    }
}

fn apply_output_extras(
    output: &souprune_api::BulletOutput,
    opacity: &mut Option<f32>,
    scale_delta: &mut Vec2,
) {
    if output.opacity >= 0.0 {
        *opacity = Some(output.opacity);
    }
    if output.scale_x != 0.0 {
        scale_delta.x += output.scale_x;
    }
    if output.scale_y != 0.0 {
        scale_delta.y += output.scale_y;
    }
}

/// System to update bullet motion.
///
/// Builtin behaviors (Linear, Orbital, Sine, Tween, Stationary, Aimed)
/// are computed directly in Rust. Only `Custom` behaviors go through WASM.
///
/// 更新弹幕运动的系统。内置行为直接在 Rust 计算，仅 Custom 经 WASM 调度。
pub fn update_bullet_motion(
    time: Res<Time>,
    mut loaded_mods: NonSendMut<LoadedMods>,
    container_query: Query<&Transform, (With<BulletContainer>, Without<Bullet>)>,
    mut query: Query<
        (
            &mut Transform,
            &ChildOf,
            &mut BulletMotionState,
            &BulletBaseScale,
            Option<&mut Sprite>,
            Option<&mut super::super::builtin_motion::BuiltinMotionStack>,
            Option<&mut ActiveDanmakuStack>,
        ),
        With<Bullet>,
    >,
    player_query: Query<&Transform, (With<BulletTarget>, Without<Bullet>)>,
    mut wasm_tracer: ResMut<crate::core::trace::WasmCallTracer>,
) {
    use super::super::builtin_motion::compute_builtin_output;

    let dt = time.delta_secs();
    let player_pos = player_query
        .iter()
        .next()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    for (mut transform, parent, mut state, base_scale, sprite, builtin_stack, danmaku_stack) in
        query.iter_mut()
    {
        state.elapsed += dt;

        let mut position = state.spawn_center + state.initial_offset;
        let mut rotation_delta = 0.0;
        let mut scale_delta = Vec2::ZERO;
        let mut opacity: Option<f32> = None;

        // --- Builtin behaviors: pure Rust ---
        if let Some(mut builtin) = builtin_stack {
            for bs in builtin.states.iter_mut() {
                let out = compute_builtin_output(
                    bs,
                    state.elapsed,
                    dt,
                    state.initial_offset,
                    state.initial_angle,
                    state.initial_radius,
                );
                position += out.offset;
                rotation_delta += out.rotation;
                if out.opacity >= 0.0 {
                    opacity = Some(out.opacity);
                }
                if out.scale_x != 0.0 {
                    scale_delta.x += out.scale_x;
                }
                if out.scale_y != 0.0 {
                    scale_delta.y += out.scale_y;
                }
            }
        }

        // --- Custom behaviors: WASM ---
        if let Some(mut stack) = danmaku_stack {
            for instance in stack.instances.iter_mut() {
                let ctx = build_bullet_ctx(&state, dt, player_pos, &instance.props);
                let start = std::time::Instant::now();
                let output = instance.call_on_update(&ctx, &mut loaded_mods);
                let elapsed_dur = start.elapsed();

                let mod_name = loaded_mods
                    .mods
                    .get(instance.mod_index())
                    .map(|m| m.name.as_str())
                    .unwrap_or("unknown");
                wasm_tracer.record(mod_name, "danmaku", "on_update", elapsed_dur);

                position += Vec2::new(output.offset.x, output.offset.y);
                rotation_delta += output.rotation;
                apply_output_extras(&output, &mut opacity, &mut scale_delta);
            }
        }

        if let Ok(parent_transform) = container_query.get(parent.0) {
            let parent_pos = parent_transform.translation.truncate();
            let local_pos = position - parent_pos;
            transform.translation.x = local_pos.x;
            transform.translation.y = local_pos.y;
        } else {
            transform.translation.x = position.x;
            transform.translation.y = position.y;
        }

        if rotation_delta != 0.0 {
            transform.rotate_z(rotation_delta);
        }

        if scale_delta != Vec2::ZERO {
            transform.scale.x = base_scale.0 * (1.0 + scale_delta.x);
            transform.scale.y = base_scale.0 * (1.0 + scale_delta.y);
        }

        if let (Some(opacity_val), Some(mut sprite)) = (opacity, sprite) {
            sprite.color.set_alpha(opacity_val);
        }
    }
}
