//! Runs `SetViewElement` chapters by applying instant sets or creating runtime tweens for view entities.
//!
//! 执行 `SetViewElement` 章节，对 View 实体进行即时设置或创建运行时 tween。
//!
//! Acts as the set/tween executor for sequence chapters. It resolves the target
//! element, evaluates expression-backed endpoints against current data, and either
//! sets the property instantly (when no `duration`) or spawns animator entities
//! that block chapter progression until the tween has finished.
//!
//! 序列章节的 set/tween 执行器。它会解析目标元素、根据当前数据求出
//! 表达式驱动的起止值，然后在没有 `duration` 时直接设置属性，或者生成
//! 动画器实体在完成前阻塞流程继续前进。

use super::interpolators::{
    SpriteAlphaInterpolator, TweenInProgress, ViewBoxAlphaInterpolator, ViewBoxSizeInterpolator,
};
use crate::core::sequencer::chapter_schema::{Chapter, TweenTarget, Value};
use crate::core::sequencer::context::{ActiveChapter, ChapterFinished, WaitTimer};
use crate::core::view::components::{ViewBox, ViewBoxAnchor};
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
    sprites: &Query<&mut Sprite>,
    ui_boxes: &Query<&mut ViewBox>,
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

pub fn process_set_view_element_system(
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
    mut transforms: Query<&mut Transform>,
    mut sprites: Query<&mut Sprite>,
    mut ui_boxes: Query<&mut ViewBox>,
    layered_db: Res<bevy_fact_rule_event::LayeredFactDatabase>,
    time: Res<Time>,
) {
    use crate::core::view::ron_view::parsing::PlayerDataView;
    use bevy_tween::combinator::tween;
    use bevy_tween::interpolate::{angle_z, scale, sprite_color, translation};

    let player_data = PlayerDataView::new(&layered_db);
    let current_time = time.elapsed_secs_f64();

    for (chapter_entity, active_chapter) in active_chapters.iter() {
        let Chapter::SetViewElement {
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
            "[SetViewElement] Processing: selector={:?}, target={:?}, duration={:?}",
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
            warn!("[SetViewElement] Target element not found: {:?}", selector);
            commands.entity(chapter_entity).insert(ChapterFinished);
            continue;
        };

        // Anchor target: always instant, insert/update ViewBoxAnchor component.
        if let TweenTarget::Anchor(ax, ay) = target {
            let (base_offset, base_width, base_height) =
                if let Ok(transform) = transforms.get(target_entity) {
                    let (w, h) = ui_boxes
                        .get(target_entity)
                        .map(|b| (b.width(), b.height()))
                        .unwrap_or((0.0, 0.0));
                    (transform.translation, w, h)
                } else {
                    (Vec3::ZERO, 0.0, 0.0)
                };

            commands.entity(target_entity).insert(ViewBoxAnchor {
                anchor: (*ax, *ay),
                base_offset,
                base_width,
                base_height,
            });

            info!(
                "[SetViewElement] Set anchor ({}, {}) on entity {:?}",
                ax, ay, target_entity
            );
            commands.entity(chapter_entity).insert(ChapterFinished);
            continue;
        }

        // Instant set: no duration — apply immediately without animation.
        if duration.is_none() {
            apply_instant_set(
                &mut commands,
                chapter_entity,
                target_entity,
                target,
                &mut transforms,
                &mut sprites,
                &mut ui_boxes,
                &player_data,
                current_time,
            );
            continue;
        }

        let duration = Duration::from_secs_f32(duration.unwrap());
        let ease_kind = *easing;
        let target_component = target_entity.into_target();

        match target {
            TweenTarget::Position { from, to } => {
                let Ok(transform) = transforms.get(target_entity) else {
                    warn!("[SetViewElement] Target has no Transform component");
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
                    warn!("[SetViewElement] Target has no Transform component");
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
                    warn!("[SetViewElement] Target has no Transform component");
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
                    warn!("[SetViewElement] Target has no Sprite component");
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
                    warn!("[SetViewElement] Target has neither Sprite nor ViewBox component");
                    commands.entity(chapter_entity).insert(ChapterFinished);
                    continue;
                };
                handle_wait_for_completion(&mut commands, chapter_entity, animator_entity, wait);
            }
            TweenTarget::BoxSize { from, to } => {
                let Ok(ui_box) = ui_boxes.get(target_entity) else {
                    warn!("[SetViewElement] Target has no ViewBox component");
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
            TweenTarget::Anchor(..) => unreachable!(),
        }

        info!(
            "[SetViewElement] Started bevy_tween animation for entity {:?}",
            target_entity
        );
    }
}

/// Applies a target property instantly without animation.
fn apply_instant_set(
    commands: &mut Commands,
    chapter_entity: Entity,
    target_entity: Entity,
    target: &TweenTarget,
    transforms: &mut Query<&mut Transform>,
    sprites: &mut Query<&mut Sprite>,
    ui_boxes: &mut Query<&mut ViewBox>,
    player_data: &crate::core::view::ron_view::parsing::PlayerDataView<'_>,
    current_time: f64,
) {
    match target {
        TweenTarget::Position { to, .. } => {
            if let Ok(mut transform) = transforms.get_mut(target_entity) {
                let current = transform.translation;
                transform.translation = Vec3::new(
                    resolve_tween_val_f32(&to.0, current.x, player_data, Some(current_time)),
                    resolve_tween_val_f32(&to.1, current.y, player_data, Some(current_time)),
                    resolve_tween_val_f32(&to.2, current.z, player_data, Some(current_time)),
                );
            }
        }
        TweenTarget::Scale { to, .. } => {
            if let Ok(mut transform) = transforms.get_mut(target_entity) {
                let current = transform.scale;
                transform.scale = Vec3::new(
                    resolve_tween_val_f32(&to.0, current.x, player_data, Some(current_time)),
                    resolve_tween_val_f32(&to.1, current.y, player_data, Some(current_time)),
                    resolve_tween_val_f32(&to.2, current.z, player_data, Some(current_time)),
                );
            }
        }
        TweenTarget::Rotation { to, .. } => {
            if let Ok(mut transform) = transforms.get_mut(target_entity) {
                let (_, _, z) = transform.rotation.to_euler(EulerRot::XYZ);
                let end = resolve_tween_val_f32(to, z, player_data, Some(current_time));
                transform.rotation = Quat::from_rotation_z(end);
            }
        }
        TweenTarget::Color { to, .. } => {
            if let Ok(mut sprite) = sprites.get_mut(target_entity) {
                let c = sprite.color.to_srgba();
                let current = Vec4::new(c.red, c.green, c.blue, c.alpha);
                sprite.color = Color::srgba(
                    resolve_tween_val_f32(&to.0, current.x, player_data, Some(current_time)),
                    resolve_tween_val_f32(&to.1, current.y, player_data, Some(current_time)),
                    resolve_tween_val_f32(&to.2, current.z, player_data, Some(current_time)),
                    resolve_tween_val_f32(&to.3, current.w, player_data, Some(current_time)),
                );
            }
        }
        TweenTarget::Alpha { to, .. } => {
            if let Ok(mut sprite) = sprites.get_mut(target_entity) {
                let cur = sprite.color.alpha();
                let end = resolve_tween_val_f32(to, cur, player_data, Some(current_time));
                sprite.color.set_alpha(end);
            } else if let Ok(mut ui_box) = ui_boxes.get_mut(target_entity) {
                let cur = ui_box.alpha;
                let end = resolve_tween_val_f32(to, cur, player_data, Some(current_time));
                ui_box.alpha = end;
            }
        }
        TweenTarget::BoxSize { to, .. } => {
            if let Ok(mut ui_box) = ui_boxes.get_mut(target_entity) {
                let end_w =
                    resolve_tween_val_f32(&to.0, ui_box.width, player_data, Some(current_time));
                let end_h =
                    resolve_tween_val_f32(&to.1, ui_box.height, player_data, Some(current_time));
                ui_box.width = end_w;
                ui_box.height = end_h;
            }
        }
        TweenTarget::Anchor(..) => unreachable!(),
    }

    info!(
        "[SetViewElement] Instant set applied to entity {:?}",
        target_entity
    );
    commands.entity(chapter_entity).insert(ChapterFinished);
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
                info!("[SetViewElement] Tween completed");
                commands.entity(chapter_entity).insert(ChapterFinished);
                commands.entity(chapter_entity).remove::<TweenInProgress>();
                commands.entity(tween_progress.animator_entity).despawn();
            }
        } else {
            info!("[SetViewElement] Tween animator no longer exists, marking as completed");
            commands.entity(chapter_entity).insert(ChapterFinished);
            commands.entity(chapter_entity).remove::<TweenInProgress>();
        }
    }
}
