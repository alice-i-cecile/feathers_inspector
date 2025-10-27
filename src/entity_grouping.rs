//! Grouping and sorting entities based on their components.

use bevy::{
    ecs::{
        archetype::{Archetype, ArchetypeId},
        component::ComponentId,
    },
    platform::collections::HashMap,
    prelude::*,
};

/// A hierarchical grouping of entities based on their components.
///
/// This can be used to organize entities into categories and sub-categories,
/// or flattened into a single sorted list to facilitate inspection and debugging.
///
/// As discussed in [`EntityGrouping::generate`], this grouping is based on the components
/// that entities share in common.
#[derive(Debug, Default, Clone)]
pub struct EntityGrouping {
    /// The entities that belong to this group.
    pub entities: Vec<Entity>,
    /// Sub-groups within this group.
    pub sub_groups: Vec<EntityGrouping>,
}

impl EntityGrouping {
    /// Creates a new, empty `EntityGrouping`.
    pub const fn new() -> Self {
        Self {
            entities: Vec::new(),
            sub_groups: Vec::new(),
        }
    }

    /// Generates an [`EntityGrouping`] based on the components of the provided entities.
    pub fn generate(world: &World, entities: impl ExactSizeIterator<Item = Entity>) -> Self {
        let mut archetype_query = world.try_query::<&Archetype>().unwrap();

        let mut represented_archetypes: HashMap<ArchetypeId, Vec<Entity>> = HashMap::default();
        for entity in entities {
            let archetype = archetype_query.get(world, entity).unwrap();
            let archetype_id = archetype.id();

            represented_archetypes
                .entry(archetype_id)
                .or_default()
                .push(entity);
        }

        let archetypes = world.archetypes();
        let mut components_in_archetype: HashMap<ArchetypeId, Vec<ComponentId>> =
            HashMap::default();
        for archetype_id in represented_archetypes.keys() {
            let archetype = archetypes.get(*archetype_id).unwrap();
            components_in_archetype.insert(*archetype_id, archetype.components().to_vec());
        }

        todo!("Implement hierarchical clustering");
    }

    /// Flattens the grouping into a single list of entities.
    ///
    /// This flattened list will represent one possible "good" ordering of the entities,
    /// where entities in the same group are kept together, and sub-groups are expanded in order.
    pub fn flatten(&self) -> Vec<Entity> {
        let mut all_entities = self.entities.clone();
        for sub_group in &self.sub_groups {
            all_entities.extend(sub_group.flatten());
        }
        all_entities
    }
}
