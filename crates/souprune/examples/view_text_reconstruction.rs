//! # view_text_reconstruction
//!
//! ## TL;DR
//!
//! This is not a traditional example. It is a reconstruction helper tool.
//! This example is for aligning a View text block against a screenshot.
//! It can also use an evolutionary search loop to help fine-tune the alignment.
//! The preferred workflow is now a staged RON session instead of a single legacy TOML task.
//!
//! ## 太长不看
//!
//! 这不是传统意义上的示例，而是一个重建辅助工具。
//! 这个示例就是拿来根据截图对齐 View 文本块的。
//! 同时也可以用进化搜索算法来辅助微调和对齐。
//! 现在更推荐使用分阶段的 RON 会话，而不是旧的单个 TOML 任务。
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
//! 1. Prefer a `session.ron` that points to `stage_1_*.ron` and `stage_2_*.ron`.
//! 2. Run the example with `--config <session.ron>`.
//! 3. In stage 1, align the first glyph, then keep fine-tuning manually if needed.
//! 4. Press `N` when the current stage is accepted and you want to advance.
//! 5. In stage 2, refine spacing for the full text, then press `N` again for the next text.
//! 6. Press `S` at any time to save the current state.
//!
//! ## 基本使用方式
//!
//! 1. 优先准备一个 `session.ron`，再让它指向 `stage_1_*.ron` 和 `stage_2_*.ron`。
//! 2. 使用 `--config <session.ron>` 启动示例。
//! 3. 阶段一先对齐首字，必要时继续手动微调。
//! 4. 当前阶段确认无误后，按 `N` 进入下一阶段。
//! 5. 阶段二再对完整文本做 spacing 微调，然后再按 `N` 进入下一个文本。
//! 6. 任意时刻都可以按 `S` 保存当前结果。
//!
//! ## Saved Outputs
//!
//! Outputs are grouped by stage and role instead of being flattened into one directory:
//!
//! - `stage_1_.../current/view.ron`
//! - `stage_1_.../current/runtime.view.ron`
//! - `stage_2_.../current/view.ron`
//! - `stage_2_.../current/runtime.view.ron`
//! - `.../summary.json`
//! - `.../render.png`
//! - `.../diff.png`
//!
//! Legacy single-task mode still writes `current/` and `best/`.
//!
//! ## 保存产物
//!
//! 产物会按阶段和职责分组，而不是全部堆在同一层目录：
//!
//! - `stage_1_.../current/view.ron`
//! - `stage_1_.../current/runtime.view.ron`
//! - `stage_2_.../current/view.ron`
//! - `stage_2_.../current/runtime.view.ron`
//! - `.../summary.json`
//! - `.../render.png`
//! - `.../diff.png`
//!
//! 旧的单任务模式仍然会写到 `current/` 和 `best/`。
//!
//! ## Main Controls
//!
//! - `Space`: start or cancel evolution
//! - `N`: accept the current stage and advance to the next stage or next text
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
//! - `N`：确认当前阶段，进入下一阶段或下一个文本
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
//!   --config generated/view_text_reconstruction/backpack_name_text/session.ron
//! ```
//!
//! ```bash
//! cargo run -p souprune --example view_text_reconstruction -- \
//!   --config path/to/session.ron
//! ```
//!
//! ## 示例
//!
//! ```bash
//! cargo run -p souprune --example view_text_reconstruction -- \
//!   --config generated/view_text_reconstruction/backpack_name_text/session.ron
//! ```
//!
//! ```bash
//! cargo run -p souprune --example view_text_reconstruction -- \
//!   --config path/to/session.ron
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
    let loaded_config = config::load_config(&config_path, &workspace_root())?;

    let mut app = bevy::prelude::App::new();
    runtime::configure_app(&mut app, souprune_config, loaded_config)?;
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
        "usage: cargo run -p souprune --example view_text_reconstruction -- --config <task.toml|session.ron|stage.ron>"
    );
}
