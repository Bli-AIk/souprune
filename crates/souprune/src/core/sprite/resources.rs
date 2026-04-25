//! # resources.rs
//!
//! # resources.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module defines resources for sprite management.
//!
//! 该模块定义了用于精灵管理的资源。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It includes the `ModuleSpriteRegistry` which caches texture atlases and handles.
//!
//! 它包含了 `ModuleSpriteRegistry`，用于缓存纹理图集和句柄。

use bevy::asset::LoadedFolder;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

pub type AtlasCacheData = (
    Handle<TextureAtlasLayout>,
    Handle<Image>,
    HashMap<String, usize>,
);
pub type ModuleAtlasCache = HashMap<String, AtlasCacheData>;

/// Registry for managing sprite modules and their loaded assets.
///
/// 用于管理精灵模块及其已加载资源的注册表。
#[derive(Resource, Default)]
pub(crate) struct ModuleSpriteRegistry {
    pub(crate) modules: HashMap<String, Handle<LoadedFolder>>,
    pub(crate) atlas_cache: ModuleAtlasCache,
    pub(crate) missing_texture: Option<Handle<Image>>,
}

impl ModuleSpriteRegistry {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            atlas_cache: HashMap::new(),
            missing_texture: None,
        }
    }

    pub fn register_module(&mut self, module_name: String, folder_handle: Handle<LoadedFolder>) {
        self.modules.insert(module_name, folder_handle);
    }

    pub fn get_module(&self, module_name: &str) -> Option<&Handle<LoadedFolder>> {
        self.modules.get(module_name)
    }

    pub fn get_cached_atlas(&self, module_name: &str) -> Option<&AtlasCacheData> {
        self.atlas_cache.get(module_name)
    }

    pub fn cache_atlas(
        &mut self,
        module_name: String,
        atlas_layout: Handle<TextureAtlasLayout>,
        texture: Handle<Image>,
        index_map: HashMap<String, usize>,
    ) {
        self.atlas_cache
            .insert(module_name, (atlas_layout, texture, index_map));
    }
}
