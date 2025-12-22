use super::chapter::Chapter;
use bevy::asset::{AssetLoader, LoadContext, io::Reader};
use bevy::prelude::*;
use bevy::tasks::ConditionalSendFuture;
use serde::{Deserialize, Serialize};

#[derive(Asset, TypePath, Debug, Clone, Deserialize, Serialize)]
pub struct BattleFlowAsset(pub Vec<Chapter>);

#[derive(Default)]
pub struct BattleFlowAssetLoader;

impl AssetLoader for BattleFlowAssetLoader {
    type Asset = BattleFlowAsset;
    type Settings = ();
    type Error = anyhow::Error;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let chapters = ron::de::from_bytes::<Vec<Chapter>>(&bytes)?;
            Ok(BattleFlowAsset(chapters))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["chapter.ron"]
    }
}
