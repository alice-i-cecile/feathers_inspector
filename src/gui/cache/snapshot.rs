use bevy::{platform::collections::HashMap, prelude::*};

use crate::{
    gui::cache::InspectorCache,
    inspection::entity_inspection::{EntityInspection, EntityInspectionSettings},
};

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

    fn create_test_inspection(entity: Entity) -> EntityInspection {
        EntityInspection {
            entity,
            name: None,
            total_memory_size: None,
            components: None,
            spawn_details: None,
        }
    }

    #[test]
    fn full_creation_populates_data() {
        let e1 = Entity::from_bits(1);
        let e2 = Entity::from_bits(2);
        let inspections = vec![create_test_inspection(e1), create_test_inspection(e2)];
        let ordering = vec![e1, e2];

        let snapshot = WorldSnapshot::full(inspections, ordering);

        assert!(snapshot.is_full());
        assert_eq!(snapshot.iter().count(), 2);
    }

    #[test]
    fn snapshot_preserves_insertion_order() {
        let e1 = Entity::from_bits(1);
        let e2 = Entity::from_bits(2);
        let i1 = create_test_inspection(e1);
        let i2 = create_test_inspection(e2);

        let snapshot = WorldSnapshot::full(vec![i1.clone(), i2.clone()], vec![e2, e1]);

        let order: Vec<Entity> = snapshot.iter().map(|i| i.entity).collect();
        assert_eq!(order, vec![e2, e1]);
    }

    #[test]
    fn snapshot_provides_lookup_access() {
        let e1 = Entity::from_bits(1);
        let e2 = Entity::from_bits(2);
        let i1 = create_test_inspection(e1);

        let snapshot = WorldSnapshot::full(vec![i1], vec![e1]);

        assert!(snapshot.get(e1).is_some());
        assert!(snapshot.get(e2).is_none());
    }

    #[test]
    fn clear_resets_all_state() {
        let e1 = Entity::from_bits(1);
        let i1 = create_test_inspection(e1);
        let mut snapshot = WorldSnapshot::full(vec![i1], vec![e1]);

        snapshot.clear();

        assert!(!snapshot.is_full());
        assert_eq!(snapshot.iter().count(), 0);
        assert!(snapshot.get(e1).is_none());
    }
}
