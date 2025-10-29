use bevy::asset::LoadedFolder;
use bevy::image::ImageSampler;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub(crate) struct ModuleSpriteRegistry {
    pub(crate) modules: HashMap<String, Handle<LoadedFolder>>,
}

impl ModuleSpriteRegistry {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    pub fn register_module(&mut self, module_name: String, folder_handle: Handle<LoadedFolder>) {
        self.modules.insert(module_name, folder_handle);
    }

    pub fn get_module(&self, module_name: &str) -> Option<&Handle<LoadedFolder>> {
        self.modules.get(module_name)
    }
}
pub fn create_texture_atlas(
    folder: &LoadedFolder,
    padding: Option<UVec2>,
    sampling: Option<ImageSampler>,
    textures: &mut ResMut<Assets<Image>>,
) -> (
    TextureAtlasLayout,
    TextureAtlasSources,
    Handle<Image>,
    HashMap<String, usize>,
) {
    let mut texture_atlas_builder = TextureAtlasBuilder::default();
    texture_atlas_builder.padding(padding.unwrap_or_default());

    let mut index_map = HashMap::new();
    let mut added_count = 0;

    for handle in folder.handles.iter() {
        if let Some(path) = handle.path() {
            let path_str = path.to_string();
            if !path_str.ends_with(".png")
                && !path_str.ends_with(".jpg")
                && !path_str.ends_with(".jpeg")
            {
                continue;
            }
            index_map.insert(path_str, added_count);
            added_count += 1;
        }

        let id = handle.id().typed_unchecked::<Image>();
        let Some(texture) = textures.get(id) else {
            warn!(
                "{} did not resolve to an `Image` asset.",
                handle.path().unwrap()
            );
            continue;
        };

        texture_atlas_builder.add_texture(Some(id), texture);
    }

    let (texture_atlas_layout, texture_atlas_sources, texture) =
        texture_atlas_builder.build().unwrap();
    let texture = textures.add(texture);

    let image = textures.get_mut(&texture).unwrap();
    image.sampler = sampling.unwrap_or_default();

    (
        texture_atlas_layout,
        texture_atlas_sources,
        texture,
        index_map,
    )
}

pub fn create_sprite_from_atlas(
    commands: &mut Commands,
    translation: (f32, f32, f32),
    atlas_texture: Handle<Image>,
    atlas_sources: TextureAtlasSources,
    atlas_handle: Handle<TextureAtlasLayout>,
    vendor_handle: &Handle<Image>,
) {
    commands.spawn((
        Transform {
            translation: Vec3::new(translation.0, translation.1, translation.2),
            ..default()
        },
        Sprite::from_atlas_image(
            atlas_texture,
            atlas_sources.handle(atlas_handle, vendor_handle).unwrap(),
        ),
    ));
}
