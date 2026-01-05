//! Asset loader for Alight Motion project files.

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext, RenderAssetUsages};
use bevy::prelude::*;
use std::collections::HashMap;
use std::io::{Cursor, Read};

use crate::error::AmError;
use crate::schema::AmScene;

/// Asset representing a loaded Alight Motion project.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct AmProject {
    /// The parsed scene data.
    pub scene: AmScene,
    /// Mapping from amproj URIs to image handles.
    pub images: HashMap<String, Handle<Image>>,
    /// Raw image data for embedded images (before loading).
    pub embedded_images: HashMap<String, Vec<u8>>,
}

/// Loader for .amproj and .xml AM files.
#[derive(Default)]
pub struct AlightMotionLoader;

impl AssetLoader for AlightMotionLoader {
    type Asset = AmProject;
    type Settings = ();
    type Error = AmError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let path = load_context.path();
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        match extension.to_lowercase().as_str() {
            "amproj" => load_amproj(&bytes, load_context).await,
            "xml" => load_xml(&bytes, load_context).await,
            _ => Err(AmError::InvalidFormat(format!(
                "Unknown file extension: {}",
                extension
            ))),
        }
    }

    fn extensions(&self) -> &[&str] {
        &["amproj", "xml"]
    }
}

/// Load from .amproj ZIP archive.
async fn load_amproj(
    bytes: &[u8],
    load_context: &mut LoadContext<'_>,
) -> Result<AmProject, AmError> {
    let cursor = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor)?;

    // Find the XML file in the archive
    let mut xml_content = None;
    let mut embedded_images = HashMap::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();

        if name.ends_with(".xml") {
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            xml_content = Some(content);
        } else if name.ends_with(".png")
            || name.ends_with(".jpg")
            || name.ends_with(".jpeg")
            || name.ends_with(".webp")
        {
            let mut data = Vec::new();
            file.read_to_end(&mut data)?;
            // Store with amproj: prefix for lookup
            let uri = format!("amproj:{}", name);
            embedded_images.insert(uri, data);
        }
    }

    let xml_content = xml_content
        .ok_or_else(|| AmError::InvalidFormat("No XML file found in amproj archive".to_string()))?;

    // Parse the XML
    let scene: AmScene = quick_xml::de::from_str(&xml_content)?;

    // Load embedded images as labeled assets
    let mut images = HashMap::new();
    for (uri, data) in &embedded_images {
        // Try to load the image from raw bytes
        if let Ok(image) = Image::from_buffer(
            data,
            bevy::image::ImageType::Extension("png"),
            bevy::image::CompressedImageFormats::NONE,
            true,
            bevy::image::ImageSampler::Default,
            RenderAssetUsages::all(),
        ) {
            let label = uri.trim_start_matches("amproj:");
            let handle = load_context.add_labeled_asset(label.to_string(), image);
            images.insert(uri.clone(), handle);
        }
    }

    Ok(AmProject {
        scene,
        images,
        embedded_images,
    })
}

/// Load from standalone .xml file.
async fn load_xml(bytes: &[u8], _load_context: &mut LoadContext<'_>) -> Result<AmProject, AmError> {
    let content = String::from_utf8_lossy(bytes);
    let scene: AmScene = quick_xml::de::from_str(&content)?;

    Ok(AmProject {
        scene,
        images: HashMap::new(),
        embedded_images: HashMap::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xml_parsing_from_string() {
        let xml = r##"<?xml version='1.0' encoding='UTF-8' ?>
        <scene title="Test" width="1280" height="960" fps="60" totalTime="2000" bgcolor="#ff000000">
            <shape id="123" label="Test Shape" startTime="0" endTime="1000" fillType="color" s=".rect">
                <transform>
                    <location value="640.0,480.0,0.0" />
                </transform>
                <property name="size" type="vec2" value="100.0,100.0" />
            </shape>
        </scene>
        "##;

        let scene: AmScene = quick_xml::de::from_str(xml).expect("Failed to parse XML");
        assert_eq!(scene.title, "Test");
        assert_eq!(scene.width, 1280);
        assert_eq!(scene.height, 960);
        assert_eq!(scene.fps, 60);
    }
}
