//! Grouping and sorting entities based on their components.

use bevy::prelude::*;

use crate::{archetype_similarity_grouping, hierarchy_grouping};

/// A hierarchical grouping of entities based on their components.
///
/// This can be used to organize entities into categories and sub-categories,
/// or flattened into a single sorted list to facilitate inspection and debugging.
///
/// As discussed in [`EntityGrouping::generate`], this grouping is based on the components
/// that entities share in common.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
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
    pub fn generate(
        world: &World,
        entities: impl ExactSizeIterator<Item = Entity>,
        strategy: GroupingStrategy,
    ) -> Self {
        match strategy {
            GroupingStrategy::Hierarchy => hierarchy_grouping::group(world, entities),
            GroupingStrategy::ArchetypeSimilarity => {
                archetype_similarity_grouping::group(world, entities)
            }
        }
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

/// Specifies what kind of grouping [`EntityGrouping::generate`] should make.
#[derive(Debug, Clone, Copy, Default)]
pub enum GroupingStrategy {
    /// Group based on parent-child relationships.
    #[default]
    Hierarchy,
    /// Group based how archetypes differ on which components represent them.
    ArchetypeSimilarity,
}
