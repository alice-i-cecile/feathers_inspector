//! Cache management for the inspector UI.

pub mod snapshot;
pub(crate) mod systems;

pub use snapshot::{InspectorCache, WorldSnapshot};
pub use systems::update_inspector_cache;
