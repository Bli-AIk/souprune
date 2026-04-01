//! SDF layer color toggle driven by FRE facts.
//!
//! FRE fact 驱动的 SDF 层颜色切换。

use bevy::prelude::*;
use bevy_alight_motion::sdf_material::SdfMaterial;
use bevy_fact_rule_event::LayeredFactDatabase;

/// Marks an SDF layer whose color is controlled by a boolean FRE fact.
///
/// 标记颜色由布尔 FRE fact 控制的 SDF 层。
#[derive(Component, Clone)]
pub struct FactToggleSdfColor {
    pub key: String,
    pub on: Color,
    pub off: Color,
}

/// Update SDF layer colors driven by FRE facts (`FactToggle` color source).
///
/// 更新由 FRE fact 驱动的 SDF 层颜色。
pub fn update_fact_toggle_sdf_colors_system(
    fact_db: Res<LayeredFactDatabase>,
    query: Query<(&FactToggleSdfColor, &MeshMaterial2d<SdfMaterial>)>,
    mut sdf_materials: ResMut<Assets<SdfMaterial>>,
) {
    if !fact_db.is_changed() {
        return;
    }

    for (toggle, mat_handle) in query.iter() {
        let Some(material) = sdf_materials.get_mut(&mat_handle.0) else {
            continue;
        };
        let is_on = fact_db.get_bool(&toggle.key).unwrap_or(false);
        let target = if is_on { toggle.on } else { toggle.off };
        let alpha = material.uniform_data.color.w;
        let srgba = target.to_srgba();
        material.uniform_data.color = Vec4::new(srgba.red, srgba.green, srgba.blue, alpha);
    }
}
