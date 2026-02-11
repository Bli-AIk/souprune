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
