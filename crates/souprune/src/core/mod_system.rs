use bevy::prelude::*;
use libloading::{Library, Symbol};
use souprune_api::{
    Action, BehaviorInstance, ContextHandle, CreateBehaviorFn, CreateDanmakuFn, DanmakuInstance,
    GetAlgorithmCountFn, GetAlgorithmIdFn, GetBehaviorCountFn, GetBehaviorIdFn, HostApi,
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
            .init_resource::<DanmakuRegistry>()
            .add_systems(Startup, load_mods_system)
            .add_systems(
                Update,
                (init_behaviors_system, update_behaviors_system)
                    .in_set(crate::app_state::battle::BattleMovementSet),
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

/// Registry for danmaku behavior factories loaded from mods.
/// Stores factory functions to create stateful bullet behavior instances.
///
/// 弹幕行为工厂注册表，从模组中加载。
/// 存储创建有状态弹幕行为实例的工厂函数。
#[derive(Resource, Default)]
pub struct DanmakuRegistry {
    factories: HashMap<String, CreateDanmakuFn>,
}

impl DanmakuRegistry {
    /// Create a new danmaku behavior instance by ID.
    /// Returns None if the ID is not registered.
    pub fn create(&self, id: &str) -> Option<DanmakuInstance> {
        let factory = self.factories.get(id)?;
        let id_cstring = CString::new(id).ok()?;
        // SAFETY: Factory function is loaded from a valid mod library
        let instance = unsafe { factory(id_cstring.as_ptr() as *const u8) };
        if instance.instance.is_null() {
            None
        } else {
            Some(instance)
        }
    }

    /// Check if an algorithm is registered.
    pub fn contains(&self, id: &str) -> bool {
        self.factories.contains_key(id)
    }

    /// Get all registered algorithm IDs.
    pub fn algorithm_ids(&self) -> impl Iterator<Item = &String> {
        self.factories.keys()
    }
}

fn load_mods_system(
    mut registry: ResMut<BehaviorRegistry>,
    mut danmaku_registry: ResMut<DanmakuRegistry>,
) {
    // TODO: Use config.project.mod_name instead of hardcoded "example_mod"
    let mod_dir_name = "example_mod";
    // Heuristic: try to match the dll name based on the directory name, 
    // but here we know the files are named 'mod_example_...' not 'mod_example_mod_...'
    // so we hardcode the stem for now to match existing files.
    let dll_stem = "example"; 

    let base_path = Path::new("projects").join(mod_dir_name);

    let mut candidate_filenames = Vec::new();
    if cfg!(target_os = "windows") {
        if cfg!(target_env = "msvc") {
            candidate_filenames.push(format!("mod_{}_msvc.dll", dll_stem));
            candidate_filenames.push(format!("mod_{}_gnu.dll", dll_stem));
        } else {
            candidate_filenames.push(format!("mod_{}_gnu.dll", dll_stem));
            candidate_filenames.push(format!("mod_{}_msvc.dll", dll_stem));
        }
    } else {
        candidate_filenames.push(format!("libmod_{}.so", dll_stem));
    }

    let mut loaded_path = None;
    for filename in &candidate_filenames {
        let p = base_path.join(filename);
        if p.exists() {
            loaded_path = Some(p);
            break;
        }
    }

    let Some(mod_path_buf) = loaded_path else {
        let msg = format!("Mod file not found. Checked in {:?} for {:?}", base_path, candidate_filenames);
        warn!("{}", msg);
        eprintln!("[Souprune] Warning: {}", msg);
        return;
    };

    // Use dunce to canonicalize the path (resolves absolute path, handles Windows UNC)
    // We pass a reference to avoid moving, though mod_path_buf is now PathBuf so it's fine.
    let mod_path = dunce::canonicalize(&mod_path_buf).unwrap_or(mod_path_buf);

    unsafe {
        info!("Loading mod: {}", mod_path.display());
        eprintln!("[Souprune] Attempting to load mod from: {}", mod_path.display());
        
        let lib = match Library::new(&mod_path) {
            Ok(l) => l,
            Err(e) => {
                let msg = format!("Failed to load DLL '{}': {:?}", mod_path.display(), e);
                error!("{}", msg);
                eprintln!("[Souprune] Error: {}", msg);
                return;
            }
        };

        // === Load Behaviors ===
        // 1. Get Count
        let get_count: Symbol<GetBehaviorCountFn> = match lib.get(b"get_behavior_count") {
            Ok(s) => s,
            Err(e) => {
                error!("Symbol 'get_behavior_count' not found: {:?}", e);
                return;
            }
        };
        let count = get_count();
        info!("Found {} Behaviors in DLL", count);
        eprintln!("[Souprune] Loaded {} behaviors from DLL", count);

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

        // === Load Danmaku Algorithms (new VTable-based API) ===
        if let Ok(get_algo_count) = lib.get::<GetAlgorithmCountFn>(b"get_algorithm_count") {
            let algo_count = get_algo_count();
            info!("Found {} Danmaku Algorithms in DLL", algo_count);
            eprintln!("[Souprune] Found {} Danmaku Algorithms", algo_count);

            if let (Ok(get_algo_id), Ok(create_danmaku)) = (
                lib.get::<GetAlgorithmIdFn>(b"get_algorithm_id"),
                lib.get::<CreateDanmakuFn>(b"create_danmaku"),
            ) {
                let create_danmaku_ptr: CreateDanmakuFn = *create_danmaku;

                for i in 0..algo_count {
                    let id_ptr = get_algo_id(i);
                    if id_ptr.is_null() {
                        continue;
                    }
                    let id = CStr::from_ptr(id_ptr as *const i8)
                        .to_string_lossy()
                        .into_owned();

                    info!("Registered Danmaku Algorithm: {}", id);
                    danmaku_registry.factories.insert(id, create_danmaku_ptr);
                }
            }
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
    for (entity, active, mut velocity, mut transform) in query.iter_mut() {
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
