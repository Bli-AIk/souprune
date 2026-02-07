//! # shader_material.rs
//!
//! ## Module Overview
//! Runtime shader material component for dynamic 2D materials.
//!
//! ## 模块概述
//! 动态 2D 材质的运行时着色器材质组件。
//!
//! This component stores shader handles, parameter expressions, and animation state
//! for entities using DynamicMaterial2d.
//!
//! 此组件为使用 DynamicMaterial2d 的实体存储着色器句柄、参数表达式和动画状态。

use bevy::prelude::*;
use bevy::shader::Shader;
use std::collections::HashMap;

use crate::core::view::layout::view_schema::{EasingDef, MaterialDef, MaterialParamValue};

/// Generic Shader Material Component.
/// Stores runtime state for entities with DynamicMaterial2d.
///
/// 通用着色器材质组件。
/// 为使用 DynamicMaterial2d 的实体存储运行时状态。
#[derive(Component)]
pub struct ShaderMaterial {
    /// Loaded shader handle.
    ///
    /// 已加载的着色器句柄。
    pub shader: Handle<Shader>,

    /// Parameter definitions from RON config.
    /// Maps parameter name -> expression/static value.
    ///
    /// RON 配置中的参数定义。
    /// 参数名 -> 表达式/静态值的映射。
    pub param_defs: HashMap<String, MaterialParamValue>,

    /// Current evaluated parameter values.
    /// Updated each frame by update_shader_materials_system.
    ///
    /// 当前评估的参数值。
    /// 由 update_shader_materials_system 每帧更新。
    pub current_values: HashMap<String, f32>,

    /// Ordered list of parameter names for Vec4 packing.
    /// Elements 0-3 go to params, 4-7 go to extra_params.
    ///
    /// 用于 Vec4 打包的参数名有序列表。
    /// 元素 0-3 放入 params，4-7 放入 extra_params。
    pub param_order: Vec<String>,

    /// Animation state (optional).
    ///
    /// 动画状态（可选）。
    pub animation: Option<MaterialAnimationState>,
}

impl ShaderMaterial {
    /// Create a new ShaderMaterial from a MaterialDef.
    ///
    /// 从 MaterialDef 创建新的 ShaderMaterial。
    pub fn from_def(shader: Handle<Shader>, def: &MaterialDef) -> Self {
        let mut param_defs = HashMap::new();
        let mut param_order = Vec::new();
        let mut current_values = HashMap::new();

        for (name, value) in &def.params {
            param_defs.insert(name.clone(), value.clone());
            param_order.push(name.clone());

            // Initialize with default/static value
            let initial = match value {
                MaterialParamValue::Static(v) => *v,
                MaterialParamValue::Expr(_) => 0.0,
            };
            current_values.insert(name.clone(), initial);
        }

        // Ensure consistent ordering (sort by name)
        param_order.sort();

        let animation = def.animations.as_ref().and_then(|anims| {
            anims.lag.as_ref().map(|lag| MaterialAnimationState {
                source_param: lag.source.clone(),
                target_param: lag.target.clone(),
                delay: lag.delay,
                duration: lag.duration,
                easing: lag.easing.clone(),
                // Runtime state
                lag_value: 1.0,
                last_source_value: 1.0,
                delay_timer: 0.0,
                anim_progress: 1.0,
            })
        });

        Self {
            shader,
            param_defs,
            current_values,
            param_order,
            animation,
        }
    }

    /// Pack current values into Vec4 for shader uniforms.
    ///
    /// 将当前值打包为 Vec4 用于着色器 uniform。
    pub fn pack_params(&self) -> Vec4 {
        let mut result = Vec4::ZERO;
        for (i, name) in self.param_order.iter().take(4).enumerate() {
            if let Some(value) = self.current_values.get(name) {
                result[i] = *value;
            }
        }
        result
    }

    /// Pack extra parameters (elements 4-7) into Vec4.
    ///
    /// 将额外参数（元素 4-7）打包为 Vec4。
    pub fn pack_extra_params(&self) -> Vec4 {
        let mut result = Vec4::ZERO;
        for (i, name) in self.param_order.iter().skip(4).take(4).enumerate() {
            if let Some(value) = self.current_values.get(name) {
                result[i] = *value;
            }
        }
        result
    }
}

/// Animation state for lag-style parameter animations.
///
/// 延迟风格参数动画的动画状态。
#[derive(Debug, Clone)]
pub struct MaterialAnimationState {
    /// Name of the source parameter to track.
    ///
    /// 要跟踪的源参数名。
    pub source_param: String,

    /// Name of the target parameter to update.
    ///
    /// 要更新的目标参数名。
    pub target_param: String,

    /// Delay before animation starts (seconds).
    ///
    /// 动画开始前的延迟（秒）。
    pub delay: f32,

    /// Animation duration (seconds).
    ///
    /// 动画时长（秒）。
    pub duration: f32,

    /// Easing function.
    ///
    /// 缓动函数。
    pub easing: EasingDef,

    // --- Runtime State / 运行时状态 ---
    /// Current lag value (interpolated).
    ///
    /// 当前延迟值（插值）。
    pub lag_value: f32,

    /// Last source value for change detection.
    ///
    /// 用于变化检测的上一个源值。
    pub last_source_value: f32,

    /// Delay countdown timer.
    ///
    /// 延迟倒计时计时器。
    pub delay_timer: f32,

    /// Animation progress (0.0 to 1.0).
    ///
    /// 动画进度（0.0 到 1.0）。
    pub anim_progress: f32,
}

impl MaterialAnimationState {
    /// Update the lag animation state.
    /// Returns the new lag value.
    ///
    /// 更新延迟动画状态。
    /// 返回新的延迟值。
    pub fn update(&mut self, source_value: f32, delta_time: f32) -> f32 {
        // Detect source value change
        if (source_value - self.last_source_value).abs() > f32::EPSILON {
            // Source changed, start delay countdown
            self.delay_timer = self.delay;
            self.anim_progress = 0.0;
            self.last_source_value = source_value;
        }

        // Update delay timer
        if self.delay_timer > 0.0 {
            self.delay_timer -= delta_time;
            return self.lag_value;
        }

        // Update animation
        if self.anim_progress < 1.0 {
            self.anim_progress += delta_time / self.duration;
            self.anim_progress = self.anim_progress.min(1.0);

            // Apply easing
            let t = self.apply_easing(self.anim_progress);

            // Interpolate from lag_value to source_value
            // Note: This is simplified; a proper implementation would store start_value
            let start_value = self.lag_value;
            self.lag_value = start_value + (source_value - start_value) * t;
        } else {
            self.lag_value = source_value;
        }

        self.lag_value
    }

    /// Apply easing function to progress value.
    ///
    /// 将缓动函数应用于进度值。
    fn apply_easing(&self, t: f32) -> f32 {
        match self.easing {
            EasingDef::Linear => t,
            EasingDef::InQuad => t * t,
            EasingDef::OutQuad => 1.0 - (1.0 - t) * (1.0 - t),
            EasingDef::InOutQuad => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            EasingDef::InCubic => t * t * t,
            EasingDef::OutCubic => 1.0 - (1.0 - t).powi(3),
            EasingDef::InOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            EasingDef::InCirc => 1.0 - (1.0 - t * t).sqrt(),
            EasingDef::OutCirc => (1.0 - (t - 1.0).powi(2)).sqrt(),
            EasingDef::InOutCirc => {
                if t < 0.5 {
                    (1.0 - (1.0 - (2.0 * t).powi(2)).sqrt()) / 2.0
                } else {
                    ((1.0 - (-2.0 * t + 2.0).powi(2)).sqrt() + 1.0) / 2.0
                }
            }
        }
    }
}
