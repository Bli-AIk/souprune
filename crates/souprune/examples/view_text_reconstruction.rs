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
