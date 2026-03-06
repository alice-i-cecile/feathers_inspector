//! Central UI state for the inspector.
//!
//! This state is the "model" for the inspector UI,
//! tracking the currently selected object, active tabs, filters, and cached data.
//!
//! This information is then used to drive the UI rendering in the various panels.

use bevy::ecs::component::ComponentId;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use crate::extension_methods::WorldInspectionExtensionTrait;
use crate::inspection::component_inspection::{ComponentInspectionSettings, ComponentMetadataMap};
use crate::inspection::entity_inspection::{EntityInspection, EntityInspectionSettings};
use crate::memory_size::MemorySize;

/// Marker component for inspector-internal entities that should not appear in the entity list.
/// Applied to cameras, observers, and other internal entities.
#[derive(Component)]
pub struct InspectorInternal;

/// Central UI state for the inspector.
/// All UI-related state flows through this resource.
#[derive(Resource, Default)]
pub struct InspectorState {
    /// Whether the inspector is currently paused.
    ///
    /// When the inspector is paused,
    /// it does not automatically receive updates,
    /// allowing the user to inspect a snapshot of the world.
    pub is_paused: bool,
    /// Currently selected object for detail view.
    pub selected_object: Option<Entity>,
    /// Previously selected object (for change detection).
    pub previous_selected_object: Option<Entity>,
    /// Active tab in the object list panel.
    pub active_objects_tab: ObjectListTab,
    /// Active tab in the detail panel.
    pub active_detail_tab: DetailTab,
    /// Previous active tab (for change detection).
    pub previous_detail_tab: DetailTab,
    /// Current search/filter text for object list.
    pub filter_text: String,
    /// Component filter: only show entities with these components.
    pub mandatory_components: Vec<ComponentId>,
}

/// Active tab in the object list panel.
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ObjectListTab {
    #[default]
    Entities,
    Resources,
    Observers,
    OneShotSystems,
}

/// Active tab in the detail panel.
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum DetailTab {
    #[default]
    Components,
    Relationships,
}

/// Cached data for the inspector.
///
/// This is regularly invalidated, but this resource helps avoid repeated allocations.
#[derive(Resource, Default)]
pub struct InspectorCache {
    /// Cached object list after filtering.
    pub filtered_objects: Vec<ObjectListEntry>,
    /// Cached metadata map (reused across inspections).
    pub metadata_map: Option<ComponentMetadataMap>,
    /// Snapshot of the world state.
    pub snapshot: WorldSnapshot,
}

/// Collects and indexes [`EntityInspection`]s in an ordered way.
#[derive(Default)]
pub struct WorldSnapshot {
    /// Maps an [`Entity`] to its [`EntityInspection`].
    inspections: HashMap<Entity, EntityInspection>,
    /// The ordered snapshotted [`Entity`]s .
    entity_order: Vec<Entity>,
    /// Whether the cache contains a full snapshot of the filtered entities (used for paused state).
    pub is_full: bool,
}

impl WorldSnapshot {
    pub fn clear(&mut self) {
        self.inspections.clear();
        self.entity_order.clear();
        self.is_full = false;
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn full(inspections: Vec<EntityInspection>, ordering: Vec<Entity>) -> Self {
        let inspections = inspections
            .into_iter()
            .map(|inspection| (inspection.entity, inspection))
            .collect();

        Self {
            inspections,
            entity_order: ordering,
            is_full: true,
        }
    }

    pub fn single(world: &mut World, entity: Entity) -> Self {
        let metadata_map = world
            .resource_mut::<InspectorCache>()
            .metadata_map
            .take()
            .unwrap();

        let inspection = world.inspect_cached(
            entity,
            &EntityInspectionSettings {
                include_components: true,
                component_settings: ComponentInspectionSettings {
                    store_reflected_value: true,
                    ..default()
                },
            },
            &metadata_map,
        );

        let mut cache = world.resource_mut::<InspectorCache>();
        cache.metadata_map = Some(metadata_map);

        if let Ok(inspection) = inspection {
            let entity_order = vec![inspection.entity];
            Self {
                inspections: HashMap::from([(inspection.entity, inspection)]),
                entity_order,
                is_full: false,
            }
        } else {
            Self::empty()
        }
    }

    pub fn get(&self, entity: Entity) -> Option<&EntityInspection> {
        self.inspections.get(&entity)
    }

    pub fn iter(&self) -> impl Iterator<Item = &EntityInspection> {
        self.entity_order
            .iter()
            .filter_map(|e| self.inspections.get(e))
    }

    pub fn is_full(&self) -> bool {
        self.is_full
    }
}

/// Data for a single entity in the object list.
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

impl ObjectListEntry {
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn entity(&self) -> Entity {
        self.entity
    }
}
