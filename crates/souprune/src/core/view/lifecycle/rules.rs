use crate::core::game_action::{GameFreAsset, GameRuleDef, GameRuleRegistry};
use bevy::prelude::*;

use super::super::components::ViewRoot;

pub(crate) fn cleanup_view_rules_system(
    mut removed_views: RemovedComponents<ViewRoot>,
    mut rule_registry: ResMut<GameRuleRegistry>,
) {
    for entity in removed_views.read() {
        rule_registry.clear_view(entity);
        info!(
            "[lifecycle] Cleared View-layer rules for despawned ViewRoot entity {:?}",
            entity
        );
    }
}

pub(crate) fn process_pending_view_rules_system(
    mut commands: Commands,
    fre_assets: Res<Assets<GameFreAsset>>,
    mut query: Query<(
        Entity,
        &mut super::super::components::PendingViewRules,
        &mut ViewRoot,
    )>,
    mut rule_registry: ResMut<GameRuleRegistry>,
    mortar_strings: Res<crate::extra::mortar::MortarStringTable>,
    mut enum_registry: ResMut<bevy_fact_rule_event::EnumRegistry>,
) {
    use crate::core::view::ron_view::spawn::load_fre_into_view_root;

    for (entity, mut pending, mut view_root) in query.iter_mut() {
        let mut loaded_paths = Vec::new();
        let mut still_pending = Vec::new();

        for (path, handle) in pending.pending_handles.drain(..) {
            let Some(fre_asset) = fre_assets.get(&handle) else {
                still_pending.push((path, handle));
                continue;
            };

            enum_registry.register_from_asset(fre_asset);
            load_fre_into_view_root(&mut view_root, fre_asset, &mortar_strings, &enum_registry);

            let rule_defs = fre_asset.get_rule_defs();
            let scope = fre_asset.scope();

            register_fre_rules(entity, &path, scope, rule_defs, &mut rule_registry);
            loaded_paths.push((path, rule_defs.len()));
        }

        for (path, count) in loaded_paths {
            info!(
                "[lifecycle] Processed pending FRE '{}': {} rules for entity {:?}",
                path, count, entity
            );
        }

        pending.pending_handles = still_pending;

        if pending.pending_handles.is_empty() {
            commands
                .entity(entity)
                .remove::<super::super::components::PendingViewRules>();

            view_root.local_facts.set(
                "view_rules_loaded",
                bevy_fact_rule_event::FactValue::Bool(true),
            );

            info!(
                "[lifecycle] All pending FRE files loaded for entity {:?}, removing PendingViewRules, set view_rules_loaded=true",
                entity
            );
        }
    }
}

fn register_fre_rules(
    entity: Entity,
    path: &str,
    scope: bevy_fact_rule_event::RuleScope,
    rule_defs: &[GameRuleDef],
    rule_registry: &mut GameRuleRegistry,
) {
    use bevy_fact_rule_event::RuleScope;

    for (idx, rule_def) in rule_defs.iter().enumerate() {
        let effective_scope = if scope == RuleScope::Local {
            RuleScope::View
        } else {
            scope
        };

        let rule = rule_def.to_rule_with_index(idx, effective_scope);
        let rule_id = rule_def.generate_id(idx);

        if effective_scope == RuleScope::View {
            info!(
                "[lifecycle] Registering View rule '{}' with {} outputs: {:?}",
                rule_id,
                rule.outputs.len(),
                rule.outputs
            );
            rule_registry.register_view_rule(entity, rule);
            info!(
                "[lifecycle] Registered pending View rule '{}' for entity {:?} from '{}'",
                rule_id, entity, path
            );
        } else {
            rule_registry.register(rule);
        }
    }
}
