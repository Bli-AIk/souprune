pub mod parsing;
pub mod player_data;
pub mod reload;
pub mod resources;
pub mod setup;
pub mod spawn;
pub mod update;

// Re-export common types and systems
pub use resources::{
    HotReloadableViewRoot, PendingViewReloads, RonDrivenView, ViewGlobalTriggerConfig,
    ViewLayoutHandle,
};

// Re-export RepeatContext for use in expression evaluation

pub use reload::update_view_from_map_system;
#[cfg(feature = "debug")]
pub use reload::{incremental_reload_system, watch_view_layout_changes_system};
pub use setup::*;
pub use spawn::*;
pub use update::*;
