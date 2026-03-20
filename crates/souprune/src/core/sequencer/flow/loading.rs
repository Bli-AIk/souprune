use bevy::prelude::*;

use super::super::SequenceAsset;
use super::super::context::{CurrentSequenceFlow, SequenceContext, SequenceRulesHandle};

/// System to load the default chapter resource.
///
/// 加载默认章节资源的系统。
pub fn load_default_chapter_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    souprune_config: Res<crate::config::SoupruneConfig>,
) {
    let chapter_path = &souprune_config.game.initial_battle_path;
    let handle = asset_server.load::<SequenceAsset>(chapter_path);
    commands.insert_resource(CurrentSequenceFlow(handle));
    info!("Loading default sequence flow: {}", chapter_path);
}

/// System to sync battle flow when asset is loaded.
/// Also loads sequence-specific FRE rules if specified.
///
/// 当资产加载完成时同步战斗流程的系统。
/// 如果指定了规则文件，也会加载序列特定的 FRE 规则。
pub fn sync_battle_flow_system(
    mut commands: Commands,
    flow_handle: Option<Res<CurrentSequenceFlow>>,
    mut context: ResMut<SequenceContext>,
    assets: Res<Assets<SequenceAsset>>,
    asset_server: Res<AssetServer>,
    mut sequence_rules_handle: ResMut<SequenceRulesHandle>,
    mut sequence_mode: ResMut<crate::core::mode::SequenceMode>,
) {
    if let Some(handle) = flow_handle
        && let Some(asset) = assets.get(&handle.0)
        && context.chapters.is_empty()
    {
        info!(
            "Sequence flow loaded. Pushing {} chapters to queue.",
            asset.chapters.len()
        );
        context.chapters.extend(asset.chapters.clone());

        if let Some(mode) = &asset.mode {
            sequence_mode.0 = Some(mode.clone());
            info!("Sequence: Setting mode to '{}'", mode);
        }

        if let Some(rules_path) = &asset.rules_file {
            let rules_handle =
                asset_server.load::<crate::core::game_action::GameFreAsset>(rules_path.clone());
            sequence_rules_handle.handle = Some(rules_handle);
            sequence_rules_handle.registered = false;
            info!("Sequence FRE: Loading rules from {}", rules_path);
        }

        commands.remove_resource::<CurrentSequenceFlow>();
    }
}
