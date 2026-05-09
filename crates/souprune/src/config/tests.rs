//! Tests for runtime project path configuration.
//!
//! 运行时项目路径配置的测试。

use super::*;
use std::path::PathBuf;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn desktop_projects_base_path_stays_relative() {
    let _guard = ENV_LOCK
        .lock()
        .expect("environment lock should be available");
    assert_eq!(get_projects_base_path(), PathBuf::from("projects"));
}

#[test]
fn android_private_base_path_points_to_app_storage() {
    let _guard = ENV_LOCK
        .lock()
        .expect("environment lock should be available");
    // SAFETY: These tests do not spawn threads that read this process
    // environment while the variable is being changed.
    unsafe {
        std::env::remove_var("SOUPRUNE_PRIVATE_ROOT");
    }
    assert_eq!(
        android_private_base_path(),
        PathBuf::from("/data/user/0/com.bliaik.souprune/files/SoupRune")
    );
    assert_eq!(
        android_private_base_path().join("projects"),
        PathBuf::from("/data/user/0/com.bliaik.souprune/files/SoupRune/projects")
    );
}

#[test]
fn android_private_base_path_can_come_from_environment() {
    let _guard = ENV_LOCK
        .lock()
        .expect("environment lock should be available");
    let root = PathBuf::from("/tmp/souprune-private-root-test");
    // SAFETY: These tests do not spawn threads that read this process
    // environment while the variable is being changed.
    unsafe {
        std::env::set_var("SOUPRUNE_PRIVATE_ROOT", &root);
    }
    assert_eq!(android_private_base_path(), root);
    // SAFETY: See safety note above.
    unsafe {
        std::env::remove_var("SOUPRUNE_PRIVATE_ROOT");
    }
}
