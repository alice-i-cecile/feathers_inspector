//! Clustering entities by archetype similarity.

use bevy::{
    ecs::{archetype::ArchetypeId, component::ComponentId},
    platform::collections::{HashMap, HashSet},
    prelude::*,
};

use crate::entity_grouping::EntityGrouping;

pub(crate) fn group(world: &World, entities: impl IntoIterator<Item = Entity>) -> EntityGrouping {
    let entities_by_archetype = get_entities_by_archetype(world, entities);
    if entities_by_archetype.is_empty() {
        return EntityGrouping::new();
    }
    if entities_by_archetype.len() == 1 {
        let mut grouping = EntityGrouping::new();
        let (_, mut entities) = entities_by_archetype
            .into_iter()
            .next()
            .expect("`entities_by_archetype.len() == 1`");
        entities.sort_by_key(|e| e.index());
        grouping.entities = entities;
        return grouping;
    }
    let components_by_archetype = get_components_by_archetype(world, &entities_by_archetype);
    cluster_archetypes(entities_by_archetype, components_by_archetype)
}

/// Associates archetypes to the entities belonging to them.
fn get_entities_by_archetype(
    world: &World,
    entities: impl IntoIterator<Item = Entity>,
) -> HashMap<ArchetypeId, Vec<Entity>> {
    let mut entities_by_archetype: HashMap<ArchetypeId, Vec<Entity>> = HashMap::default();
    for entity in entities {
        let Ok(entity_ref) = world.get_entity(entity) else {
            continue;
        };
        let archetype_id = entity_ref.archetype().id();
        entities_by_archetype
            .entry(archetype_id)
            .or_default()
            .push(entity);
    }
    entities_by_archetype
}

/// Associates components to the entities belonging to them.
fn get_components_by_archetype(
    world: &World,
    entities_by_archetype: &HashMap<ArchetypeId, Vec<Entity>>,
) -> HashMap<ArchetypeId, HashSet<ComponentId>> {
    let archetypes = world.archetypes();
    let mut archetype_ids: Vec<ArchetypeId> = entities_by_archetype.keys().cloned().collect();
    archetype_ids.sort_by_key(|archetype_id| archetype_id.index());
    let mut components_by_archetype = HashMap::default();
    for archetype_id in &archetype_ids {
        let component_set: HashSet<ComponentId> = archetypes
            .get(*archetype_id)
            .map_or_else(HashSet::default, |archetype| {
                archetype.components().iter().copied().collect()
            });
        components_by_archetype.insert(*archetype_id, component_set);
    }
    components_by_archetype
}

/// An intermediate object that helps agglomerative clustering.
#[derive(Clone)]
struct Cluster {
    /// Intersection of all archetypes inside. Functions as cache.
    signature: HashSet<ComponentId>,
    /// A transient grouping sub-tree.
    group: EntityGrouping,
}

/// Holds values for cluster distance evaluation and merging.
#[derive(Clone, Copy)]
struct ClusterPairMetadata {
    /// The `Cluster` with the lower vector index.
    low: usize,
    /// The `Cluster` with the higher vector index.
    high: usize,
    /// The distance between the two `Cluster`s.
    distance: f32,
}

/// Generates an [`EntityGrouping`] via agglomerative clustering.
fn cluster_archetypes(
    entities_by_archetype: HashMap<ArchetypeId, Vec<Entity>>,
    components_by_archetype: HashMap<ArchetypeId, HashSet<ComponentId>>,
) -> EntityGrouping {
    let mut clusters = seed_clusters(entities_by_archetype, components_by_archetype);
    while clusters.len() > 1 {
        clustering_pass(&mut clusters);
    }
    clusters.pop().expect("`clusters.len() == 1`").group
}

/// Creates one [`Cluster`] per archetype.
fn seed_clusters(
    mut entities_by_archetype: HashMap<ArchetypeId, Vec<Entity>>,
    components_by_archetype: HashMap<ArchetypeId, HashSet<ComponentId>>,
) -> Vec<Cluster> {
    let mut archetype_ids: Vec<ArchetypeId> = components_by_archetype.keys().cloned().collect();
    archetype_ids.sort_by_key(|archetype_id| archetype_id.index());
    let mut clusters: Vec<Cluster> = Vec::with_capacity(archetype_ids.len());
    for archetype_id in &archetype_ids {
        let mut entities = entities_by_archetype
            .remove(archetype_id)
            .unwrap_or_default();
        entities.sort_by_key(|entity| entity.index());
        let signature = components_by_archetype
            .get(archetype_id)
            .cloned()
            .unwrap_or_default();
        clusters.push(Cluster {
            signature,
            group: EntityGrouping {
                entities,
                sub_groups: Vec::new(),
            },
        });
    }
    clusters
}

/// Finds and merges the pair of [`Cluster`]s with the highest similarity.
fn clustering_pass(clusters: &mut Vec<Cluster>) {
    let nearest_pair = find_closest_pair(clusters);
    merge_clusters(clusters, nearest_pair);
}

/// Finds the closest pair among the given `clusters`.
fn find_closest_pair(clusters: &[Cluster]) -> ClusterPairMetadata {
    const EPSILON: f32 = 1e-5;
    let mut nearest_pair = ClusterPairMetadata {
        low: 0,
        high: 1,
        distance: f32::INFINITY,
    };
    for i in 0..clusters.len() {
        for j in (i + 1)..clusters.len() {
            let candidate_pair = ClusterPairMetadata {
                low: i,
                high: j,
                distance: jaccard_distance(&clusters[i].signature, &clusters[j].signature),
            };
            if candidate_pair.distance < nearest_pair.distance
                || ((candidate_pair.distance - nearest_pair.distance).abs() < EPSILON
                    && tie_break(candidate_pair, nearest_pair))
            {
                nearest_pair = candidate_pair;
            }
        }
    }
    nearest_pair
}

/// Merges the given `pair` among `clusters`.
fn merge_clusters(clusters: &mut Vec<Cluster>, pair: ClusterPairMetadata) {
    let right_cluster = clusters.remove(pair.high);
    let left_cluster = clusters.remove(pair.low);
    let parent_signature = left_cluster
        .signature
        .intersection(&right_cluster.signature)
        .copied()
        .collect();
    let parent_group = EntityGrouping {
        entities: Vec::new(),
        sub_groups: vec![left_cluster.group, right_cluster.group],
    };
    clusters.push(Cluster {
        signature: parent_signature,
        group: parent_group,
    });
}

/// Computes the normalized distance between two sets.
///
/// The returned value is between `0.0` and `1.0`,
/// where identical sets yield `0.0`
/// and disjoint sets yield `1.0`.
fn jaccard_distance(a: &HashSet<ComponentId>, b: &HashSet<ComponentId>) -> f32 {
    if a.is_empty() && b.is_empty() {
        return 0.0;
    }
    let intersection_size = a.intersection(b).count() as f32;
    if intersection_size == 0.0 {
        return 1.0;
    }
    let union_size = (a.len() + b.len()) as f32 - intersection_size;
    1.0 - (intersection_size / union_size.max(1.0))
}

/// Determines a preference when two clusters have equal distance.
fn tie_break(pair_a: ClusterPairMetadata, pair_b: ClusterPairMetadata) -> bool {
    let key = (pair_a.low.min(pair_a.high), pair_a.low.max(pair_a.high));
    let nearest_key = (pair_b.low.min(pair_b.high), pair_b.low.max(pair_b.high));
    key < nearest_key
}
