use bevy::asset::LoadedFolder;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

pub type AtlasCacheData = (
    Handle<TextureAtlasLayout>,
    Handle<Image>,
    HashMap<String, usize>,
);
pub type ModuleAtlasCache = HashMap<String, AtlasCacheData>;

#[derive(Resource, Default)]
pub(crate) struct ModuleSpriteRegistry {
    pub(crate) modules: HashMap<String, Handle<LoadedFolder>>,
    pub(crate) atlas_cache: ModuleAtlasCache,
}

impl ModuleSpriteRegistry {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            atlas_cache: HashMap::new(),
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
