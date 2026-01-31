//! Navigation and transition config for UI layers.
//!
//! UI 层的导航和转换配置。

use bevy::prelude::*;
use std::collections::HashMap;

use super::layer::ViewLayer;
use crate::core::input::Action;

/// Describes how directional inputs should modify the index of a [`ViewLayer`].
///
/// 描述方向输入应如何修改 [`ViewLayer`] 的索引。
#[derive(Debug, Clone)]
pub(crate) struct ViewLayerNavigationRule {
    adjustments: HashMap<Action, isize>,
    looping: bool,
    min_index: Option<IndexBound>,
    max_index: Option<IndexBound>,
    sound_on_navigate: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) enum IndexBound {
    Static(usize),
    Dynamic(String),
}

impl ViewLayerNavigationRule {
    pub(crate) fn new(pairs: impl IntoIterator<Item = (Action, isize)>) -> Self {
        Self {
            adjustments: pairs.into_iter().collect::<HashMap<_, _>>(),
            looping: false,
            min_index: None,
            max_index: None,
            sound_on_navigate: None,
        }
    }

    pub(crate) fn new_with_bounds(
        pairs: impl IntoIterator<Item = (Action, isize)>,
        looping: bool,
        min_index: Option<IndexBound>,
        max_index: Option<IndexBound>,
        sound_on_navigate: Option<String>,
    ) -> Self {
        Self {
            adjustments: pairs.into_iter().collect::<HashMap<_, _>>(),
            looping,
            min_index,
            max_index,
            sound_on_navigate,
        }
    }

    pub(crate) fn delta_for(&self, action: Action) -> Option<isize> {
        self.adjustments.get(&action).copied()
    }

    pub(crate) fn looping(&self) -> bool {
        self.looping
    }

    pub(crate) fn min_index(&self) -> &Option<IndexBound> {
        &self.min_index
    }

    pub(crate) fn max_index(&self) -> &Option<IndexBound> {
        &self.max_index
    }

    pub(crate) fn sound_on_navigate(&self) -> Option<&str> {
        self.sound_on_navigate.as_deref()
    }
}

/// Registry that stores the navigation rules for every [`ViewLayer`].
///
/// 存储每个 [`ViewLayer`] 导航规则的注册表。
#[derive(Resource, Debug, Default)]
pub(crate) struct ViewLayerNavigationConfig {
    rules: HashMap<ViewLayer, ViewLayerNavigationRule>,
}

impl ViewLayerNavigationConfig {
    pub(crate) fn get(&self, layer: &ViewLayer) -> Option<&ViewLayerNavigationRule> {
        self.rules.get(layer)
    }

    pub(crate) fn set_rule(&mut self, layer: ViewLayer, rule: ViewLayerNavigationRule) {
        self.rules.insert(layer, rule);
    }
}

impl Default for ViewLayerNavigationRule {
    fn default() -> Self {
        Self::new([])
    }
}

/// Stores state transition logic for UI layers, loaded from RON configuration.
///
/// 存储 UI 层的状态转换逻辑，从 RON 配置中加载。
#[derive(Resource, Debug, Default)]
pub(crate) struct ViewLayerTransitionConfig {
    transitions: HashMap<ViewLayer, LayerTransitions>,
}

#[derive(Debug, Clone)]
pub(crate) struct LayerTransitions {
    pub(crate) on_confirm: Vec<TransitionRule>,
    pub(crate) on_cancel: Option<TransitionAction>,
    pub(crate) sound_on_confirm: Option<String>,
    pub(crate) sound_on_cancel: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct TransitionRule {
    pub(crate) condition: Option<String>,
    pub(crate) action: TransitionAction,
}

#[derive(Debug, Clone)]
pub(crate) enum TransitionAction {
    GotoLayer(ViewLayer),
    PopState,
    PushState(String),
}

impl ViewLayerTransitionConfig {
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self {
            transitions: HashMap::new(),
        }
    }

    pub(crate) fn set_transitions(&mut self, layer: ViewLayer, transitions: LayerTransitions) {
        self.transitions.insert(layer, transitions);
    }

    pub(crate) fn get(&self, layer: &ViewLayer) -> Option<&LayerTransitions> {
        self.transitions.get(layer)
    }
}
