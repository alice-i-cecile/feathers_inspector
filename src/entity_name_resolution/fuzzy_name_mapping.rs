//! Tools to fuzzily map a component or resource name to its corresponding ID.
//!
//! This is useful for GUI and text-based inspection tools,
//! where users may want to search for components or resources by name,
//! but may not know the exact spelling or formatting of the name.

use bevy::ecs::component::ComponentId;
use bevy::ecs::world::World;
use strsim::normalized_levenshtein;

/// Minimum required similarity for inclusion in a fuzzy mapping.
const THRESHOLD: f64 = 0.3;

/// Attempts to find a [`ComponentId`] for the given fuzzy component name.
///
/// A best-effort match will be returned,
/// or `None` if no suitable match could be found.
///
/// See [`fuzzy_resource_name_to_id`] for a similar function for resources.
///
/// Matching uses normalized Levenshtein similarity to find the closest match,
/// and is case-insensitive and ignores leading/trailing whitespace.
/// Only the "shortname" of the component (i.e., without module paths) is considered.
pub fn fuzzy_component_name_to_id(world: &World, fuzzy_name: &str) -> Option<ComponentId> {
    let candidates = world.components().iter_registered().map(|info| info.id());
    fuzzy_name_to_id(world, fuzzy_name, candidates)
}

/// Attempts to find a [`ComponentId`] for the given fuzzy resource name.
///
/// A best-effort match will be returned,
/// or `None` if no suitable match could be found.
///
/// See [`fuzzy_component_name_to_id`] for a similar function for components.
///
/// Matching uses normalized Levenshtein similarity to find the closest match,
/// and is case-insensitive and ignores leading/trailing whitespace.
/// Only the "shortname" of the component (i.e., without module paths) is considered.
pub fn fuzzy_resource_name_to_id(world: &World, fuzzy_name: &str) -> Option<ComponentId> {
    // We can restrict the candidate set to the component id values that are registered as resources,
    // allowing us to share code with the component equivalent above.
    let candidates = world.resource_entities().iter().map(|(id, _)| id);
    fuzzy_name_to_id(world, fuzzy_name, candidates)
}

/// Finds the best fuzzy match for `fuzzy_name` among the provided candidate [`ComponentId`]s.
///
/// Matching uses normalized Levenshtein similarity over each candidate's "shortname",
/// which trims module paths.
///
/// This is normalized by trimming whitespace and converting to lowercase.
/// An exact (post-normalization) match short-circuits and is always preferred.
fn fuzzy_name_to_id(
    world: &World,
    fuzzy_name: &str,
    candidates: impl Iterator<Item = ComponentId>,
) -> Option<ComponentId> {
    let processed_fuzzy_name = fuzzy_name.trim().to_lowercase();

    // PERF: it is almost certainly more efficient to build an accelerated structure
    // across all possible names once, rather than re-computing distances each time.
    let mut best_match: Option<(ComponentId, f64)> = None;
    for id in candidates {
        let Some(name) = world.components().get_name(id) else {
            continue;
        };
        let processed_name = name.shortname().to_string().trim().to_lowercase();

        if processed_fuzzy_name == processed_name {
            return Some(id);
        }
        let similarity = normalized_levenshtein(&processed_fuzzy_name, &processed_name);
        if similarity >= THRESHOLD && best_match.is_none_or(|best_match| similarity > best_match.1)
        {
            best_match = Some((id, similarity));
        }
    }

    best_match.map(|(id, _)| id)
}
