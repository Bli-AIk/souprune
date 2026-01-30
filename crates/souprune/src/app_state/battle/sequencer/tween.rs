//! # sequencer/tween.rs
//!
//! ## Module Overview
//!
//! TweenViewElement systems and utilities for the battle sequencer.
//!
//! 战斗序列管理器的 TweenViewElement 系统和工具函数。

use super::super::chapter_schema::{Chapter, EasingFunction, TweenTarget, Val};
use super::context::*;
use crate::core::view::components::UIBox;
use bevy::prelude::*;

// Re-export easing function from dedicated module
pub use super::easing::apply_easing;

/// Component to track an active tween animation.
///
/// 跟踪活动补间动画的组件。
#[derive(Component)]
pub struct ActiveTween {
    /// Target entity being animated.
    pub target_entity: Entity,
    /// Start values for interpolation.
    pub start_value: TweenValue,
    /// End values for interpolation.
    pub end_value: TweenValue,
    /// Animation timer.
    pub timer: Timer,
    /// Easing function.
    pub easing: EasingFunction,
    /// Whether to wait for completion.
    pub wait_for_completion: bool,
}

/// Interpolatable value types for tween animations.
///
/// 用于补间动画的可插值值类型。
#[derive(Debug, Clone)]
pub enum TweenValue {
    /// Box size (width, height).
    BoxSize(f32, f32),
    /// Position (x, y, z).
    Position(Vec3),
    /// Scale (x, y, z).
    Scale(Vec3),
    /// Color (r, g, b, a).
    Color(Vec4),
    /// Rotation (radians).
    Rotation(f32),
    /// Alpha only.
    Alpha(f32),
}

impl TweenValue {
    /// Interpolate between two values.
    pub fn lerp(&self, other: &TweenValue, t: f32) -> TweenValue {
        match (self, other) {
            (TweenValue::BoxSize(w1, h1), TweenValue::BoxSize(w2, h2)) => {
                TweenValue::BoxSize(w1 + (w2 - w1) * t, h1 + (h2 - h1) * t)
            }
            (TweenValue::Position(v1), TweenValue::Position(v2)) => {
                TweenValue::Position(v1.lerp(*v2, t))
            }
            (TweenValue::Scale(v1), TweenValue::Scale(v2)) => TweenValue::Scale(v1.lerp(*v2, t)),
            (TweenValue::Color(v1), TweenValue::Color(v2)) => TweenValue::Color(v1.lerp(*v2, t)),
            (TweenValue::Rotation(r1), TweenValue::Rotation(r2)) => {
                TweenValue::Rotation(r1 + (r2 - r1) * t)
            }
            (TweenValue::Alpha(a1), TweenValue::Alpha(a2)) => TweenValue::Alpha(a1 + (a2 - a1) * t),
            _ => self.clone(), // Mismatched types, return start value
        }
    }
}

/// Helper to resolve a Val<f32> to an f32 value using the unified expression system.
///
/// 使用统一的表达式系统解析 Val<f32> 为 f32 值。
fn resolve_tween_val_f32(
    val: &Val<f32>,
    current: f32,
    player_data: &crate::core::data::PlayerData,
    time: Option<f64>,
) -> f32 {
    crate::core::view::ron_view::parsing::resolve_val_f32(val, Some(current), player_data, time)
}

/// System to process TweenViewElement chapters.
///
/// 处理 TweenViewElement 章节的系统。
#[allow(clippy::type_complexity)]
pub fn process_tween_view_element_system(
    mut commands: Commands,
    active_chapters: Query<
        (Entity, &ActiveChapter),
        (
            Without<ActiveTween>,
            Without<WaitTimer>,
            Without<ChapterFinished>,
        ),
    >,
    view_elements: Query<(Entity, &crate::core::view::components::ViewElement)>,
    transforms: Query<&Transform>,
    sprites: Query<&Sprite>,
    ui_boxes: Query<&UIBox>,
    player_data: Res<crate::core::data::PlayerData>,
    time: Res<Time>,
) {
    // Get current elapsed time for expression evaluation
    let current_time = time.elapsed_secs_f64();

    for (chapter_entity, active_chapter) in active_chapters.iter() {
        if let Chapter::TweenViewElement {
            selector,
            target,
            duration,
            easing,
            wait_for_completion,
        } = &active_chapter.chapter
        {
            info!(
                "[TweenViewElement] Processing: selector={:?}, target={:?}, duration={}s",
                selector, target, duration
            );

            // Resolve the selector to get target entity
            let target_entity = match selector {
                super::super::chapter_schema::ElementSelector::FullName(full_name) => {
                    crate::core::view::find_element_by_full_name(&view_elements, full_name)
                }
                super::super::chapter_schema::ElementSelector::LocalName(local_name) => {
                    view_elements
                        .iter()
                        .find(|(_, elem)| elem.local_name == *local_name)
                        .map(|(entity, _)| entity)
                }
                super::super::chapter_schema::ElementSelector::Tag(tag) => {
                    crate::core::view::find_elements_by_tag(&view_elements, tag)
                        .into_iter()
                        .next()
                }
            };

            let Some(target_entity) = target_entity else {
                warn!(
                    "[TweenViewElement] Target element not found: {:?}",
                    selector
                );
                commands.entity(chapter_entity).insert(ChapterFinished);
                continue;
            };

            // Get start and end values based on target type
            let result = resolve_tween_values(
                target,
                target_entity,
                &transforms,
                &sprites,
                &ui_boxes,
                &player_data,
                current_time,
            );

            let Some((start_value, end_value)) = result else {
                warn!("[TweenViewElement] Failed to resolve tween values");
                commands.entity(chapter_entity).insert(ChapterFinished);
                continue;
            };

            info!(
                "[TweenViewElement] Starting tween: {:?} -> {:?}",
                start_value, end_value
            );

            // Create the tween component
            if *wait_for_completion {
                commands.entity(chapter_entity).insert(ActiveTween {
                    target_entity,
                    start_value,
                    end_value,
                    timer: Timer::from_seconds(*duration, TimerMode::Once),
                    easing: *easing,
                    wait_for_completion: *wait_for_completion,
                });
            } else {
                // Spawn a detached tween entity that will clean itself up when done
                commands.spawn(ActiveTween {
                    target_entity,
                    start_value,
                    end_value,
                    timer: Timer::from_seconds(*duration, TimerMode::Once),
                    easing: *easing,
                    wait_for_completion: false,
                });
                // Mark chapter as finished immediately
                commands.entity(chapter_entity).insert(ChapterFinished);
            }
        }
    }
}

/// Resolve tween start and end values based on target type.
///
/// 根据目标类型解析补间动画的起始和结束值。
#[allow(clippy::type_complexity)]
fn resolve_tween_values(
    target: &TweenTarget,
    target_entity: Entity,
    transforms: &Query<&Transform>,
    sprites: &Query<&Sprite>,
    ui_boxes: &Query<&UIBox>,
    player_data: &crate::core::data::PlayerData,
    current_time: f64,
) -> Option<(TweenValue, TweenValue)> {
    match target {
        TweenTarget::BoxSize { from, to } => {
            let ui_box = ui_boxes.get(target_entity).ok()?;
            let current_w = ui_box.width();
            let current_h = ui_box.height();

            let start = if let Some(from) = from {
                TweenValue::BoxSize(
                    resolve_tween_val_f32(&from.0, current_w, player_data, Some(current_time)),
                    resolve_tween_val_f32(&from.1, current_h, player_data, Some(current_time)),
                )
            } else {
                TweenValue::BoxSize(current_w, current_h)
            };

            let end = TweenValue::BoxSize(
                resolve_tween_val_f32(&to.0, current_w, player_data, Some(current_time)),
                resolve_tween_val_f32(&to.1, current_h, player_data, Some(current_time)),
            );
            Some((start, end))
        }
        TweenTarget::Position { from, to } => {
            let transform = transforms.get(target_entity).ok()?;
            let current = transform.translation;

            let start = if let Some(from) = from {
                TweenValue::Position(Vec3::new(
                    resolve_tween_val_f32(&from.0, current.x, player_data, Some(current_time)),
                    resolve_tween_val_f32(&from.1, current.y, player_data, Some(current_time)),
                    resolve_tween_val_f32(&from.2, current.z, player_data, Some(current_time)),
                ))
            } else {
                TweenValue::Position(current)
            };

            let end = TweenValue::Position(Vec3::new(
                resolve_tween_val_f32(&to.0, current.x, player_data, Some(current_time)),
                resolve_tween_val_f32(&to.1, current.y, player_data, Some(current_time)),
                resolve_tween_val_f32(&to.2, current.z, player_data, Some(current_time)),
            ));
            Some((start, end))
        }
        TweenTarget::Scale { from, to } => {
            let transform = transforms.get(target_entity).ok()?;
            let current = transform.scale;

            let start = if let Some(from) = from {
                TweenValue::Scale(Vec3::new(
                    resolve_tween_val_f32(&from.0, current.x, player_data, Some(current_time)),
                    resolve_tween_val_f32(&from.1, current.y, player_data, Some(current_time)),
                    resolve_tween_val_f32(&from.2, current.z, player_data, Some(current_time)),
                ))
            } else {
                TweenValue::Scale(current)
            };

            let end = TweenValue::Scale(Vec3::new(
                resolve_tween_val_f32(&to.0, current.x, player_data, Some(current_time)),
                resolve_tween_val_f32(&to.1, current.y, player_data, Some(current_time)),
                resolve_tween_val_f32(&to.2, current.z, player_data, Some(current_time)),
            ));
            Some((start, end))
        }
        TweenTarget::Color { from, to } => {
            let sprite = sprites.get(target_entity).ok()?;
            let c = sprite.color.to_linear();
            let current = Vec4::new(c.red, c.green, c.blue, c.alpha);

            let start = if let Some(from) = from {
                TweenValue::Color(Vec4::new(
                    resolve_tween_val_f32(&from.0, current.x, player_data, Some(current_time)),
                    resolve_tween_val_f32(&from.1, current.y, player_data, Some(current_time)),
                    resolve_tween_val_f32(&from.2, current.z, player_data, Some(current_time)),
                    resolve_tween_val_f32(&from.3, current.w, player_data, Some(current_time)),
                ))
            } else {
                TweenValue::Color(current)
            };

            let end = TweenValue::Color(Vec4::new(
                resolve_tween_val_f32(&to.0, current.x, player_data, Some(current_time)),
                resolve_tween_val_f32(&to.1, current.y, player_data, Some(current_time)),
                resolve_tween_val_f32(&to.2, current.z, player_data, Some(current_time)),
                resolve_tween_val_f32(&to.3, current.w, player_data, Some(current_time)),
            ));
            Some((start, end))
        }
        TweenTarget::Rotation { from, to } => {
            let transform = transforms.get(target_entity).ok()?;
            let (_, _, z) = transform.rotation.to_euler(EulerRot::XYZ);

            let start = if let Some(from) = from {
                TweenValue::Rotation(resolve_tween_val_f32(
                    from,
                    z,
                    player_data,
                    Some(current_time),
                ))
            } else {
                TweenValue::Rotation(z)
            };

            let end = TweenValue::Rotation(resolve_tween_val_f32(
                to,
                z,
                player_data,
                Some(current_time),
            ));
            Some((start, end))
        }
        TweenTarget::Alpha { from, to } => {
            let sprite = sprites.get(target_entity).ok()?;
            let current_alpha = sprite.color.alpha();

            let start = if let Some(from) = from {
                TweenValue::Alpha(resolve_tween_val_f32(
                    from,
                    current_alpha,
                    player_data,
                    Some(current_time),
                ))
            } else {
                TweenValue::Alpha(current_alpha)
            };

            let end = TweenValue::Alpha(resolve_tween_val_f32(
                to,
                current_alpha,
                player_data,
                Some(current_time),
            ));
            Some((start, end))
        }
    }
}

/// System to update active tween animations.
///
/// 更新活动补间动画的系统。
#[allow(clippy::type_complexity)]
pub fn process_tween_wait_chapter_system(
    mut commands: Commands,
    time: Res<Time>,
    mut tween_query: Query<(Entity, &mut ActiveTween), Without<ChapterFinished>>,
    mut transforms: Query<&mut Transform>,
    mut sprites: Query<&mut Sprite>,
    mut ui_boxes: Query<&mut UIBox>,
) {
    for (entity, mut tween) in tween_query.iter_mut() {
        tween.timer.tick(time.delta());

        let linear_t = tween.timer.elapsed_secs() / tween.timer.duration().as_secs_f32();
        let eased_t = apply_easing(linear_t.min(1.0), tween.easing);

        // Interpolate and apply the value
        let current = tween.start_value.lerp(&tween.end_value, eased_t);

        match current {
            TweenValue::BoxSize(width, height) => {
                if let Ok(mut ui_box) = ui_boxes.get_mut(tween.target_entity) {
                    ui_box.width = width;
                    ui_box.height = height;
                }
            }
            TweenValue::Position(pos) => {
                if let Ok(mut transform) = transforms.get_mut(tween.target_entity) {
                    transform.translation = pos;
                }
            }
            TweenValue::Scale(scale) => {
                if let Ok(mut transform) = transforms.get_mut(tween.target_entity) {
                    transform.scale = scale;
                }
            }
            TweenValue::Color(color) => {
                if let Ok(mut sprite) = sprites.get_mut(tween.target_entity) {
                    sprite.color = Color::linear_rgba(color.x, color.y, color.z, color.w);
                }
            }
            TweenValue::Rotation(angle) => {
                if let Ok(mut transform) = transforms.get_mut(tween.target_entity) {
                    transform.rotation = Quat::from_rotation_z(angle);
                }
            }
            TweenValue::Alpha(alpha) => {
                if let Ok(mut sprite) = sprites.get_mut(tween.target_entity) {
                    sprite.color = sprite.color.with_alpha(alpha);
                }
            }
        }

        if tween.timer.is_finished() {
            info!("[TweenViewElement] Tween completed");
            if tween.wait_for_completion {
                commands.entity(entity).insert(ChapterFinished);
            } else {
                // This is a detached tween entity, despawn it
                commands.entity(entity).despawn();
            }
        }
    }
}
