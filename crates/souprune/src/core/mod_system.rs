use bevy::prelude::*;
use libloading::{Library, Symbol};
use souprune_api::{
    Action, BehaviorInstance, ContextHandle, CreateBehaviorFn, GetBehaviorCountFn, GetBehaviorIdFn,
    HostApi,
};
use std::collections::HashMap;
use std::ffi::{CStr, CString, c_float, c_void};
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
        let ctx = &mut *(context as *mut BehaviorContext);
        ctx.velocity = Vec2::new(x, y);
    }
}

// === Context Structure ===

pub struct BehaviorContext {
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
        app.init_resource::<BehaviorRegistry>()
            .add_systems(Startup, load_mods_system)
            .add_systems(
                Update,
                (init_behaviors_system, update_behaviors_system)
                    .in_set(crate::app_state::battle::BattleUpdate),
            );
    }
}

#[derive(Resource, Default)]
pub struct BehaviorRegistry {
    // Keep libraries alive so symbols are valid
    libs: Vec<Library>,
    // Map ID to the Factory function that creates it
    factories: HashMap<String, CreateBehaviorFn>,
}

fn load_mods_system(mut registry: ResMut<BehaviorRegistry>) {
    // Hardcoded loading for now
    let lib_name = if cfg!(target_os = "windows") {
        "mod_example.dll"
    } else {
        "libmod_example.so"
    };
    let mod_path = format!("projects/example_mod/{}", lib_name);

    if !Path::new(&mod_path).exists() {
        warn!("Mod file not found: {}", mod_path);
        return;
    }

    unsafe {
        info!("Loading mod: {}", mod_path);
        let lib = Library::new(&mod_path).expect("Failed to load DLL");

        // 1. Get Count
        let get_count: Symbol<GetBehaviorCountFn> =
            lib.get(b"get_behavior_count").expect("No count fn found");
        let count = get_count();
        info!("Found {} Behaviors in DLL", count);

        // 2. Get IDs helper
        let get_id_fn: Symbol<GetBehaviorIdFn> =
            lib.get(b"get_behavior_id").expect("No ID fn found");

        // 3. Get Factory
        let create_fn: Symbol<CreateBehaviorFn> =
            lib.get(b"create_behavior").expect("No factory found");
        // Transmute the symbol to a function pointer so we can store it Copy
        let create_fn_ptr: CreateBehaviorFn = *create_fn;

        for i in 0..count {
            let id_ptr = get_id_fn(i);
            let id = CStr::from_ptr(id_ptr as *const i8)
                .to_string_lossy()
                .into_owned();

            info!("Registered Behavior: {}", id);
            registry.factories.insert(id, create_fn_ptr);
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
pub struct BehaviorParams {
    pub mode_id: String,
}

#[derive(Component, Default)]
pub struct BehaviorState {
    initialized: bool,
}

// This component holds the raw instance pointer.
// It implements Drop to ensure the heap memory in the SDK is freed.
#[derive(Component)]
pub struct ActiveBehavior {
    instance: BehaviorInstance,
}

unsafe impl Send for ActiveBehavior {}
unsafe impl Sync for ActiveBehavior {}

impl Drop for ActiveBehavior {
    fn drop(&mut self) {
        // Critical: Call destroy to free memory on the guest side
        if let Some(destroy) = self.instance.vtable.destroy {
            (destroy)(self.instance.instance);
        }
    }
}

// 简单的 Velocity 组件，之后应该合并到核心 Physics 组件中
#[derive(Component, Default)]
pub struct BehaviorVelocity(pub Vec2);

/// System to initialize new behaviors
fn init_behaviors_system(
    mut commands: Commands,
    mut query: Query<(Entity, &BehaviorParams, &mut BehaviorVelocity), Added<BehaviorParams>>,
    registry: Res<BehaviorRegistry>,
) {
    for (entity, params, mut velocity) in query.iter_mut() {
        if let Some(&create_fn) = registry.factories.get(&params.mode_id) {
            let c_id = CString::new(params.mode_id.clone()).unwrap();

            // Call factory to allocate instance
            let instance = unsafe { (create_fn)(c_id.as_ptr() as *const u8, &HOST_API_INSTANCE) };

            if instance.instance.is_null() {
                error!("Failed to create behavior instance for {}", params.mode_id);
                continue;
            }

            // Call on_enter immediately
            let mut ctx = BehaviorContext {
                entity,
                velocity: velocity.0,
            };
            let ctx_ptr = &mut ctx as *mut BehaviorContext as *mut ContextHandle;

            if let Some(on_enter) = instance.vtable.on_enter {
                (on_enter)(instance.instance, ctx_ptr);
            }

            // Sync back velocity
            velocity.0 = ctx.velocity;

            // Insert ActiveBehavior component
            commands.entity(entity).insert(ActiveBehavior { instance });
        } else {
            error!("Behavior ID not found: {}", params.mode_id);
        }
    }
}

/// System to update active behaviors
fn update_behaviors_system(
    mut query: Query<(
        Entity,
        &mut ActiveBehavior,
        &mut BehaviorVelocity,
        &mut Transform,
    )>,
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

    // 2. Iterate Active Behaviors
    for (entity, mut active, mut velocity, mut transform) in query.iter_mut() {
        let mut ctx = BehaviorContext {
            entity,
            velocity: velocity.0,
        };

        let ctx_ptr = &mut ctx as *mut BehaviorContext as *mut ContextHandle;

        // Call on_update via VTable, passing the instance pointer
        if let Some(on_update) = active.instance.vtable.on_update {
            (on_update)(active.instance.instance, ctx_ptr, time.delta_secs());
        }

        // Sync Back
        velocity.0 = ctx.velocity;

        // Apply Velocity to Transform
        transform.translation += velocity.0.extend(0.0) * time.delta_secs();
    }
}
