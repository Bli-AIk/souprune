//! Configures Souprune's tracing pipeline for both local debugging and shipped builds.
//!
//! 配置 Souprune 的 tracing 日志管线，兼顾本地调试和发布运行。
//!
//! Owns startup-time logging policy: where logs are written, which
//! crates are filtered, and whether Tracy integration is attached. It exists so
//! the rest of the runtime can emit tracing events without caring about
//! platform-specific log destinations or profiler setup.
//!
//! 负责启动期的日志策略：日志写到哪里、哪些 crate 需要过滤、以及
//! 是否挂接 Tracy 分析。这样其他运行时模块只管产生日志事件，不需要关心
//! 平台差异、文件输出路径或性能分析器的初始化。

use chrono::Local;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[cfg(target_os = "android")]
fn android_private_base_path() -> PathBuf {
    crate::config::android_private_base_path()
}

#[cfg(target_os = "android")]
fn android_private_log_dir() -> PathBuf {
    android_private_base_path().join("logs")
}

#[cfg(target_os = "android")]
fn log_dir_candidates() -> Vec<PathBuf> {
    vec![android_private_log_dir()]
}

#[cfg(not(target_os = "android"))]
fn log_dir_candidates() -> Vec<PathBuf> {
    vec![PathBuf::from("logs")]
}

fn open_file_in_first_writable_dir(
    dirs: &[PathBuf],
    filename: &str,
) -> anyhow::Result<(std::fs::File, PathBuf)> {
    let mut errors = Vec::new();
    for dir in dirs {
        if let Err(err) = std::fs::create_dir_all(dir) {
            errors.push(format!("{}: {}", dir.display(), err));
            continue;
        }

        let path = dir.join(filename);
        match std::fs::File::create(&path) {
            Ok(file) => return Ok((file, path)),
            Err(err) => errors.push(format!("{}: {}", path.display(), err)),
        }
    }

    anyhow::bail!("no writable log directory found: {}", errors.join("; "))
}

#[cfg(target_os = "android")]
pub(crate) fn android_panic_log_path() -> Option<PathBuf> {
    let dir = android_private_log_dir();
    if std::fs::create_dir_all(&dir).is_ok() {
        return Some(dir.join("panic.log"));
    }
    None
}

fn open_timestamped_log_file(prefix: &str) -> anyhow::Result<(std::fs::File, PathBuf)> {
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    let filename = format!("{}_{}.log", prefix, timestamp);
    open_file_in_first_writable_dir(&log_dir_candidates(), &filename)
}

/// Sets up the logging system with both stdout and file output.
#[cfg(not(feature = "trace_tracy"))]
pub(crate) fn setup_logging() -> anyhow::Result<tracing_appender::non_blocking::WorkerGuard> {
    use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};

    let (file, file_path) = open_timestamped_log_file("souprune")?;
    #[cfg(target_os = "android")]
    eprintln!("[SoupRune] logging to {:?}", file_path);
    #[cfg(not(target_os = "android"))]
    let _ = file_path;
    let (non_blocking, guard) = tracing_appender::non_blocking(file);

    let filter = EnvFilter::builder()
        .with_default_directive(tracing::Level::INFO.into())
        .from_env_lossy()
        .add_directive("souprune=info".parse()?)
        .add_directive("wgpu=error".parse()?)
        .add_directive("naga=warn".parse()?)
        .add_directive("bevy=info".parse()?)
        .add_directive("bevy_render=warn".parse()?)
        .add_directive("bevy_app=warn".parse()?)
        .add_directive("bevy_ecs=warn".parse()?)
        .add_directive("bevy_alight_motion=warn".parse()?);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_writer(non_blocking)
        .with_filter(filter.clone());
    let stdout_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_filter(filter);

    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer)
        .init();

    Ok(guard)
}

/// Sets up the logging system with Tracy profiler support.
#[cfg(feature = "trace_tracy")]
pub(crate) fn setup_logging() -> anyhow::Result<tracing_appender::non_blocking::WorkerGuard> {
    use tracing_subscriber::prelude::*;

    let (file, file_path) = open_timestamped_log_file("souprune")?;
    #[cfg(target_os = "android")]
    eprintln!("[SoupRune] logging to {:?}", file_path);
    #[cfg(not(target_os = "android"))]
    let _ = file_path;
    let (non_blocking, guard) = tracing_appender::non_blocking(file);

    let filter = EnvFilter::builder()
        .with_default_directive(tracing::Level::INFO.into())
        .from_env_lossy()
        .add_directive("wgpu=error".parse()?)
        .add_directive("naga=warn".parse()?);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_writer(non_blocking)
        .with_filter(filter.clone());
    let stdout_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_filter(filter);
    let tracy_layer = tracing_tracy::TracyLayer::default();

    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer)
        .with(tracy_layer)
        .init();

    Ok(guard)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_writable_log_dir_is_created_and_used() {
        let root = std::env::temp_dir().join(format!("souprune-log-test-{}", std::process::id()));
        let bad_path = root.join("not_a_dir");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(&bad_path, b"file").unwrap();
        let good_dir = root.join("logs");

        let (file, path) =
            open_file_in_first_writable_dir(&[bad_path, good_dir.clone()], "souprune.log").unwrap();
        drop(file);

        assert_eq!(path, good_dir.join("souprune.log"));
        assert!(path.is_file());

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn reports_all_unwritable_log_dirs() {
        let root =
            std::env::temp_dir().join(format!("souprune-log-fail-test-{}", std::process::id()));
        let bad_path = root.join("not_a_dir");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(&bad_path, b"file").unwrap();

        let error = open_file_in_first_writable_dir(&[bad_path], "souprune.log").unwrap_err();
        assert!(
            error
                .to_string()
                .contains("no writable log directory found")
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn desktop_log_dir_candidate_is_relative() {
        #[cfg(not(target_os = "android"))]
        {
            assert_eq!(log_dir_candidates(), vec![PathBuf::from("logs")]);
        }
    }

    #[cfg(target_os = "android")]
    #[test]
    fn android_log_dir_candidate_uses_private_storage() {
        assert_eq!(
            log_dir_candidates(),
            vec![PathBuf::from(
                "/data/user/0/com.bliaik.souprune/files/SoupRune/logs"
            )]
        );
        assert_eq!(
            android_private_log_dir(),
            PathBuf::from("/data/user/0/com.bliaik.souprune/files/SoupRune/logs")
        );
    }
}
