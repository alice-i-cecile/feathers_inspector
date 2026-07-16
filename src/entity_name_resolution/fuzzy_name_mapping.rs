//! Tools to fuzzily map a component or resource name to its corresponding ID.
//!
//! This is useful for GUI and text-based inspection tools,
//! where users may want to search for components or resources by name,
//! but may not know the exact spelling or formatting of the name.

use bevy::ecs::component::ComponentId;
use bevy::ecs::world::World;
use strsim::jaro_winkler;

/// Minimum required similarity for inclusion in a fuzzy mapping.
const THRESHOLD: f64 = 0.6;

/// Attempts to find a [`ComponentId`] for the given fuzzy component name.
///
/// A vector of candidate matches will be returned, with the best-effort match first.
/// If no suitable match could be found, an empty vector will be returned.
/// The returned `f64` is a normalized similarity score between `0.0` and `1.0`,
/// where `1.0` is an exact match.
///
/// See [`fuzzy_resource_name_to_id`] for a similar function for resources.
///
/// Matching uses normalized Levenshtein similarity to find the closest match,
/// and is case-insensitive and ignores leading/trailing whitespace.
/// Only the "shortname" of the component (i.e., without module paths) is considered.
pub fn fuzzy_component_name_to_id(world: &World, fuzzy_name: &str) -> Vec<(f64, ComponentId)> {
    let candidates = world.components().iter_registered().map(|info| info.id());
    fuzzy_name_to_id(world, fuzzy_name, candidates)
}

/// Attempts to find a [`ComponentId`] for the given fuzzy resource name.
///
/// A vector of candidate matches will be returned, with the best-effort match first.
/// If no suitable match could be found, an empty vector will be returned.
/// The returned `f64` is a normalized similarity score between `0.0` and `1.0`,
/// where `1.0` is an exact match.
///
/// See [`fuzzy_component_name_to_id`] for a similar function for components.
///
/// Matching uses normalized Levenshtein similarity to find the closest match,
/// and is case-insensitive and ignores leading/trailing whitespace.
/// Only the "shortname" of the resource (i.e., without module paths) is considered.
pub fn fuzzy_resource_name_to_id(world: &World, fuzzy_name: &str) -> Vec<(f64, ComponentId)> {
    // We can restrict the candidate set to the component id values that are registered as resources,
    // allowing us to share code with the component equivalent above.
    let candidates = world.resource_entities().iter().map(|(id, _)| id);
    fuzzy_name_to_id(world, fuzzy_name, candidates)
}

/// Finds the best fuzzy match for `fuzzy_name` among the provided candidate [`ComponentId`]s.
///
/// A vector of candidate matches will be returned, with the best-effort match first.
/// If no suitable match could be found, an empty vector will be returned.
///
/// The returned `f64` is a normalized similarity score between `0.0` and `1.0`,
/// where `1.0` is an exact match.
///
/// This is normalized by trimming whitespace and converting to lowercase.
/// An exact (post-normalization) match short-circuits and is always preferred.
///
/// Shortnames (i.e., without module paths) are used for matching.
fn fuzzy_name_to_id(
    world: &World,
    fuzzy_name: &str,
    candidates: impl Iterator<Item = ComponentId>,
) -> Vec<(f64, ComponentId)> {
    let processed_fuzzy_name = fuzzy_name.trim().to_lowercase();

    // PERF: it is almost certainly more efficient to build an accelerated structure
    // across all possible names once, rather than re-computing distances
    // whenever a user enters a new fuzzy name.
    let mut matches = Vec::with_capacity(5);
    for id in candidates {
        let Some(name) = world.components().get_name(id) else {
            continue;
        };
        let processed_name = name.shortname().to_string().trim().to_lowercase();

        if processed_fuzzy_name == processed_name {
            return vec![(1.0, id)];
        }
        let similarity = jaro_winkler(&processed_fuzzy_name, &processed_name);
        if similarity >= THRESHOLD {
            matches.push((similarity, id));
        }
    }

    matches.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    matches
}
