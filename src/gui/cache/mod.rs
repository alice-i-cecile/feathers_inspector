//! Cache management for the inspector UI.

use crate::inspection::component_inspection::ComponentMetadataMap;
use bevy::prelude::*;

pub mod snapshot;
pub(crate) mod systems;

pub use snapshot::WorldSnapshot;
pub use systems::update_inspector_cache;

/// Cached data for the inspector.
///
/// This is regularly invalidated, but this resource helps avoid repeated allocations.
#[derive(Resource, Default)]
pub struct InspectorCache {
    /// Cached object list after filtering.
    pub filtered_objects: Vec<crate::gui::state::ObjectListEntry>,
    /// Cached metadata map (reused across inspections).
    pub metadata_map: Option<ComponentMetadataMap>,
    /// Snapshot of the world state.
    pub snapshot: WorldSnapshot,
    /// Signals to force-refresh the cache.
    pub is_dirty: bool,
}
