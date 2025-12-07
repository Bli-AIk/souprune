//! # asset_loader.rs
//!
//! # asset_loader.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module defines the asset loader for UI layout RON files.
//!
//! 本模块定义了 UI 布局 RON 文件的资源加载器。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It reads `.ui.ron` files and deserializes them into `UILayoutAsset` structures.
//!
//! 读取 `.ui.ron` 文件并将其反序列化为 `UILayoutAsset` 结构。

use super::layout::UILayoutAsset;
use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;

#[derive(Default)]
pub struct UILayoutAssetLoader;

impl AssetLoader for UILayoutAssetLoader {
    type Asset = UILayoutAsset;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let asset = ron::de::from_bytes::<UILayoutAsset>(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        Ok(asset)
    }

    fn extensions(&self) -> &[&str] {
        &["ui.ron"]
    }
}
