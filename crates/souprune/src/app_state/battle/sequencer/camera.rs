//! # sequencer/camera.rs
//!
//! ## Module Overview
//!
//! Camera control systems for the battle sequencer.
//!
//! 战斗序列管理器的摄像机控制系统。

use super::super::chapter_schema::Chapter;
use super::context::*;
use bevy::prelude::*;

/// System to process camera actions.
///
/// 处理摄像机动作的系统。
#[allow(clippy::type_complexity)]
pub fn process_camera_action_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), (Without<WaitTimer>, Without<ChapterFinished>)>,
    mut camera_query: Query<
        (Entity, &mut Transform, &mut Projection),
        With<crate::app_state::battle::BattleCamera>,
    >,
    resolution_scale: Res<crate::app_state::app_setup::ResolutionScale>,
) {
    for (entity, active_chapter) in query.iter() {
        if let Chapter::SetCamera(action) = &active_chapter.chapter {
            for (_cam_entity, mut transform, mut proj) in camera_query.iter_mut() {
                match action {
                    super::super::chapter_schema::CameraAction::SetPosition(pos) => {
                        transform.translation = pos.extend(transform.translation.z);
                    }
                    super::super::chapter_schema::CameraAction::SetZoom(zoom) => {
                        if let Projection::Orthographic(ortho) = &mut *proj {
                            // On Android with ScalingMode::Fixed, scale=1.0 already shows
                            // base resolution. On desktop with WindowSize, divide by resolution_scale.
                            #[cfg(target_os = "android")]
                            {
                                ortho.scale = *zoom;
                            }
                            #[cfg(not(target_os = "android"))]
                            {
                                ortho.scale = *zoom / resolution_scale.get() as f32;
                            }
                            info!(
                                "[Battle] SetZoom: requested={}, actual={}",
                                zoom, ortho.scale
                            );
                        }
                    }
                    _ => {
                        warn!("Camera action {:?} not implemented yet", action);
                    }
                }
            }
            commands.entity(entity).insert(ChapterFinished);
        }
    }
}
