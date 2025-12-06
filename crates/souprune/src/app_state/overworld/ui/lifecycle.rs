use super::components::{OverworldUI, UILayer};
use crate::extra::mortar::LocaleLoaded;
use bevy::prelude::*;

/// Spawn the root UI entity that drives the Undertale-style backpack menu.
///
/// 生成 Undertale 风背包菜单的根 UI 实体。
pub(crate) fn spawn_backpack_ui_system(
    mut commands: Commands,
    overworld_ui_query: Query<&OverworldUI>,
    locale_loaded: Option<Res<LocaleLoaded>>,
) {
    if locale_loaded.is_none() {
        return;
    }

    // Only create the UI if it does not exist yet and we are in the menu state.
    //
    // 仅在处于菜单状态且 UI 尚未存在时才创建 UI。
    if !overworld_ui_query.is_empty() {
        return;
    }

    // Dynamically compute `UILayer` total count.
    //
    // 动态获取 UILayer 的总数。
    let max_index = UILayer::BACKPACK_MENU_OPTIONS.len();

    commands.spawn((
        OverworldUI::new(UILayer::BACKPACK_MENU, max_index),
        // Add a Transform so the UI entity can be positioned.
        //
        // 添加 Transform 组件以便控制 UI 实体的位置。
        Transform::from_translation(Vec3::ZERO),
        Name::new("Backpack Menu UI"),
    ));

    info!("Spawned backpack UI in Menu state");
}
