//! # restart.rs
//!
//! Debug system to restart the game by relaunching the process (F9).
//!
//! 使用 F9 通过重新启动进程来重启游戏。

#[cfg(feature = "debug")]
pub mod debug_restart {
    use bevy::prelude::*;

    pub(crate) fn setup_restart_debug(app: &mut App) {
        app.add_systems(Update, debug_restart_system);
    }

    fn debug_restart_system(input: Res<ButtonInput<KeyCode>>) {
        if !input.just_pressed(KeyCode::F9) {
            return;
        }

        info!("Debug: Restarting game (relaunching process)");

        let Ok(exe) = std::env::current_exe() else {
            error!("Failed to get current executable path");
            return;
        };

        let args: Vec<String> = std::env::args().skip(1).collect();
        let mut cmd = std::process::Command::new(&exe);
        cmd.args(&args);

        // Detach child's stdio so it doesn't inherit parent's terminal handles
        cmd.stdin(std::process::Stdio::null());

        match cmd.spawn() {
            Ok(_) => {
                std::process::exit(0);
            }
            Err(e) => {
                error!("Failed to relaunch process: {e}");
            }
        }
    }
}
