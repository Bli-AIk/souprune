use crate::extra::toml::{AnimationConfig, SpriteConfig};
use bevy::log::info;
use bevy::platform::collections::HashMap;
use bevy::prelude::Resource;

/// A resource that stores TOML configuration data for all modules
///
/// 存储所有模块 TOML 配置数据的资源
#[derive(Resource, Default)]
pub struct TomlConfigRegistry {
    pub sprite_animations: HashMap<String, AnimationConfig>,
    pub sprites: HashMap<String, SpriteConfig>,
    pub modules: HashMap<String, Vec<String>>,
    pub registered_modules: std::collections::HashSet<String>,
}

impl TomlConfigRegistry {
    pub fn register_module_config(
        &mut self,
        module_name: &str,
        config: &crate::extra::toml::TomlConfig,
    ) {
        // Check if this module has already been registered
        //
        // 检查此模块是否已注册
        if self.registered_modules.contains(module_name) {
            return;
        }

        for anim in &config.animations {
            self.sprite_animations
                .insert(anim.name.clone(), anim.clone());
        }

        for sprite in &config.sprites {
            self.sprites.insert(sprite.name.clone(), sprite.clone());
        }

        for module in &config.module {
            self.modules
                .insert(module_name.to_string(), module.loaded_before.clone());
        }

        // Mark module as registered
        //
        // 将模块标记为已注册
        self.registered_modules.insert(module_name.to_string());

        info!(
            "Configuration of registered module '{}': {} animations, {} sprites",
            module_name,
            config.animations.len(),
            config.sprites.len()
        );
    }

    /// Get animation configuration by name
    ///
    /// 按名称获取动画配置
    pub fn get_animation(&self, name: &str) -> Option<&AnimationConfig> {
        self.sprite_animations.get(name)
    }

    /// Get sprite configuration by name
    ///
    /// 按名称获取精灵配置
    pub fn get_sprite(&self, name: &str) -> Option<&SpriteConfig> {
        self.sprites.get(name)
    }

    /// Get the dependency list of a module
    ///
    /// 获取模块的依赖列表
    #[allow(dead_code)]
    pub fn get_module_dependencies(&self, module_name: &str) -> Option<&Vec<String>> {
        self.modules.get(module_name)
    }

    /// List all available animation names
    ///
    /// 列出所有可用的动画名称
    #[allow(dead_code)]
    pub fn list_animations(&self) -> Vec<&String> {
        self.sprite_animations.keys().collect()
    }

    /// List all available sprite names
    ///
    /// 列出所有可用的精灵名称
    #[allow(dead_code)]
    pub fn list_sprites(&self) -> Vec<&String> {
        self.sprites.keys().collect()
    }
}
