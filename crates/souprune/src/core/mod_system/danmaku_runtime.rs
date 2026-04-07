//! Adapts mod-provided danmaku instances to Souprune's battle runtime.
//!
//! 把 mod 提供的 danmaku 实例适配到 Souprune 的战斗运行时中。
//!
//! Acts as the danmaku-specific slice of the WASM mod system. It owns the
//! live instance wrappers, creates algorithm instances from the registry, and
//! translates Souprune bullet context/output values to and from the generated
//! WIT bindings.
//!
//! WASM mod 系统里专门面向 danmaku 的一层。它负责活跃实例包装、
//! 根据注册表创建算法实例，并在 Souprune 的子弹上下文/输出结构与生成出来的
//! WIT 绑定类型之间做双向转换。

use super::*;

/// Active WASM danmaku instance.
#[derive(Component)]
pub struct ActiveDanmaku {
    mod_index: usize,
    resource_handle: wasmtime::component::ResourceAny,
    initialized: bool,
    pub props: HashMap<String, f32>,
}

unsafe impl Send for ActiveDanmaku {}
unsafe impl Sync for ActiveDanmaku {}

/// Stack of active WASM danmaku instances.
#[derive(Component, Default)]
pub struct ActiveDanmakuStack {
    pub instances: Vec<ActiveDanmaku>,
}

unsafe impl Send for ActiveDanmakuStack {}
unsafe impl Sync for ActiveDanmakuStack {}

impl ActiveDanmakuStack {
    pub fn call_on_exit_all(&mut self, loaded_mods: &mut LoadedMods) {
        for instance in &mut self.instances {
            instance.call_on_exit(loaded_mods);
        }
    }
}

impl DanmakuRegistry {
    /// Create a new danmaku instance by algorithm ID.
    pub fn create(&self, id: &str, loaded_mods: &mut LoadedMods) -> Option<ActiveDanmaku> {
        let &mod_index = self.algorithm_mods.get(id)?;
        let loaded = loaded_mods.mods.get_mut(mod_index)?;

        let danmaku_iface = loaded.bindings.souprune_plugin_danmaku();
        let handle = match danmaku_iface
            .danmaku_instance()
            .call_constructor(&mut loaded.store, id)
        {
            Ok(h) => h,
            Err(e) => {
                error!("Failed to create danmaku '{}': {:?}", id, e);
                return None;
            }
        };

        Some(ActiveDanmaku {
            mod_index,
            resource_handle: handle,
            initialized: false,
            props: HashMap::new(),
        })
    }
}

impl ActiveDanmaku {
    pub fn new_with_props(
        mod_index: usize,
        resource_handle: wasmtime::component::ResourceAny,
        props: HashMap<String, f32>,
    ) -> Self {
        Self {
            mod_index,
            resource_handle,
            initialized: false,
            props,
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Returns the index of the WASM mod that owns this instance.
    pub fn mod_index(&self) -> usize {
        self.mod_index
    }

    /// Call on_enter via WASM.
    pub fn call_on_enter(
        &mut self,
        ctx: &souprune_api::BulletContext,
        loaded_mods: &mut LoadedMods,
    ) {
        if self.initialized {
            return;
        }
        let Some(loaded) = loaded_mods.mods.get_mut(self.mod_index) else {
            return;
        };

        let wit_ctx = to_wit_bullet_context(ctx);
        let danmaku_iface = loaded.bindings.souprune_plugin_danmaku();
        match danmaku_iface.danmaku_instance().call_on_enter(
            &mut loaded.store,
            self.resource_handle,
            &wit_ctx,
        ) {
            Ok(()) => self.initialized = true,
            Err(e) => error!("Danmaku on_enter failed: {:?}", e),
        }
    }

    /// Call on_update via WASM and return the output.
    pub fn call_on_update(
        &mut self,
        ctx: &souprune_api::BulletContext,
        loaded_mods: &mut LoadedMods,
    ) -> souprune_api::BulletOutput {
        let Some(loaded) = loaded_mods.mods.get_mut(self.mod_index) else {
            return souprune_api::BulletOutput::ZERO;
        };

        let wit_ctx = to_wit_bullet_context(ctx);
        let danmaku_iface = loaded.bindings.souprune_plugin_danmaku();
        match danmaku_iface.danmaku_instance().call_on_update(
            &mut loaded.store,
            self.resource_handle,
            &wit_ctx,
        ) {
            Ok(output) => souprune_api::BulletOutput {
                offset: souprune_api::Vec2::new(output.offset.x, output.offset.y),
                rotation: output.rotation,
                opacity: output.opacity,
                scale_x: output.scale_x,
                scale_y: output.scale_y,
            },
            Err(e) => {
                error!("Danmaku on_update failed: {:?}", e);
                souprune_api::BulletOutput::ZERO
            }
        }
    }

    /// Call on_exit via WASM.
    pub fn call_on_exit(&mut self, loaded_mods: &mut LoadedMods) {
        if !self.initialized {
            return;
        }
        let Some(loaded) = loaded_mods.mods.get_mut(self.mod_index) else {
            return;
        };

        let danmaku_iface = loaded.bindings.souprune_plugin_danmaku();
        if let Err(e) = danmaku_iface
            .danmaku_instance()
            .call_on_exit(&mut loaded.store, self.resource_handle)
        {
            error!("Danmaku on_exit failed: {:?}", e);
        }
    }
}

fn to_wit_bullet_context(
    ctx: &souprune_api::BulletContext,
) -> wasm_runtime::exports::souprune::plugin::danmaku::BulletContext {
    wasm_runtime::exports::souprune::plugin::danmaku::BulletContext {
        elapsed: ctx.elapsed,
        delta_time: ctx.delta_time,
        spawn_pos: wasm_runtime::souprune::plugin::host_api::Vec2 {
            x: ctx.spawn_pos.x,
            y: ctx.spawn_pos.y,
        },
        offset: wasm_runtime::souprune::plugin::host_api::Vec2 {
            x: ctx.offset.x,
            y: ctx.offset.y,
        },
        initial_angle: ctx.initial_angle,
        initial_radius: ctx.initial_radius,
        player_pos: wasm_runtime::souprune::plugin::host_api::Vec2 {
            x: ctx.player_pos.x,
            y: ctx.player_pos.y,
        },
        props: ctx
            .props
            .iter()
            .map(|p| wasm_runtime::exports::souprune::plugin::danmaku::Prop {
                name: p.name.clone(),
                value: p.value,
            })
            .collect(),
    }
}
