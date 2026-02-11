//! # markdown.rs
//!
//! # markdown.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module handles Markdown files as Bevy assets.
//!
//! 该模块负责将 Markdown 文件作为 Bevy 资产处理。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It defines `MarkdownPlugin`, `MarkdownAsset`, and `MarkdownAssetLoader` for registering and handling Markdown files.
//!
//! 本文件定义了 `MarkdownPlugin`、`MarkdownAsset` 与 `MarkdownAssetLoader`，用于注册并处理 Markdown 文件。

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use bevy::tasks::ConditionalSendFuture;
pub struct MarkdownPlugin;

impl Plugin for MarkdownPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<MarkdownAsset>()
            .init_asset_loader::<MarkdownAssetLoader>();
    }
}
#[derive(Asset, TypePath, Debug)]
pub struct MarkdownAsset;
#[derive(Default, TypePath)]
pub struct MarkdownAssetLoader;

impl AssetLoader for MarkdownAssetLoader {
    type Asset = MarkdownAsset;
    type Settings = ();
    type Error = std::io::Error;

    fn load(
        &self,
        _reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> impl ConditionalSendFuture<
        Output = std::result::Result<<Self as AssetLoader>::Asset, <Self as AssetLoader>::Error>,
    > {
        info!(
            "Successfully 'loaded' (ignored) markdown file: {:?}",
            load_context.path()
        );

        Box::pin(async move { Ok(MarkdownAsset) })
    }

    fn extensions(&self) -> &[&str] {
        &["md"]
    }
}
