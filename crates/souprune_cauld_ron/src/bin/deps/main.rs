//! CLI binary for resolving SoupRune mod dependency trees.
//!
//! 解析 SoupRune mod 依赖树的 CLI 二进制文件。

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "cauld-ron-deps")]
#[command(about = "Resolve mod dependency tree for build ordering")]
struct Cli {
    /// Name of the target mod.
    mod_name: String,
    /// Path to the projects directory.
    #[arg(long, default_value = "projects")]
    projects_root: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let order = souprune_cauld_ron::deps::resolve_deps(&cli.projects_root, &cli.mod_name)?;
    for name in &order {
        println!("{name}");
    }
    Ok(())
}
