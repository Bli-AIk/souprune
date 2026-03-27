//! # view_text_reconstruction
//!
//! ## TL;DR
//!
//! This is not a traditional example. It is a reconstruction helper tool.
//! This example is for aligning a View text block against a screenshot.
//! It can also use an evolutionary search loop to help fine-tune the alignment.
//!
//! ## 太长不看
//!
//! 这不是传统意义上的示例，而是一个重建辅助工具。
//! 这个示例就是拿来根据截图对齐 View 文本块的。
//! 同时也可以用进化搜索算法来辅助微调和对齐。
//!
//! ---
//!
//! Reconstructs one View text block from a reference image by using the real SoupRune View
//! pipeline instead of a separate mock renderer.
//!
//! 通过真实的 SoupRune View 渲染链，而不是另一套假的渲染器，从参考图反推一个
//! View 文本块。
//!
//! This example is the entry point for the text reconstruction tool.
//! It parses `--config`, resolves the workspace root, loads the task file, and boots the
//! Bevy application. The real search logic, runtime interaction, and scoring live in the
//! sibling `config`, `runtime`, and `search` modules.
//!
//! 这个文件是文本重建工具的入口。
//! 它负责解析 `--config`、定位工作区根目录、加载任务文件，并启动 Bevy 应用。
//! 真正的搜索逻辑、运行时交互和评分逻辑位于旁边的 `config`、`runtime` 和
//! `search` 模块中。
//!
//! ## When To Use It
//!
//! Use this example when you already have a screenshot and want a reusable View result:
//!
//! - match font, position, scale, and spacing against a reference
//! - edit the text content while keeping the same reconstruction workflow
//! - save the current state and continue later
//! - export a `.view.ron` result that can be inspected or integrated later
//!
//! ## 什么时候使用
//!
//! 当你已经有一张截图，并且希望得到可复用的 View 结果时，就使用这个示例：
//!
//! - 对齐字体、位置、缩放和间距
//! - 在同一套流程里直接修改文本内容
//! - 保存当前状态，并在下次继续
//! - 导出可继续检查或接入的 `.view.ron` 结果
//!
//! ## How It Works
//!
//! The tool turns candidate parameters into `.view.ron`, spawns that view through the normal
//! runtime View system, captures the result from Bevy, and compares it against the reference
//! image. If a saved current state exists, the next launch resumes from that state instead of
//! starting from the task defaults.
//!
//! ## 工作方式
//!
//! 这个工具会先把候选参数生成成 `.view.ron`，再通过正常的运行时 View 系统生成，
//! 从 Bevy 场景中截图，并与参考图比较。如果存在已保存的当前结果，下一次启动时会
//! 优先从该结果继续，而不是回到任务默认值。
//!
//! ## Basic Usage
//!
//! 1. Prepare a task TOML with a reference image and a `generated_view_path`.
//! 2. Run the example with `--config <task.toml>`.
//! 3. Adjust values manually in the inspector, or press `Space` to start and stop evolution.
//! 4. Press `S` to save the current state.
//! 5. Reopen the example later to continue from the saved current result.
//!
//! ## 基本使用方式
//!
//! 1. 准备任务 TOML，提供参考图和 `generated_view_path`。
//! 2. 使用 `--config <task.toml>` 启动示例。
//! 3. 在 Inspector 里手调，或者按 `Space` 启动 / 停止进化搜索。
//! 4. 按 `S` 保存当前状态。
//! 5. 下次重新打开时，会从已保存的当前结果继续。
//!
//! ## Saved Outputs
//!
//! Outputs are grouped by role instead of being flattened into one directory:
//!
//! - `current/view.ron`
//! - `current/summary.json`
//! - `current/render.png`
//! - `current/diff.png`
//! - `best/view.ron`
//! - `best/summary.json`
//! - `best/render.png`
//! - `best/diff.png`
//!
//! ## 保存产物
//!
//! 产物会按职责分组，而不是全部堆在同一层目录：
//!
//! - `current/view.ron`
//! - `current/summary.json`
//! - `current/render.png`
//! - `current/diff.png`
//! - `best/view.ron`
//! - `best/summary.json`
//! - `best/render.png`
//! - `best/diff.png`
//!
//! ## Main Controls
//!
//! - `Space`: start or cancel evolution
//! - `S`: save the current result
//! - `R`: restore the current best result
//! - `C`: switch the selected property
//! - `M`: change the manual step multiplier
//! - `G`: toggle grid snapping
//! - `Enter`: edit text when `content` is selected
//! - arrow keys: adjust the selected property
//!
//! ## 主要操作
//!
//! - `Space`：启动或取消进化搜索
//! - `S`：保存当前结果
//! - `R`：恢复当前最佳结果
//! - `C`：切换当前选中的属性
//! - `M`：切换手动步进倍率
//! - `G`：开关网格吸附
//! - `Enter`：在选中 `content` 时编辑文本
//! - 方向键：调整当前选中的属性
//!
//! # Examples
//!
//! ```bash
//! cargo run -p souprune --example view_text_reconstruction -- \
//!   --config generated/view_text_reconstruction/case_0/task.toml
//! ```
//!
//! ```bash
//! cargo run -p souprune --example view_text_reconstruction -- \
//!   --config path/to/task.toml
//! ```
//!
//! ## 示例
//!
//! ```bash
//! cargo run -p souprune --example view_text_reconstruction -- \
//!   --config generated/view_text_reconstruction/case_0/task.toml
//! ```
//!
//! ```bash
//! cargo run -p souprune --example view_text_reconstruction -- \
//!   --config path/to/task.toml
//! ```
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
