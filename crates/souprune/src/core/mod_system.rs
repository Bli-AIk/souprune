use bevy::prelude::*;
use libloading::{Library, Symbol};
use souprune_api::{
    Action, ContextHandle, CreateSoulModeFn, GetSoulModeCountFn, GetSoulModeIdFn, HostApi,
    SoulModeVTable,
};
use std::collections::HashMap;
use std::ffi::{CStr, CString, c_float};
use std::path::Path;

// === Host API Implementation (Must be static / extern "C") ===

extern "C" fn host_log(_level: u32, msg: *const u8, len: usize) {
    unsafe {
        let slice = std::slice::from_raw_parts(msg, len);
        let s = String::from_utf8_lossy(slice);
        info!("[MOD] {}", s);
    }
}

extern "C" fn host_input_is_action_pressed(_context: *const ContextHandle, action: Action) -> bool {
    INPUT_SNAPSHOT.with(|snapshot| snapshot.borrow().is_pressed(action))
}

extern "C" fn host_kinematics_set_velocity(context: *mut ContextHandle, x: c_float, y: c_float) {
    unsafe {
        let ctx = &mut *(context as *mut SoulContext);
        ctx.velocity = Vec2::new(x, y);
    }
}

// === Context Structure ===

pub struct SoulContext {
    pub entity: Entity,
    pub velocity: Vec2,
}

// Thread Local Input Helper
use std::cell::RefCell;

#[derive(Default)]
struct InputSnapshot {
    pressed: [bool; 7], // Mapping Action enum
}

impl InputSnapshot {
    fn is_pressed(&self, action: Action) -> bool {
        self.pressed[action as usize]
    }
}

thread_local! {
    static INPUT_SNAPSHOT: RefCell<InputSnapshot> = RefCell::new(InputSnapshot::default());
}

// === Plugin ===

pub struct ModPlugin;

impl Plugin for ModPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SoulRegistry>()
            .add_systems(Startup, load_mods_system)
            .add_systems(
                Update,
                update_souls_system.in_set(crate::app_state::battle::BattleUpdate),
            );
    }
}

#[derive(Resource, Default)]
pub struct SoulRegistry {
    libs: Vec<Library>,
    modes: HashMap<String, SoulModeVTable>,
}

fn load_mods_system(mut registry: ResMut<SoulRegistry>) {
    // Hardcoded loading for now
    // In production, this should iterate over `projects/example_mod/*.so` or read `mod.toml`
    let lib_name = if cfg!(target_os = "windows") {
        "mod_example.dll"
    } else {
        "libmod_example.so"
    };
    // Changed path: mod directly in project root, not in mods/
    let mod_path = format!("projects/example_mod/{}", lib_name);

    if !Path::new(&mod_path).exists() {
        warn!("Mod file not found: {}", mod_path);
        return;
    }

    unsafe {
        info!("Loading mod: {}", mod_path);
        let lib = Library::new(&mod_path).expect("Failed to load DLL");

        // 1. Get Count
        let get_count: Symbol<GetSoulModeCountFn> =
            lib.get(b"get_soul_mode_count").expect("No count fn found");
        let count = get_count();
        info!("Found {} Soul Modes in DLL", count);

        // 2. Get IDs helper
        let get_id_fn: Symbol<GetSoulModeIdFn> =
            lib.get(b"get_soul_mode_id").expect("No ID fn found");

        // 3. Get Factory
        let create_fn: Symbol<CreateSoulModeFn> =
            lib.get(b"create_soul_mode").expect("No factory found");

        for i in 0..count {
            let id_ptr = get_id_fn(i);
            let id = CStr::from_ptr(id_ptr as *const i8)
                .to_string_lossy()
                .into_owned();

            // Create VTable for this ID
            let c_id = CString::new(id.clone()).unwrap();
            let vtable = create_fn(c_id.as_ptr() as *const u8, &HOST_API_INSTANCE);

            info!("Registered Soul Mode: {}", id);
            registry.modes.insert(id, vtable);
        }

        registry.libs.push(lib);
    }
}

// Static instance of HostApi
static HOST_API_INSTANCE: HostApi = HostApi {
    log: host_log,
    input_is_action_pressed: host_input_is_action_pressed,
    kinematics_set_velocity: host_kinematics_set_velocity,
};

// === Runtime System ===

#[derive(Component)]
pub struct SoulParams {
    pub mode_id: String,
}

#[derive(Component, Default)]
pub struct SoulState {
    initialized: bool,
}

// 简单的 Velocity 组件，之后应该合并到核心 Physics 组件中
#[derive(Component, Default)]
pub struct SoulVelocity(pub Vec2);

fn update_souls_system(
    mut query: Query<(
        Entity,
        &SoulParams,
        &mut SoulState,
        &mut SoulVelocity,
        &mut Transform,
    )>,
    registry: Res<SoulRegistry>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    // 1. Update Global Input Snapshot
    INPUT_SNAPSHOT.with(|s| {
        let mut snap = s.borrow_mut();
        snap.pressed = [false; 7];

        snap.pressed[Action::Up as usize] = input.pressed(KeyCode::ArrowUp);
        snap.pressed[Action::Down as usize] = input.pressed(KeyCode::ArrowDown);
        snap.pressed[Action::Left as usize] = input.pressed(KeyCode::ArrowLeft);
        snap.pressed[Action::Right as usize] = input.pressed(KeyCode::ArrowRight);
        snap.pressed[Action::Cancel as usize] =
            input.pressed(KeyCode::KeyX) || input.pressed(KeyCode::ShiftLeft);
        snap.pressed[Action::Confirm as usize] =
            input.pressed(KeyCode::KeyZ) || input.pressed(KeyCode::Enter);
    });

    // 2. Iterate Souls
    for (entity, params, mut state, mut velocity, mut transform) in query.iter_mut() {
        if let Some(vtable) = registry.modes.get(&params.mode_id) {
            let mut ctx = SoulContext {
                entity,
                velocity: velocity.0,
            };

            let ctx_ptr = &mut ctx as *mut SoulContext as *mut ContextHandle;

            // OnEnter
            if !state.initialized {
                if let Some(on_enter) = vtable.on_enter {
                    (on_enter)(ctx_ptr);
                }
                state.initialized = true;
            }

            // OnUpdate
            if let Some(on_update) = vtable.on_update {
                (on_update)(ctx_ptr, time.delta_secs());
            }

            // Sync Back
            velocity.0 = ctx.velocity;

            // Apply Velocity to Transform
            transform.translation += velocity.0.extend(0.0) * time.delta_secs();
        }
    }
}
