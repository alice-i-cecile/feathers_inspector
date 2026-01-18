//! Grouping entities by parent-child relationship.

use bevy::{ecs::entity::EntityIndex, platform::collections::HashSet, prelude::*};

use crate::entity_grouping::EntityGrouping;

/// Groups entities by their parent-child hierarchy.
///
/// Returns an [`EntityGrouping`] tree,
/// Where each grouping contains one entity,
/// and the [`Children`] are stored as [`sub_groups`],
/// one for each child.
/// Children that are not included in the provided `entities` are not added.
///
/// The only exception is the root entity grouping,
/// where [`entities`] is empty,
/// and each element of [`sub_groups`] represents the root entities,
/// (entities either with no [`ChildOf`] component,
/// or whose parent isn't among the provided `entities`).
///
/// Cycles or malformed hierarchies are guarded against;
/// entities involved in cycles may be omitted if no acyclic root exists.
///
/// [`entities`]: EntityGrouping::entities
/// [`sub_groups`]: EntityGrouping::sub_groups
pub(crate) fn group(world: &World, entities: impl IntoIterator<Item = Entity>) -> EntityGrouping {
    // `HashSet` for deduplication.
    let entities: HashSet<Entity> = entities.into_iter().collect();
    if entities.is_empty() {
        return EntityGrouping::new();
    }
    let mut root_entities = collect_root_entities(world, &entities);
    sort_entities(world, &mut root_entities);
    let sub_groups = generate_forest(world, &entities, &root_entities);

    EntityGrouping {
        entities: Vec::new(),
        sub_groups,
    }
}

/// Returns a collection of entities
/// that either have no [`ChildOf`] component,
/// or whose [`parent`] isn't in `entities`.
///
/// [`parent`]: ChildOf::parent
fn collect_root_entities(world: &World, entities: &HashSet<Entity>) -> Vec<Entity> {
    entities
        .iter()
        .copied()
        .filter(|&entity| world.get_entity(entity).is_ok())
        .filter(|&entity| {
            let has_parent_in_set = world
                .get::<ChildOf>(entity)
                .map(|child_of| entities.contains(&child_of.parent()))
                .unwrap_or(false);
            !has_parent_in_set
        })
        .collect()
}

/// Generates a forest of entities,
/// where each tree is a root entity with its descendants.
fn generate_forest(
    world: &World,
    entities: &HashSet<Entity>,
    root_entities: &[Entity],
) -> Vec<EntityGrouping> {
    let mut visited: HashSet<Entity> = HashSet::default();
    root_entities
        .iter()
        .filter_map(|root| generate_grouping_tree(world, *root, entities, &mut visited))
        .collect()
}

/// Returns an entity tree as an [`EntityGrouping`].
///
/// The grouping's [`entities`] only contains the provided `entity`,
/// and its [`Children`] are stored as [`sub_groups`],
/// one for each child.
///
/// Descendants of `entity` that are not in `entities` are not included.
///
/// [`entities`]: EntityGrouping::entities
/// [`sub_groups`]: EntityGrouping::sub_groups
fn generate_grouping_tree(
    world: &World,
    entity: Entity,
    entities: &HashSet<Entity>,
    visited: &mut HashSet<Entity>,
) -> Option<EntityGrouping> {
    if world.get_entity(entity).is_err() {
        return None;
    }
    if !visited.insert(entity) {
        return None;
    }
    let mut tree = EntityGrouping {
        entities: vec![entity],
        sub_groups: Vec::new(),
    };

    if let Some(children) = world.get::<Children>(entity) {
        let mut included_children: Vec<Entity> = children
            .iter()
            .filter(|child| entities.contains(child))
            .collect();
        sort_entities(world, &mut included_children);
        tree.sub_groups = included_children
            .into_iter()
            .filter_map(|child| generate_grouping_tree(world, child, entities, visited))
            .collect();
    }
    Some(tree)
}

/// Sorts entities using [`sorting_key`].
fn sort_entities(world: &World, entities: &mut [Entity]) {
    entities.sort_by_cached_key(|&entity| sorting_key(world, entity));
}

/// Generates a sorting key for entities.
///
/// Three criteria are used in order,
/// with the next one being used if the comparison before has equal result:
///
/// 1. Entities with [`Name`] component are ordered before unnamed entities.
/// 2. Named entities are ordered alphabetically (case-insensitive).
/// 3. If [`Name`]s are equal, entities are ordered by [`Entity::index`].
fn sorting_key(world: &World, entity: Entity) -> (bool, String, EntityIndex) {
    match world.get::<Name>(entity) {
        Some(name) => (false, name.as_str().to_lowercase(), entity.index()),
        None => (true, String::new(), entity.index()),
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn hierarchy_preservation() {
        let mut world = World::new();
        let a = world.spawn_empty().id();
        let b = world.spawn_empty().set_parent_in_place(a).id();
        let c = world.spawn_empty().set_parent_in_place(a).id();
        let d = world.spawn_empty().id();
        let e = world.spawn_empty().set_parent_in_place(d).id();
        let f = world.spawn_empty().set_parent_in_place(e).id();
        let g = world.spawn_empty().id();

        let grouping = group(&world, vec![a, b, c, d, e, f, g]);
        let expected_grouping = EntityGrouping {
            entities: Vec::new(),
            sub_groups: vec![
                EntityGrouping {
                    entities: vec![a],
                    sub_groups: vec![
                        EntityGrouping {
                            entities: vec![b],
                            sub_groups: Vec::new(),
                        },
                        EntityGrouping {
                            entities: vec![c],
                            sub_groups: Vec::new(),
                        },
                    ],
                },
                EntityGrouping {
                    entities: vec![d],
                    sub_groups: vec![EntityGrouping {
                        entities: vec![e],
                        sub_groups: vec![EntityGrouping {
                            entities: vec![f],
                            sub_groups: Vec::new(),
                        }],
                    }],
                },
                EntityGrouping {
                    entities: vec![g],
                    sub_groups: Vec::new(),
                },
            ],
        };
        assert_eq!(grouping, expected_grouping);
    }

    #[test]
    fn named_vs_unnamed_sorting() {
        let mut world = World::new();
        // Root entities
        let unnamed = world.spawn_empty().id();
        let parent = world.spawn_empty().id();
        let beta = world.spawn(Name::new("Beta")).id();
        let alpha = world.spawn(Name::new("alpha")).id();
        // Children under `parent`
        let child_unnamed = world.spawn_empty().set_parent_in_place(parent).id();
        let child_named = world
            .spawn(Name::new("Child"))
            .set_parent_in_place(parent)
            .id();

        let grouping = group(
            &world,
            vec![unnamed, parent, beta, alpha, child_unnamed, child_named],
        );
        let expected_grouping = EntityGrouping {
            entities: Vec::new(),
            sub_groups: vec![
                EntityGrouping {
                    entities: vec![alpha],
                    sub_groups: Vec::new(),
                },
                EntityGrouping {
                    entities: vec![beta],
                    sub_groups: Vec::new(),
                },
                EntityGrouping {
                    entities: vec![unnamed],
                    sub_groups: Vec::new(),
                },
                EntityGrouping {
                    entities: vec![parent],
                    sub_groups: vec![
                        EntityGrouping {
                            entities: vec![child_named],
                            sub_groups: Vec::new(),
                        },
                        EntityGrouping {
                            entities: vec![child_unnamed],
                            sub_groups: Vec::new(),
                        },
                    ],
                },
            ],
        };
        assert_eq!(grouping, expected_grouping);
    }

    #[test]
    fn unicode_sorting_case_insensitive() {
        let mut world = World::new();
        let lower_a = world.spawn(Name::new("a")).id();
        let upper_a_umlaut = world.spawn(Name::new("Ä")).id();
        let grouping = group(&world, vec![upper_a_umlaut, lower_a]);

        let expected_grouping = EntityGrouping {
            entities: Vec::new(),
            sub_groups: vec![
                EntityGrouping {
                    entities: vec![lower_a],
                    sub_groups: Vec::new(),
                },
                EntityGrouping {
                    entities: vec![upper_a_umlaut],
                    sub_groups: Vec::new(),
                },
            ],
        };
        assert_eq!(grouping, expected_grouping);
    }

    #[test]
    fn sort_by_index() {
        let mut world = World::new();
        let first = world.spawn(Name::new("same")).id();
        let second = world.spawn(Name::new("same")).id();

        let grouping = group(&world, vec![second, first]);
        let expected_grouping = EntityGrouping {
            entities: Vec::new(),
            sub_groups: vec![
                EntityGrouping {
                    entities: vec![first],
                    sub_groups: Vec::new(),
                },
                EntityGrouping {
                    entities: vec![second],
                    sub_groups: Vec::new(),
                },
            ],
        };
        assert_eq!(grouping, expected_grouping);
    }

    #[test]
    fn child_without_parent_in_set_becomes_root() {
        let mut world = World::new();
        let parent = world.spawn_empty().id();
        let child = world.spawn_empty().set_parent_in_place(parent).id();

        let grouping = group(&world, vec![child]);
        let expected_grouping = EntityGrouping {
            entities: Vec::new(),
            sub_groups: vec![EntityGrouping {
                entities: vec![child],
                sub_groups: Vec::new(),
            }],
        };
        assert_eq!(grouping, expected_grouping);
    }

    #[test]
    fn skip_non_existent_entities() {
        let mut world = World::new();
        let alive = world.spawn_empty().id();
        let dead = world.spawn_empty().id();
        let _ = world.despawn(dead);

        let grouping = group(&world, vec![alive, dead]);
        let expected_grouping = EntityGrouping {
            entities: Vec::new(),
            sub_groups: vec![EntityGrouping {
                entities: vec![alive],
                sub_groups: Vec::new(),
            }],
        };
        assert_eq!(grouping, expected_grouping);
    }

    #[test]
    fn deduplication() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let grouping = group(&world, vec![entity, entity, entity]);

        let expected_grouping = EntityGrouping {
            entities: Vec::new(),
            sub_groups: vec![EntityGrouping {
                entities: vec![entity],
                sub_groups: Vec::new(),
            }],
        };
        assert_eq!(grouping, expected_grouping);
    }
}
