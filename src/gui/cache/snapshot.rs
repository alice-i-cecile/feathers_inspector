use bevy::{platform::collections::HashMap, prelude::*};

use crate::inspection::{
    component_inspection::ComponentMetadataMap,
    entity_inspection::{EntityInspection, EntityInspectionSettings},
};

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

        use crate::extension_methods::WorldInspectionExtensionTrait;
        let inspection = world.inspect_cached(
            entity,
            &EntityInspectionSettings {
                include_components: true,
                component_settings:
                    crate::inspection::component_inspection::ComponentInspectionSettings {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_empty() {
        let mut snapshot = WorldSnapshot::empty();
        assert!(!snapshot.is_full());
        assert_eq!(snapshot.iter().count(), 0);
        snapshot.clear();
        assert!(!snapshot.is_full());
    }
}
