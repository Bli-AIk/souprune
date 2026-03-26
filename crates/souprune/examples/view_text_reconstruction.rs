//! # view_text_reconstruction
//!
//! Interactive example for reconstructing a single View text block from a reference image.
//! It loads a task file, generates `.view.ron` candidates, renders them through the real View
//! pipeline, and compares the result against the source image.
//!
//! 用于根据参考图反推单个 View 文本块的交互式示例。
//! 它负责加载任务文件、生成 `.view.ron` 候选、走真实的 View 渲染链路出图，
//! 再把渲染结果与参考图进行比较。
//!
//! This file is the example entry point. It owns CLI parsing, workspace resolution, and app
//! startup wiring, while the runtime and search modules own the reconstruction behavior.
//!
//! 这个文件是示例入口。它负责命令行解析、工作区定位和应用启动接线；
//! 真正的重建运行时和搜索逻辑分别由 `runtime` 与 `search` 模块负责。
#[path = "view_text_reconstruction/config.rs"]
mod config;
#[path = "view_text_reconstruction/runtime.rs"]
mod runtime;
#[path = "view_text_reconstruction/search.rs"]
mod search;

use anyhow::{Context, Result};
use std::env;
use std::path::Path;
use std::path::PathBuf;

fn main() {
    if let Err(error) = try_main() {
        eprintln!("[view_text_reconstruction] {error:#}");
        std::process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let config_path = parse_config_path()?;
    let souprune_config = souprune::config::load_config();
    let task_config = config::TaskConfig::load(&config_path, &workspace_root())?;

    let mut app = bevy::prelude::App::new();
    runtime::configure_app(&mut app, souprune_config, task_config)?;
    let _ = app.run();

    Ok(())
}

fn workspace_root() -> PathBuf {
    dunce::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .expect("workspace root should exist")
}

fn parse_config_path() -> Result<PathBuf> {
    let mut args = env::args().skip(1);
    while let Some(argument) = args.next() {
        if argument == "--config" {
            let path = args.next().context("missing value for `--config`")?;
            return Ok(PathBuf::from(path));
        }
    }

    anyhow::bail!(
        "usage: cargo run -p souprune --example view_text_reconstruction -- --config <task.toml>"
    );
}
