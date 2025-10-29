use crate::extra::toml::{AnimationConfig, SpriteConfig};
use bevy::log::info;
use bevy::platform::collections::HashMap;
use bevy::prelude::Resource;

/// A resource that stores TOML configuration data for all modules
#[derive(Resource, Default)]
pub struct TomlConfigRegistry {
    pub animations: HashMap<String, AnimationConfig>,
    pub sprites: HashMap<String, SpriteConfig>,
    pub modules: HashMap<String, Vec<String>>,
}

impl TomlConfigRegistry {
    pub fn register_module_config(
        &mut self,
        module_name: &str,
        config: &crate::extra::toml::TomlConfig,
    ) {
        for anim in &config.animations {
            self.animations.insert(anim.name.clone(), anim.clone());
        }

        for sprite in &config.sprites {
            self.sprites.insert(sprite.name.clone(), sprite.clone());
        }

        for module in &config.module {
            self.modules
                .insert(module_name.to_string(), module.loaded_before.clone());
        }

        info!(
            "Configuration of registered module '{}': {} animations, {} sprites",
            module_name,
            config.animations.len(),
            config.sprites.len()
        );
    }

    /// Get animation configuration by name
    pub fn get_animation(&self, name: &str) -> Option<&AnimationConfig> {
        self.animations.get(name)
    }

    /// Get sprite configuration by name
    pub fn get_sprite(&self, name: &str) -> Option<&SpriteConfig> {
        self.sprites.get(name)
    }

    /// Get the dependency list of a module
    pub fn get_module_dependencies(&self, module_name: &str) -> Option<&Vec<String>> {
        self.modules.get(module_name)
    }

    /// List all available animation names
    pub fn list_animations(&self) -> Vec<&String> {
        self.animations.keys().collect()
    }

    /// List all available sprite names
    pub fn list_sprites(&self) -> Vec<&String> {
        self.sprites.keys().collect()
    }
}
