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
use tracing_subscriber::EnvFilter;

/// Sets up the logging system with both stdout and file output.
#[cfg(not(feature = "trace_tracy"))]
pub(crate) fn setup_logging() -> anyhow::Result<tracing_appender::non_blocking::WorkerGuard> {
    use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};

    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    let filename = format!("souprune_{}.log", timestamp);
    let log_dir: std::path::PathBuf = if cfg!(target_os = "android") {
        "/sdcard/SoupRune".into()
    } else {
        "logs".into()
    };

    #[cfg(not(target_os = "android"))]
    std::fs::create_dir_all(&log_dir)?;

    let file_path = log_dir.join(filename);
    let file = std::fs::File::create(file_path)?;
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

    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    let filename = format!("souprune_{}.log", timestamp);
    let log_dir: std::path::PathBuf = if cfg!(target_os = "android") {
        "/sdcard/SoupRune".into()
    } else {
        "logs".into()
    };

    #[cfg(not(target_os = "android"))]
    std::fs::create_dir_all(&log_dir)?;

    let file_path = log_dir.join(filename);
    let file = std::fs::File::create(file_path)?;
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
