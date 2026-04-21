//! Wasmtime host for executing Vessel content components.
//!
//! 用于执行 Vessel 内容组件的 Wasmtime 宿主。

use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Engine, Store};
use wasmtime_wasi::WasiCtxBuilder;

wasmtime::component::bindgen!({
    path: "wit",
    world: "content-module",
});

/// A single generated RON file emitted by a Vessel content component.
///
/// Vessel 内容组件发射的一条 RON 文件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedRonFile {
    pub path: PathBuf,
    pub ron_text: String,
}

/// Summary returned after writing component output to disk.
///
/// 将组件输出写入磁盘后的摘要。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildSummary {
    pub component_path: PathBuf,
    pub output_dir: PathBuf,
    pub written_files: usize,
}

struct HostState {
    wasi: wasmtime_wasi::WasiCtx,
    table: ResourceTable,
}

impl wasmtime_wasi::WasiView for HostState {
    fn ctx(&mut self) -> wasmtime_wasi::WasiCtxView<'_> {
        wasmtime_wasi::WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

/// Load a Vessel content component and collect the files it emits.
///
/// 加载一个 Vessel 内容组件并收集它发射的文件。
pub fn load_component_files(component_path: impl AsRef<Path>) -> Result<Vec<GeneratedRonFile>> {
    let component_path = component_path.as_ref();

    let mut config = wasmtime::Config::new();
    config.wasm_component_model(true);

    let engine =
        Engine::new(&config).map_err(|err| anyhow!("failed to create wasmtime engine: {err}"))?;
    let component = Component::from_file(&engine, component_path).map_err(|err| {
        anyhow!(
            "failed to load component {}: {err}",
            component_path.display()
        )
    })?;

    let mut linker = Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker)
        .map_err(|err| anyhow!("failed to add wasi imports to linker: {err}"))?;

    let wasi = WasiCtxBuilder::new().build();
    let mut store = Store::new(
        &engine,
        HostState {
            wasi,
            table: ResourceTable::new(),
        },
    );

    let bindings = ContentModule::instantiate(&mut store, &component, &linker)
        .map_err(|err| anyhow!("failed to instantiate vessel content module: {err}"))?;

    let files = bindings
        .call_build(&mut store)
        .map_err(|err| anyhow!("guest build() call failed: {err}"))?;

    Ok(files
        .into_iter()
        .map(|file| GeneratedRonFile {
            path: PathBuf::from(file.path),
            ron_text: file.ron_text,
        })
        .collect())
}

/// Execute a Vessel content component and write its output under `output_dir`.
///
/// 执行 Vessel 内容组件并将其输出写入 `output_dir`。
pub fn build_component(
    component_path: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
) -> Result<BuildSummary> {
    let component_path = component_path.as_ref().to_path_buf();
    let output_dir = output_dir.as_ref().to_path_buf();

    let files = load_component_files(&component_path)?;
    crate::write_generated_files(&files, &output_dir)?;

    Ok(BuildSummary {
        component_path,
        output_dir,
        written_files: files.len(),
    })
}
