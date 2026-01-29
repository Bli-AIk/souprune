pub mod parsing;
pub mod reload;
pub mod resources;
pub mod setup;
pub mod spawn;
pub mod update;

// Re-export common types and systems
// New names
pub use resources::{
    RonDrivenView, ViewGenerated, ViewGlobalTriggerConfig, ViewLayoutHandle, ViewLayoutWatcher,
};
// Backwards compatibility aliases
pub use resources::{
    RonDrivenUI, UIGenerated, UIGlobalTriggerConfig, UILayoutHandle, UILayoutWatcher,
};

pub use parsing::*;
pub use reload::*;
pub use setup::*;
pub use spawn::*;
pub use update::*;
