//! Wasmtime store state shared by all host-api callbacks.
//!
//! Carries WASI state, the per-call context, and shared primitive registries used
//! by callbacks while a mod is executing.
//!
//! 所有 host-api 回调共享的 Wasmtime Store 状态。
//! 承载 WASI 状态、单次调用上下文，以及 mod 执行期间回调使用的共享 primitive 注册表。

use bevy::prelude::*;
use std::sync::{Arc, Mutex};
use wasmtime::component::ResourceTable;
use wasmtime_wasi::WasiCtxBuilder;

use super::context::CallContext;
use crate::core::collision::CollisionRegionStore;

pub(super) type SharedCollisionRegionStore = Arc<Mutex<CollisionRegionStore>>;

/// Host state stored inside the Wasmtime `Store`.
pub struct HostState {
    pub wasi: wasmtime_wasi::WasiCtx,
    pub table: ResourceTable,
    pub call_ctx: CallContext,
    collision_regions: SharedCollisionRegionStore,
}

impl HostState {
    pub(super) fn new_for_mod(collision_regions: SharedCollisionRegionStore) -> Self {
        Self {
            wasi: WasiCtxBuilder::new().build(),
            table: ResourceTable::new(),
            call_ctx: CallContext::default(),
            collision_regions,
        }
    }

    pub(super) fn collision_region_store(
        &self,
    ) -> Option<std::sync::MutexGuard<'_, CollisionRegionStore>> {
        match self.collision_regions.lock() {
            Ok(store) => Some(store),
            Err(err) => {
                error!("Collision region store lock is poisoned: {err}");
                None
            }
        }
    }
}

impl wasmtime_wasi::WasiView for HostState {
    fn ctx(&mut self) -> wasmtime_wasi::WasiCtxView<'_> {
        wasmtime_wasi::WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}
