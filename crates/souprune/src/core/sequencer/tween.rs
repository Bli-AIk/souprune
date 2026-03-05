//! # sequencer/tween.rs
//!
//! ## Module Overview
//!
//! TweenViewElement systems and utilities for the battle sequencer using bevy_tween.
//!
//! 战斗序列管理器的 TweenViewElement 系统，使用 bevy_tween。

use super::chapter_schema::{Chapter, TweenTarget, Val};
use super::context::*;
use crate::core::view::components::ViewBox;
use bevy::prelude::*;
use bevy_tween::interpolate::Interpolator;
use bevy_tween::prelude::*;
use std::time::Duration;

// ============================================================================
// Custom Interpolators for ViewBox
// ViewBox 的自定义插值器
// ============================================================================

/// Interpolator for ViewBox size.
///
/// ViewBox 尺寸的插值器。
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

/// Interpolator for Sprite alpha only.
///
/// 仅精灵透明度的插值器。
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

// ============================================================================
// Tween Tracking Components
// 补间跟踪组件
// ============================================================================

/// Marker component to track that a tween animation is in progress for this chapter.
///
/// 标记组件，用于跟踪此章节的补间动画正在进行中。
#[derive(Component)]
pub struct TweenInProgress {
    /// Whether to wait for completion before marking chapter as finished.
    pub wait_for_completion: bool,
    /// The entity running the TimeRunner for this tween.
    pub animator_entity: Entity,
}

// ============================================================================
// Systems
// 系统
// ============================================================================

/// Helper to resolve a Val<f32> to an f32 value using the unified expression system.
///
/// 使用统一的表达式系统解析 Val<f32> 为 f32 值。
fn resolve_tween_val_f32(
    val: &Val<f32>,
    current: f32,
    player_data: &crate::core::view::ron_view::parsing::PlayerDataView<'_>,
    time: Option<f64>,
) -> f32 {
    crate::core::view::ron_view::parsing::resolve_val_f32(val, Some(current), player_data, time)
}

/// System to process TweenViewElement chapters and spawn bevy_tween animations.
///
/// 处理 TweenViewElement 章节并生成 bevy_tween 动画的系统。
#[expect(clippy::type_complexity)] // reason: Bevy query type complexity
pub fn process_tween_view_element_system(
    mut commands: Commands,
    active_chapters: Query<
        (Entity, &ActiveChapter),
        (
            Without<TweenInProgress>,
            Without<WaitTimer>,
            Without<ChapterFinished>,
        ),
    >,
    view_elements: Query<(Entity, &crate::core::view::components::ViewElement)>,
    transforms: Query<&Transform>,
    sprites: Query<&Sprite>,
    ui_boxes: Query<&ViewBox>,
    layered_db: Res<bevy_fact_rule_event::LayeredFactDatabase>,
    time: Res<Time>,
) {
    use crate::core::view::ron_view::parsing::PlayerDataView;
    use bevy_tween::combinator::tween;
    use bevy_tween::interpolate::{angle_z, scale, sprite_color, translation};

    let player_data = PlayerDataView::new(&layered_db);
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
                super::chapter_schema::ElementSelector::FullName(full_name) => {
                    crate::core::view::find_element_by_full_name(&view_elements, full_name)
                }
                super::chapter_schema::ElementSelector::LocalName(local_name) => view_elements
                    .iter()
                    .find(|(_, elem)| elem.local_name == *local_name)
                    .map(|(entity, _)| entity),
                super::chapter_schema::ElementSelector::Tag(tag) => {
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

            let duration = Duration::from_secs_f32(*duration);
            let ease_kind = *easing;
            let target_component = target_entity.into_target();

            // Create tween based on target type
            match target {
                TweenTarget::Position { from, to } => {
                    let Ok(transform) = transforms.get(target_entity) else {
                        warn!("[TweenViewElement] Target has no Transform component");
                        commands.entity(chapter_entity).insert(ChapterFinished);
                        continue;
                    };
                    let current = transform.translation;

                    let start = if let Some(from) = from {
                        Vec3::new(
                            resolve_tween_val_f32(
                                &from.0,
                                current.x,
                                &player_data,
                                Some(current_time),
                            ),
                            resolve_tween_val_f32(
                                &from.1,
                                current.y,
                                &player_data,
                                Some(current_time),
                            ),
                            resolve_tween_val_f32(
                                &from.2,
                                current.z,
                                &player_data,
                                Some(current_time),
                            ),
                        )
                    } else {
                        current
                    };

                    let end = Vec3::new(
                        resolve_tween_val_f32(&to.0, current.x, &player_data, Some(current_time)),
                        resolve_tween_val_f32(&to.1, current.y, &player_data, Some(current_time)),
                        resolve_tween_val_f32(&to.2, current.z, &player_data, Some(current_time)),
                    );

                    let animator_entity = commands
                        .spawn_empty()
                        .animation()
                        .insert(tween(
                            duration,
                            ease_kind,
                            target_component.with(translation(start, end)),
                        ))
                        .id();

                    handle_wait_for_completion(
                        &mut commands,
                        chapter_entity,
                        animator_entity,
                        *wait_for_completion,
                    );
                }
                TweenTarget::Scale { from, to } => {
                    let Ok(transform) = transforms.get(target_entity) else {
                        warn!("[TweenViewElement] Target has no Transform component");
                        commands.entity(chapter_entity).insert(ChapterFinished);
                        continue;
                    };
                    let current = transform.scale;

                    let start = if let Some(from) = from {
                        Vec3::new(
                            resolve_tween_val_f32(
                                &from.0,
                                current.x,
                                &player_data,
                                Some(current_time),
                            ),
                            resolve_tween_val_f32(
                                &from.1,
                                current.y,
                                &player_data,
                                Some(current_time),
                            ),
                            resolve_tween_val_f32(
                                &from.2,
                                current.z,
                                &player_data,
                                Some(current_time),
                            ),
                        )
                    } else {
                        current
                    };

                    let end = Vec3::new(
                        resolve_tween_val_f32(&to.0, current.x, &player_data, Some(current_time)),
                        resolve_tween_val_f32(&to.1, current.y, &player_data, Some(current_time)),
                        resolve_tween_val_f32(&to.2, current.z, &player_data, Some(current_time)),
                    );

                    let animator_entity = commands
                        .spawn_empty()
                        .animation()
                        .insert(tween(
                            duration,
                            ease_kind,
                            target_component.with(scale(start, end)),
                        ))
                        .id();

                    handle_wait_for_completion(
                        &mut commands,
                        chapter_entity,
                        animator_entity,
                        *wait_for_completion,
                    );
                }
                TweenTarget::Rotation { from, to } => {
                    let Ok(transform) = transforms.get(target_entity) else {
                        warn!("[TweenViewElement] Target has no Transform component");
                        commands.entity(chapter_entity).insert(ChapterFinished);
                        continue;
                    };
                    let (_, _, z) = transform.rotation.to_euler(EulerRot::XYZ);

                    let start = if let Some(from) = from {
                        resolve_tween_val_f32(from, z, &player_data, Some(current_time))
                    } else {
                        z
                    };

                    let end = resolve_tween_val_f32(to, z, &player_data, Some(current_time));

                    let animator_entity = commands
                        .spawn_empty()
                        .animation()
                        .insert(tween(
                            duration,
                            ease_kind,
                            target_component.with(angle_z(start, end)),
                        ))
                        .id();

                    handle_wait_for_completion(
                        &mut commands,
                        chapter_entity,
                        animator_entity,
                        *wait_for_completion,
                    );
                }
                TweenTarget::Color { from, to } => {
                    let Ok(sprite) = sprites.get(target_entity) else {
                        warn!("[TweenViewElement] Target has no Sprite component");
                        commands.entity(chapter_entity).insert(ChapterFinished);
                        continue;
                    };
                    let c = sprite.color.to_srgba();
                    let current = Vec4::new(c.red, c.green, c.blue, c.alpha);

                    let start = if let Some(from) = from {
                        Color::srgba(
                            resolve_tween_val_f32(
                                &from.0,
                                current.x,
                                &player_data,
                                Some(current_time),
                            ),
                            resolve_tween_val_f32(
                                &from.1,
                                current.y,
                                &player_data,
                                Some(current_time),
                            ),
                            resolve_tween_val_f32(
                                &from.2,
                                current.z,
                                &player_data,
                                Some(current_time),
                            ),
                            resolve_tween_val_f32(
                                &from.3,
                                current.w,
                                &player_data,
                                Some(current_time),
                            ),
                        )
                    } else {
                        sprite.color
                    };

                    let end = Color::srgba(
                        resolve_tween_val_f32(&to.0, current.x, &player_data, Some(current_time)),
                        resolve_tween_val_f32(&to.1, current.y, &player_data, Some(current_time)),
                        resolve_tween_val_f32(&to.2, current.z, &player_data, Some(current_time)),
                        resolve_tween_val_f32(&to.3, current.w, &player_data, Some(current_time)),
                    );

                    let animator_entity = commands
                        .spawn_empty()
                        .animation()
                        .insert(tween(
                            duration,
                            ease_kind,
                            target_component.with(sprite_color(start, end)),
                        ))
                        .id();

                    handle_wait_for_completion(
                        &mut commands,
                        chapter_entity,
                        animator_entity,
                        *wait_for_completion,
                    );
                }
                TweenTarget::Alpha { from, to } => {
                    let Ok(sprite) = sprites.get(target_entity) else {
                        warn!("[TweenViewElement] Target has no Sprite component");
                        commands.entity(chapter_entity).insert(ChapterFinished);
                        continue;
                    };
                    let current_alpha = sprite.color.alpha();

                    let start = if let Some(from) = from {
                        resolve_tween_val_f32(from, current_alpha, &player_data, Some(current_time))
                    } else {
                        current_alpha
                    };

                    let end =
                        resolve_tween_val_f32(to, current_alpha, &player_data, Some(current_time));

                    let animator_entity = commands
                        .spawn_empty()
                        .animation()
                        .insert(tween(
                            duration,
                            ease_kind,
                            target_component.with(SpriteAlphaInterpolator { start, end }),
                        ))
                        .id();

                    handle_wait_for_completion(
                        &mut commands,
                        chapter_entity,
                        animator_entity,
                        *wait_for_completion,
                    );
                }
                TweenTarget::BoxSize { from, to } => {
                    let Ok(ui_box) = ui_boxes.get(target_entity) else {
                        warn!("[TweenViewElement] Target has no ViewBox component");
                        commands.entity(chapter_entity).insert(ChapterFinished);
                        continue;
                    };
                    let current_w = ui_box.width();
                    let current_h = ui_box.height();

                    let (start_w, start_h) = if let Some(from) = from {
                        (
                            resolve_tween_val_f32(
                                &from.0,
                                current_w,
                                &player_data,
                                Some(current_time),
                            ),
                            resolve_tween_val_f32(
                                &from.1,
                                current_h,
                                &player_data,
                                Some(current_time),
                            ),
                        )
                    } else {
                        (current_w, current_h)
                    };

                    let end_w =
                        resolve_tween_val_f32(&to.0, current_w, &player_data, Some(current_time));
                    let end_h =
                        resolve_tween_val_f32(&to.1, current_h, &player_data, Some(current_time));

                    let animator_entity = commands
                        .spawn_empty()
                        .animation()
                        .insert(tween(
                            duration,
                            ease_kind,
                            target_component.with(ViewBoxSizeInterpolator {
                                start_width: start_w,
                                start_height: start_h,
                                end_width: end_w,
                                end_height: end_h,
                            }),
                        ))
                        .id();

                    handle_wait_for_completion(
                        &mut commands,
                        chapter_entity,
                        animator_entity,
                        *wait_for_completion,
                    );
                }
            }

            info!(
                "[TweenViewElement] Started bevy_tween animation for entity {:?}",
                target_entity
            );
        }
    }
}

/// Helper to handle wait_for_completion logic.
///
/// 处理 wait_for_completion 逻辑的辅助函数。
fn handle_wait_for_completion(
    commands: &mut Commands,
    chapter_entity: Entity,
    animator_entity: Entity,
    wait_for_completion: bool,
) {
    if wait_for_completion {
        commands.entity(chapter_entity).insert(TweenInProgress {
            wait_for_completion: true,
            animator_entity,
        });
    } else {
        // Mark chapter as finished immediately
        commands.entity(chapter_entity).insert(ChapterFinished);
    }
}

/// System to check if tween animations have completed.
///
/// 检查补间动画是否完成的系统。
pub fn process_tween_wait_chapter_system(
    mut commands: Commands,
    tween_chapters: Query<(Entity, &TweenInProgress), Without<ChapterFinished>>,
    time_runners: Query<&bevy_tween::bevy_time_runner::TimeRunner>,
) {
    for (chapter_entity, tween_progress) in tween_chapters.iter() {
        // Check if the animator's TimeRunner has finished
        if let Ok(runner) = time_runners.get(tween_progress.animator_entity) {
            if runner.is_completed() {
                info!("[TweenViewElement] Tween completed");
                commands.entity(chapter_entity).insert(ChapterFinished);
                commands.entity(chapter_entity).remove::<TweenInProgress>();
                // Clean up the animator entity
                commands.entity(tween_progress.animator_entity).despawn();
            }
        } else {
            // TimeRunner doesn't exist anymore, animation must have finished
            info!("[TweenViewElement] Tween animator no longer exists, marking as completed");
            commands.entity(chapter_entity).insert(ChapterFinished);
            commands.entity(chapter_entity).remove::<TweenInProgress>();
        }
    }
}
