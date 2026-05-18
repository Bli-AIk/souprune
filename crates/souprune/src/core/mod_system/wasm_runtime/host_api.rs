//! Implementation of generated WIT host-api imports for Souprune mods.
//!
//! Translates calls from WASM into callback-local state changes and queued host
//! effects that Bevy systems can apply after the callback returns.
//!
//! Souprune mod 生成的 WIT host-api 导入实现。
//! 将 WASM 调用转换为回调局部状态变化，以及回调返回后由 Bevy 系统提交的宿主副作用。

use bevy::prelude::*;

use super::conversions::{
    action_to_index, fre_to_wit_fact, is_valid_positive_vec2, wit_rgba_to_color, wit_to_fre_fact,
    wit_to_physics_collider, wit_to_trigger_collider,
};
use super::effects::{PendingHostEffect, PendingSoundEffect};
use super::host_state::HostState;
use super::souprune::plugin::host_api::{
    Action as WitAction, ColliderShape as WitColliderShape, FactValue as WitFact,
    SpriteEntityDesc as WitSpriteEntityDesc, Vec2 as WitVec2,
};
use crate::core::collision::{CollisionBoundary, CollisionRegion, ConstraintHandle, RegionHandle};

impl super::souprune::plugin::host_api::Host for HostState {
    fn log(&mut self, level: u32, message: String) {
        match level {
            0 => info!("[MOD] {}", message),
            1 => warn!("[MOD] {}", message),
            _ => error!("[MOD] {}", message),
        }
    }

    fn is_action_pressed(&mut self, action: WitAction) -> bool {
        let idx = action_to_index(action);
        self.call_ctx.input_pressed[idx]
    }

    fn is_action_just_pressed(&mut self, action: WitAction) -> bool {
        let idx = action_to_index(action);
        self.call_ctx.input_just_pressed[idx]
    }

    fn set_velocity(&mut self, velocity: WitVec2) {
        self.call_ctx.velocity = Vec2::new(velocity.x, velocity.y);
    }

    fn create_collision_region(&mut self, center: WitVec2, half_size: WitVec2) -> u64 {
        let center = Vec2::new(center.x, center.y);
        let half_size = Vec2::new(half_size.x, half_size.y);
        if !is_valid_positive_vec2(half_size) || !center.is_finite() {
            warn!(
                "Ignoring invalid collision region center={:?} half_size={:?}",
                center, half_size
            );
            return 0;
        }

        let Some(mut collision_regions) = self.collision_region_store() else {
            return 0;
        };
        collision_regions
            .create_region(CollisionRegion::new(CollisionBoundary {
                center,
                half_size,
            }))
            .raw()
    }

    fn remove_collision_region(&mut self, region: u64) {
        if let Some(mut collision_regions) = self.collision_region_store() {
            collision_regions.remove_region(RegionHandle::from_raw(region));
        }
    }

    fn set_collision_region_bounds(&mut self, region: u64, center: WitVec2, half_size: WitVec2) {
        let center = Vec2::new(center.x, center.y);
        let half_size = Vec2::new(half_size.x, half_size.y);
        if region == 0 || !center.is_finite() || !is_valid_positive_vec2(half_size) {
            warn!(
                "Ignoring invalid collision region update handle={} center={:?} half_size={:?}",
                region, center, half_size
            );
            return;
        }

        if let Some(mut collision_regions) = self.collision_region_store() {
            collision_regions.update_region(
                RegionHandle::from_raw(region),
                CollisionRegion::new(CollisionBoundary { center, half_size }),
            );
        }
    }

    fn create_movement_constraint(
        &mut self,
        region: u64,
        collider: WitColliderShape,
    ) -> Option<u64> {
        let collider = wit_to_physics_collider(collider)?;
        self.collision_region_store()?
            .create_movement_constraint(RegionHandle::from_raw(region), collider)
            .map(ConstraintHandle::raw)
    }

    fn remove_movement_constraint(&mut self, constraint: u64) {
        if let Some(mut collision_regions) = self.collision_region_store() {
            collision_regions.remove_movement_constraint(ConstraintHandle::from_raw(constraint));
        }
    }

    fn constrain_movement(&mut self, constraint: u64, position: WitVec2) -> Option<WitVec2> {
        self.collision_region_store()?
            .constrain_movement(
                ConstraintHandle::from_raw(constraint),
                Vec2::new(position.x, position.y),
            )
            .map(|pos| WitVec2 { x: pos.x, y: pos.y })
    }

    fn spawn_view_box(&mut self, center: WitVec2, size: WitVec2, border_width: f32) -> u64 {
        let center = Vec2::new(center.x, center.y);
        let size = Vec2::new(size.x, size.y);
        if !center.is_finite()
            || !is_valid_positive_vec2(size)
            || !border_width.is_finite()
            || border_width < 0.0
        {
            warn!(
                "Ignoring invalid ViewBox primitive center={:?} size={:?} border_width={}",
                center, size, border_width
            );
            return 0;
        }

        let handle = self.call_ctx.next_host_entity_handle();
        self.call_ctx
            .pending_host_effects
            .push(PendingHostEffect::SpawnViewBox {
                handle,
                center,
                size,
                border_width,
            });
        handle
    }

    fn spawn_sprite_entity(&mut self, desc: WitSpriteEntityDesc) -> u64 {
        let position = Vec2::new(desc.position.x, desc.position.y);
        let Some(color) = wit_rgba_to_color(desc.color) else {
            warn!("Ignoring sprite entity with invalid color");
            return 0;
        };
        if desc.texture.trim().is_empty() || !position.is_finite() || !desc.z.is_finite() {
            warn!(
                "Ignoring invalid sprite entity texture='{}' position={:?} z={}",
                desc.texture, position, desc.z
            );
            return 0;
        }

        let physics_collider = desc.physics_collider.and_then(wit_to_physics_collider);
        let trigger_collider = desc.trigger_collider.and_then(wit_to_trigger_collider);
        let handle = self.call_ctx.next_host_entity_handle();
        self.call_ctx
            .pending_host_effects
            .push(PendingHostEffect::SpawnSpriteEntity {
                handle,
                texture: desc.texture,
                position,
                z: desc.z,
                color,
                physics_collider,
                trigger_collider,
                behavior_id: desc.behavior_id,
                behavior_context: desc.behavior_context,
                bullet_target: desc.bullet_target,
                mode_scope: desc.mode_scope,
                name: desc.name,
            });
        handle
    }

    fn set_view_box_bounds(&mut self, handle: u64, center: WitVec2, size: WitVec2) {
        let center = Vec2::new(center.x, center.y);
        let size = Vec2::new(size.x, size.y);
        if handle == 0 || !center.is_finite() || !is_valid_positive_vec2(size) {
            warn!(
                "Ignoring invalid ViewBox bounds handle={} center={:?} size={:?}",
                handle, center, size
            );
            return;
        }

        self.call_ctx
            .pending_host_effects
            .push(PendingHostEffect::SetViewBoxBounds {
                handle,
                center,
                size,
            });
    }

    fn tween_view_box_bounds(
        &mut self,
        handle: u64,
        center: WitVec2,
        size: WitVec2,
        duration_secs: f32,
    ) {
        let center = Vec2::new(center.x, center.y);
        let size = Vec2::new(size.x, size.y);
        if handle == 0
            || !center.is_finite()
            || !is_valid_positive_vec2(size)
            || !duration_secs.is_finite()
            || duration_secs <= 0.0
        {
            warn!(
                "Ignoring invalid ViewBox tween handle={} center={:?} size={:?} duration={}",
                handle, center, size, duration_secs
            );
            return;
        }

        self.call_ctx
            .pending_host_effects
            .push(PendingHostEffect::TweenViewBoxBounds {
                handle,
                center,
                size,
                duration_secs,
            });
    }

    fn set_view_box_visible(&mut self, handle: u64, visible: bool) {
        if handle == 0 {
            warn!("Ignoring invalid ViewBox visibility handle=0");
            return;
        }

        self.call_ctx
            .pending_host_effects
            .push(PendingHostEffect::SetViewBoxVisible { handle, visible });
    }

    fn remove_entity(&mut self, handle: u64) {
        if handle == 0 {
            warn!("Ignoring invalid host entity handle=0");
            return;
        }

        self.call_ctx
            .pending_host_effects
            .push(PendingHostEffect::RemoveEntity { handle });
    }

    fn get_entity_position(&mut self) -> WitVec2 {
        WitVec2 {
            x: self.call_ctx.entity_position.x,
            y: self.call_ctx.entity_position.y,
        }
    }

    fn delta_time(&mut self) -> f32 {
        self.call_ctx.delta_time
    }

    fn get_fact(&mut self, key: String) -> Option<WitFact> {
        self.call_ctx.fact_snapshot.get(&key).map(fre_to_wit_fact)
    }

    fn set_fact(&mut self, key: String, value: WitFact) {
        self.call_ctx
            .pending_fact_mutations
            .push((key, wit_to_fre_fact(value)));
    }

    fn set_global_fact(&mut self, key: String, value: WitFact) {
        self.call_ctx
            .pending_global_fact_mutations
            .push((key, wit_to_fre_fact(value)));
    }

    fn emit_event(&mut self, event_name: String) {
        self.call_ctx.pending_events.push(event_name);
    }

    fn get_entity_position_by_tag(&mut self, tag: String) -> Option<WitVec2> {
        self.call_ctx
            .entity_positions_by_tag
            .get(&tag)
            .map(|pos| WitVec2 { x: pos.x, y: pos.y })
    }

    fn spawn_emitter(&mut self, pattern_id: String, position: WitVec2) -> u64 {
        let handle = self.call_ctx.emitter_handle_counter;
        self.call_ctx.emitter_handle_counter += 1;
        self.call_ctx
            .pending_emitter_spawns
            .push((pattern_id, Vec2::new(position.x, position.y)));
        handle
    }

    fn despawn_emitter(&mut self, handle: u64) {
        self.call_ctx.pending_emitter_despawns.push(handle);
    }

    fn open_view(&mut self, view_id: String) {
        self.call_ctx.pending_view_opens.push(view_id);
    }

    fn close_view(&mut self) {
        self.call_ctx.pending_view_close = true;
    }

    fn play_sound(&mut self, sound_key: String) {
        self.call_ctx
            .pending_sounds
            .push(PendingSoundEffect::SoundKey(sound_key));
    }

    fn play_sound_full_path(&mut self, sound_path: String) {
        self.call_ctx
            .pending_sounds
            .push(PendingSoundEffect::FullPath(sound_path));
    }

    fn get_current_mode(&mut self) -> Option<String> {
        self.call_ctx.current_mode.clone()
    }

    fn get_current_sub_state(&mut self) -> String {
        self.call_ctx.current_sub_state.clone()
    }
}
