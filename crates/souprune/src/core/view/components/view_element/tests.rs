//! Tests for View element components.
//!
//! View 元素组件测试。

use super::*;
use bevy_fact_rule_event::FactValue;

#[test]
fn view_root_exposes_readonly_local_state_and_controlled_writes() {
    let mut view_root = ViewRoot::new("battle/menu.view.ron".to_string());

    view_root.set_local_value("selection", FactValue::Int(2));
    view_root.request_close();
    view_root.switch_state("dialogue");

    assert_eq!(view_root.local_state().get_int("selection"), Some(2));
    assert_eq!(
        view_root.local_state().get_bool("view:close_requested"),
        Some(true)
    );
    assert_eq!(
        view_root.local_state().get_string("view:switch_state"),
        Some("dialogue")
    );

    assert_eq!(
        view_root.remove_local_value("selection"),
        Some(FactValue::Int(2))
    );
    assert_eq!(view_root.local_state().get_int("selection"), None);
}
