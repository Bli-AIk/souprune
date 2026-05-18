//! Conversions between WIT host-api types and framework runtime types.
//!
//! Keeps the generated WIT surface isolated from Bevy/FRE data structures used
//! by the rest of the framework.
//!
//! WIT host-api 类型与框架运行时类型之间的转换。
//! 将生成的 WIT 接口与框架其余部分使用的 Bevy/FRE 数据结构隔离开。

use bevy::prelude::*;
use bevy_fact_rule_event::FactValue;
use souprune_api::Action;

use super::souprune::plugin::host_api::{
    Action as WitAction, ColliderShape as WitColliderShape, FactValue as WitFact, Rgba as WitRgba,
};
use crate::core::collision::{PhysicsCollider, TriggerCollider};

pub(super) fn action_to_index(action: WitAction) -> usize {
    match action {
        WitAction::Up => Action::Up as usize,
        WitAction::Down => Action::Down as usize,
        WitAction::Left => Action::Left as usize,
        WitAction::Right => Action::Right as usize,
        WitAction::Confirm => Action::Confirm as usize,
        WitAction::Cancel => Action::Cancel as usize,
        WitAction::Menu => Action::Menu as usize,
    }
}

pub(super) fn wit_to_physics_collider(collider: WitColliderShape) -> Option<PhysicsCollider> {
    match collider {
        WitColliderShape::Circle(radius) if radius.is_finite() && radius > 0.0 => {
            Some(PhysicsCollider::Circle { radius })
        }
        WitColliderShape::Rectangle(half_size) => {
            let half_size = Vec2::new(half_size.x, half_size.y);
            is_valid_positive_vec2(half_size).then_some(PhysicsCollider::Box { half_size })
        }
        _ => None,
    }
}

pub(super) fn wit_to_trigger_collider(collider: WitColliderShape) -> Option<TriggerCollider> {
    match collider {
        WitColliderShape::Circle(radius) if radius.is_finite() && radius > 0.0 => {
            Some(TriggerCollider::Circle { radius })
        }
        WitColliderShape::Rectangle(half_size) => {
            let half_size = Vec2::new(half_size.x, half_size.y);
            is_valid_positive_vec2(half_size).then_some(TriggerCollider::Box { half_size })
        }
        _ => None,
    }
}

pub(super) fn wit_rgba_to_color(color: WitRgba) -> Option<Color> {
    [color.red, color.green, color.blue, color.alpha]
        .iter()
        .all(|component| component.is_finite())
        .then(|| Color::srgba(color.red, color.green, color.blue, color.alpha))
}

pub(super) fn is_valid_positive_vec2(value: Vec2) -> bool {
    value.is_finite() && value.x > 0.0 && value.y > 0.0
}

/// Convert FRE `FactValue` to the WIT-generated `FactValue` variant.
pub(super) fn fre_to_wit_fact(v: &FactValue) -> WitFact {
    match v {
        FactValue::Int(n) => WitFact::IntVal(*n),
        FactValue::Float(f) => WitFact::FloatVal(*f),
        FactValue::Bool(b) => WitFact::BoolVal(*b),
        FactValue::String(s) => WitFact::TextVal(s.clone()),
        FactValue::StringList(list) => WitFact::TextList(list.clone()),
        FactValue::IntList(list) => WitFact::IntList(list.clone()),
        FactValue::FloatList(list) => WitFact::FloatList(list.clone()),
        FactValue::BoolList(list) => WitFact::BoolList(list.clone()),
    }
}

/// Convert WIT-generated `FactValue` variant to FRE `FactValue`.
pub(super) fn wit_to_fre_fact(v: WitFact) -> FactValue {
    match v {
        WitFact::IntVal(n) => FactValue::Int(n),
        WitFact::FloatVal(f) => FactValue::Float(f),
        WitFact::BoolVal(b) => FactValue::Bool(b),
        WitFact::TextVal(s) => FactValue::String(s),
        WitFact::IntList(list) => FactValue::IntList(list),
        WitFact::FloatList(list) => FactValue::FloatList(list),
        WitFact::BoolList(list) => FactValue::BoolList(list),
        WitFact::TextList(list) => FactValue::StringList(list),
    }
}
