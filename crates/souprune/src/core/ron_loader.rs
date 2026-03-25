//! Provides the generic RON asset loader used by Souprune runtime asset types.
//!
//! 提供 Souprune 运行时各类 RON 资产共用的通用加载器。
//!
//! Allows asset modules to declare their schema type once and reuse the same
//! loader implementation for `.ron`-backed assets. It is
//! infrastructure rather than gameplay logic: read bytes, deserialize RON, and
//! hand Bevy a strongly typed asset.
//!
//! 让各个资产模块只声明自己的 schema 类型，并复用同一份 `.ron`
//! 资产加载实现。它属于基础设施而不是玩法逻辑：读取字节、反序列化 RON，
//! 再把强类型资产交给 Bevy。

use bevy::asset::io::Reader;
use bevy::asset::{Asset, AssetLoader, LoadContext};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::tasks::ConditionalSendFuture;
use serde::Deserialize;
use std::marker::PhantomData;

/// A generic asset loader for RON files.
///
/// 泛型 RON 文件资产加载器。
pub struct RonAssetLoader<A: TypePath> {
    extensions: &'static [&'static str],
    _marker: PhantomData<A>,
}

impl<A: TypePath> TypePath for RonAssetLoader<A> {
    fn type_path() -> &'static str {
        // Use concat! with type_ident for a reasonable approximation
        "souprune::core::ron_loader::RonAssetLoader"
    }

    fn short_type_path() -> &'static str {
        "RonAssetLoader"
    }
}

impl<A: TypePath> RonAssetLoader<A> {
    pub fn new(extensions: &'static [&'static str]) -> Self {
        Self {
            extensions,
            _marker: PhantomData,
        }
    }
}

impl<A> AssetLoader for RonAssetLoader<A>
where
    A: Asset + for<'de> Deserialize<'de> + Send + Sync + TypePath + 'static,
{
    type Asset = A;
    type Settings = ();
    type Error = anyhow::Error;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let asset = ron::de::from_bytes::<A>(&bytes)?;
            Ok(asset)
        })
    }

    fn extensions(&self) -> &[&str] {
        self.extensions
    }
}
