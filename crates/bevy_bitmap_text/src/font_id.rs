use bevy::prelude::*;

/// Identifies a loaded font by name.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Reflect)]
pub struct FontId(pub String);

impl FontId {
    pub fn from_name(name: &str) -> Self {
        Self(name.to_string())
    }
}

impl<S: Into<String>> From<S> for FontId {
    fn from(s: S) -> Self {
        Self(s.into())
    }
}
