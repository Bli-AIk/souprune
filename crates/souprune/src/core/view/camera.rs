//! # camera.rs
//!
//! Previously contained CameraAnchored systems that kept UI elements
//! fixed on screen by manually syncing their transform to the camera.
//!
//! REMOVED: Replaced by Bevy's native transform hierarchy.
//! Views that need camera-relative positioning now parent their root
//! entity to the camera entity, so child transforms are automatically
//! relative to the camera. No custom systems needed.
