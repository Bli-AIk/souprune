//! Command-line interface for Vessel.
//!
//! Vessel 的命令行接口。

use crate::component_host::build_component;
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Vessel CLI entrypoint.
///
/// Vessel CLI 入口。
#[derive(Debug, Parser)]
#[command(name = "vessel")]
#[command(about = "Build RON files from WASM content modules")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// Supported Vessel commands.
///
/// Vessel 支持的命令。
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Execute a content component and write its generated RON files.
    ///
    /// 执行内容组件并写出其生成的 RON 文件。
    Build {
        /// Path to the WASM component.
        component: PathBuf,
        /// Output root directory.
        #[arg(long)]
        output: PathBuf,
    },
}

/// Parse CLI arguments and execute the selected command.
///
/// 解析命令行参数并执行所选命令。
pub fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Build { component, output } => {
            let summary = build_component(&component, &output)?;
            println!(
                "vessel: built {} file(s) from {} -> {}",
                summary.written_files,
                summary.component_path.display(),
                summary.output_dir.display()
            );
            Ok(())
        }
    }
}
