//! Runs `TweenViewElement` chapters by creating the appropriate runtime tweens for view entities.
//!
//! 执行 `TweenViewElement` 章节，为 View 实体创建对应的运行时 tween。
//!
//! Acts as the tween executor for sequence chapters. It resolves the target
//! element, evaluates expression-backed tween endpoints against current data, and
//! spawns animator entities that either complete immediately or block chapter
//! progression until the tween has finished.
//!
//! 序列章节的 tween 执行器。它会解析目标元素、根据当前数据求出
//! 表达式驱动的 tween 起止值，并生成动画器实体；这些 tween 要么立刻放行章节，
//! 要么在完成之前阻塞流程继续前进。

use super::interpolators::{
    SpriteAlphaInterpolator, TweenInProgress, ViewBoxAlphaInterpolator, ViewBoxSizeInterpolator,
};
use crate::core::sequencer::chapter_schema::{Chapter, TweenTarget, Value};
use crate::core::sequencer::context::{ActiveChapter, ChapterFinished, WaitTimer};
use crate::core::view::components::ViewBox;
use bevy::prelude::*;
use bevy_tween::prelude::*;
use std::time::Duration;

fn resolve_tween_val_f32(
    val: &Value<f32>,
    current: f32,
    player_data: &crate::core::view::ron_view::parsing::PlayerDataView<'_>,
    time: Option<f64>,
) -> f32 {
    crate::core::view::ron_view::parsing::resolve_val_f32(val, Some(current), player_data, time)
}

enum AlphaTweenKind {
    Sprite(f32, f32),
    ViewBox(f32, f32),
}

fn resolve_alpha_tween(
    entity: Entity,
    from: Option<&Value<f32>>,
    to: &Value<f32>,
    sprites: &Query<&Sprite>,
    ui_boxes: &Query<&ViewBox>,
    player_data: &crate::core::view::ron_view::parsing::PlayerDataView<'_>,
    time: f64,
) -> Option<AlphaTweenKind> {
    if let Ok(sprite) = sprites.get(entity) {
        let cur = sprite.color.alpha();
        let start = from.map_or(cur, |f| {
            resolve_tween_val_f32(f, cur, player_data, Some(time))
        });
        let end = resolve_tween_val_f32(to, cur, player_data, Some(time));
        return Some(AlphaTweenKind::Sprite(start, end));
    }
    if let Ok(ui_box) = ui_boxes.get(entity) {
        let cur = ui_box.alpha();
        let start = from.map_or(cur, |f| {
            resolve_tween_val_f32(f, cur, player_data, Some(time))
        });
        let end = resolve_tween_val_f32(to, cur, player_data, Some(time));
        return Some(AlphaTweenKind::ViewBox(start, end));
    }
    None
}

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
        let Chapter::TweenViewElement {
            selector,
            target,
            duration,
            easing,
            wait_for_completion,
        } = &active_chapter.chapter
        else {
            continue;
        };

        info!(
            "[TweenViewElement] Processing: selector={:?}, target={:?}, duration={}s",
            selector, target, duration
        );

        let target_entity = match selector {
            crate::core::sequencer::chapter_schema::ElementSelector::FullName(full_name) => {
                crate::core::view::find_element_by_full_name(&view_elements, full_name)
            }
            crate::core::sequencer::chapter_schema::ElementSelector::LocalName(local_name) => {
                view_elements
                    .iter()
                    .find(|(_, elem)| elem.local_name == *local_name)
                    .map(|(entity, _)| entity)
            }
            crate::core::sequencer::chapter_schema::ElementSelector::Tag(tag) => {
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
                        resolve_tween_val_f32(&from.0, current.x, &player_data, Some(current_time)),
                        resolve_tween_val_f32(&from.1, current.y, &player_data, Some(current_time)),
                        resolve_tween_val_f32(&from.2, current.z, &player_data, Some(current_time)),
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
                        resolve_tween_val_f32(&from.0, current.x, &player_data, Some(current_time)),
                        resolve_tween_val_f32(&from.1, current.y, &player_data, Some(current_time)),
                        resolve_tween_val_f32(&from.2, current.z, &player_data, Some(current_time)),
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
                        resolve_tween_val_f32(&from.0, current.x, &player_data, Some(current_time)),
                        resolve_tween_val_f32(&from.1, current.y, &player_data, Some(current_time)),
                        resolve_tween_val_f32(&from.2, current.z, &player_data, Some(current_time)),
                        resolve_tween_val_f32(&from.3, current.w, &player_data, Some(current_time)),
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
                let alpha_result = resolve_alpha_tween(
                    target_entity,
                    from.as_ref(),
                    to,
                    &sprites,
                    &ui_boxes,
                    &player_data,
                    current_time,
                );
                let Some((animator_entity, wait)) = alpha_result.map(|r| {
                    let id = match r {
                        AlphaTweenKind::Sprite(start, end) => commands
                            .spawn_empty()
                            .animation()
                            .insert(tween(
                                duration,
                                ease_kind,
                                target_component.with(SpriteAlphaInterpolator { start, end }),
                            ))
                            .id(),
                        AlphaTweenKind::ViewBox(start, end) => commands
                            .spawn_empty()
                            .animation()
                            .insert(tween(
                                duration,
                                ease_kind,
                                target_component.with(ViewBoxAlphaInterpolator { start, end }),
                            ))
                            .id(),
                    };
                    (id, *wait_for_completion)
                }) else {
                    warn!("[TweenViewElement] Target has neither Sprite nor ViewBox component");
                    commands.entity(chapter_entity).insert(ChapterFinished);
                    continue;
                };
                handle_wait_for_completion(&mut commands, chapter_entity, animator_entity, wait);
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
                        resolve_tween_val_f32(&from.0, current_w, &player_data, Some(current_time)),
                        resolve_tween_val_f32(&from.1, current_h, &player_data, Some(current_time)),
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
        commands.entity(chapter_entity).insert(ChapterFinished);
    }
}

pub fn process_tween_wait_chapter_system(
    mut commands: Commands,
    tween_chapters: Query<(Entity, &TweenInProgress), Without<ChapterFinished>>,
    time_runners: Query<&bevy_tween::bevy_time_runner::TimeRunner>,
) {
    for (chapter_entity, tween_progress) in tween_chapters.iter() {
        if let Ok(runner) = time_runners.get(tween_progress.animator_entity) {
            if runner.is_completed() {
                info!("[TweenViewElement] Tween completed");
                commands.entity(chapter_entity).insert(ChapterFinished);
                commands.entity(chapter_entity).remove::<TweenInProgress>();
                commands.entity(tween_progress.animator_entity).despawn();
            }
        } else {
            info!("[TweenViewElement] Tween animator no longer exists, marking as completed");
            commands.entity(chapter_entity).insert(ChapterFinished);
            commands.entity(chapter_entity).remove::<TweenInProgress>();
        }
    }
}
