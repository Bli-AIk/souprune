//! # markdown.rs
//!
//! ## Module Overview
//! This module provides functionalities for handling Markdown files as assets within Bevy.
//!
//! ## Source File Overview
//! This file defines the `MarkdownPlugin`, `MarkdownAsset`, and `MarkdownAssetLoader` for registering and potentially processing Markdown files.
//!
//! ## 模块概述
//! 该模块提供了在 Bevy 中将 Markdown 文件作为资产处理的功能。
//!
//! ## 源文件概述
//! 该文件定义了 `MarkdownPlugin`、`MarkdownAsset` 和 `MarkdownAssetLoader`，用于注册和可能处理 Markdown 文件。

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
#[derive(Default)]
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