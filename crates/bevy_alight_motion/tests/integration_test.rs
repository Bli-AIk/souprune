use std::io::{Cursor, Read};

#[test]
fn test_parse_real_amproj() {
    let amproj_path = "/home/aik/Downloads/am/新项目 24 20260105_182938.amproj";

    // Skip if file doesn't exist
    if !std::path::Path::new(amproj_path).exists() {
        eprintln!("Skipping test: amproj file not found");
        return;
    }

    let bytes = std::fs::read(amproj_path).expect("Failed to read amproj");
    let cursor = Cursor::new(&bytes);
    let mut archive = zip::ZipArchive::new(cursor).expect("Failed to open ZIP");

    let mut xml_found = false;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let name = file.name().to_string();

        if name.ends_with(".xml") {
            xml_found = true;
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();

            let scene: bevy_alight_motion::schema::AmScene =
                quick_xml::de::from_str(&content).expect("Failed to parse XML");

            assert_eq!(scene.title, "新项目 24");
            assert_eq!(scene.width, 1280);
            assert_eq!(scene.height, 960);
            assert_eq!(scene.fps, 60);
            assert!(!scene.media.is_empty(), "Should have media");
            assert!(!scene.layers.is_empty(), "Should have layers");

            // Check media URIs
            for media in &scene.media {
                assert!(
                    media.uri.starts_with("amproj:"),
                    "Media URI should start with amproj:"
                );
            }
        }
    }

    assert!(xml_found, "Should find XML in amproj");
}
