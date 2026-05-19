//! Loads the initial sequence flow asset and syncs it into the live sequence context.
//!
//! 加载初始序列流程资源，并把它同步进正在运行的序列上下文。
//!
//! Connects startup configuration to the sequencer runtime. It is
//! responsible for choosing the first sequence asset, waiting for it to load,
//! copying its chapters into the execution queue, and attaching any sequence-
//! scoped FRE rules that should accompany that flow.
//!
//! 把启动配置连接到 sequencer 运行时。它负责选择首个序列资源、
//! 等待资源加载完成、把其中的章节复制进执行队列，并接入该流程需要附带的
//! sequence 作用域 FRE 规则。

use bevy::prelude::*;

use super::super::SequenceAsset;
use super::super::context::{CurrentSequenceFlow, SequenceContext, SequenceRulesHandle};

/// System to sync the active sequence flow when its asset is loaded.
/// Also loads sequence-specific FRE rules if specified.
///
/// 当资源加载完成时同步当前 sequence flow 的系统。
/// 如果指定了规则文件，也会加载序列特定的 FRE 规则。
pub fn sync_sequence_flow_system(
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
