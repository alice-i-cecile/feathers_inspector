//! Central UI state for the inspector.
//!
//! This state is the "model" for the inspector UI,
//! tracking the currently selected entity, active tabs, filters, and cached data.
//!
//! This information is then used to drive the UI rendering in the various panels.

use bevy::ecs::component::ComponentId;
use bevy::prelude::*;

use crate::component_inspection::ComponentMetadataMap;
use crate::memory_size::MemorySize;

/// Marker component for inspector-internal entities that should not appear in the entity list.
/// Applied to cameras, observers, and other internal entities.
#[derive(Component)]
pub struct InspectorInternal;

/// Central UI state for the inspector.
/// All UI-related state flows through this resource.
#[derive(Resource, Default)]
pub struct InspectorState {
    /// Currently selected object for detail view.
    pub selected_object: Option<Entity>,
    /// Active tab in the object list panel.
    pub active_list_tab: ObjectListTab,
    /// Active tab in the detail panel.
    pub active_detail_tab: DetailTab,
    /// Current search/filter text for entity list.
    pub filter_text: String,
    /// Component filter: only show entities with these components.
    pub mandatory_components: Vec<ComponentId>,
    /// Previously selected entity (for change detection).
    pub previous_selection: Option<Entity>,
    /// Previous active tab (for change detection).
    pub previous_tab: DetailTab,
}

/// Active tab in the object list panel.
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ObjectListTab {
    #[default]
    Entities,
    Resources,
}

/// Active tab in the detail panel.
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum DetailTab {
    #[default]
    Components,
    Relationships,
}

/// Cached data for the inspector to avoid recomputation.
#[derive(Resource, Default)]
pub struct InspectorCache {
    /// Cached entity list after filtering.
    pub filtered_entities: Vec<ObjectListEntry>,
    /// Cached metadata map (reused across inspections).
    pub metadata_map: Option<ComponentMetadataMap>,
    /// Whether the cache needs to be refreshed.
    pub stale: bool,
}

/// Data for a single object in the object list.
#[derive(Clone, Debug)]
pub struct ObjectListEntry {
    /// The entity.
    pub entity: Entity,
    /// Display name for the entity.
    pub display_name: String,
    /// Number of components on this entity.
    pub component_count: usize,
    /// Total memory size of all components.
    pub memory_size: MemorySize,
}

/// Tracks the state of the inspector window.
#[derive(Resource, Default)]
pub struct InspectorWindowState {
    /// Entity ID of the inspector window, if it exists.
    pub window_entity: Option<Entity>,
    /// Entity ID of the camera rendering to the inspector window.
    pub camera_entity: Option<Entity>,
    /// Whether the inspector window is currently open.
    pub is_open: bool,
}
