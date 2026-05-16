//! # sequencer/camera.rs
//!
//! ## Module Overview
//!
//! Camera control systems for the battle sequencer.
//!
//! 战斗序列管理器的摄像机控制系统。

use super::chapter_schema::Chapter;
use super::context::*;
use bevy::prelude::*;

/// System to process camera actions.
///
/// 处理摄像机动作的系统。
pub fn process_camera_action_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), (Without<WaitTimer>, Without<ChapterFinished>)>,
    mut camera_query: Query<
        (Entity, &mut Transform, &mut Projection),
        With<crate::core::fixed_scene::FixedSceneCamera>,
    >,
    #[cfg(not(target_os = "android"))] resolution_scale: Option<
        Res<crate::core::camera::ResolutionScale>,
    >,
) {
    for (entity, active_chapter) in query.iter() {
        let Chapter::SetCamera(action) = &active_chapter.chapter else {
            continue;
        };
        for (_cam_entity, mut transform, mut proj) in camera_query.iter_mut() {
            match action {
                super::chapter_schema::CameraAction::SetPosition(pos) => {
                    transform.translation = pos.extend(transform.translation.z);
                }
                super::chapter_schema::CameraAction::SetZoom(zoom) => {
                    #[cfg(target_os = "android")]
                    let scale_factor = 1.0;
                    #[cfg(not(target_os = "android"))]
                    let scale_factor = resolution_scale
                        .as_ref()
                        .map(|r| r.get() as f32)
                        .unwrap_or(1.0);
                    apply_camera_zoom(&mut proj, *zoom, scale_factor);
                }
                _ => {
                    warn!("Camera action {:?} not implemented yet", action);
                }
            }
        }
        commands.entity(entity).insert(ChapterFinished);
    }
}

/// Apply zoom to an orthographic projection.
fn apply_camera_zoom(proj: &mut Projection, zoom: f32, scale_factor: f32) {
    let Projection::Orthographic(ortho) = proj else {
        return;
    };
    ortho.scale = zoom / scale_factor;
    info!(
        "[Battle] SetZoom: requested={}, actual={}",
        zoom, ortho.scale
    );
}
