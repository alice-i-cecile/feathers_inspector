//! Tools to fuzzily map a component or resource name to its corresponding ID.
//!
//! This is useful for GUI and text-based inspection tools,
//! where users may want to search for components or resources by name,
//! but may not know the exact spelling or formatting of the name.

use bevy::ecs::component::ComponentId;
use bevy::ecs::world::World;
use strsim::levenshtein;

/// Attempts to find a [`ComponentId`] for the given fuzzy component name.
///
/// A best-effort match will be returned,
/// or `None` if no suitable match could be found.
///
/// See [`fuzzy_resource_name_to_id`] for a similar function for resources.
///
/// Matching uses Levenshtein distance to find the closest match,
/// and is case-insensitive and ignores leading/trailing whitespace.
/// Only the "shortname" of the component (i.e., without module paths) is considered.
pub fn fuzzy_component_name_to_id(world: &World, fuzzy_name: &str) -> Option<ComponentId> {
    let processed_fuzzy_name = fuzzy_name.trim().to_lowercase();

    let component_names: Vec<(ComponentId, String)> = world
        .components()
        .iter_registered()
        .map(|info| {
            let id = info.id();
            let name = info.name();
            let processed_name = name.shortname().to_string().trim().to_lowercase();
            (id, processed_name)
        })
        .collect();

    // PERF: it is almost certainly more efficient to build an accelerated structure
    // across all possible names once, rather than re-computing distances each time.
    let mut best_match: Option<(ComponentId, usize)> = None;
    for (id, name) in component_names {
        let distance = levenshtein(&processed_fuzzy_name, &name);
        if best_match.is_none() || distance < best_match.as_ref().unwrap().1 {
            best_match = Some((id, distance));
        }
    }

    best_match.map(|(id, _)| id)
}

/// Attempts to find a [`ComponentId`] for the given fuzzy resource name.
///
/// A best-effort match will be returned,
/// or `None` if no suitable match could be found.
///
/// See [`fuzzy_component_name_to_id`] for a similar function for components.
///
/// Matching uses Levenshtein distance to find the closest match,
/// and is case-insensitive and ignores leading/trailing whitespace.
/// Only the "shortname" of the component (i.e., without module paths) is considered.
pub fn fuzzy_resource_name_to_id(world: &World, fuzzy_name: &str) -> Option<ComponentId> {
    let processed_fuzzy_name = fuzzy_name.trim().to_lowercase();

    // TODO: this should be much easier to look up, but Bevy's API for this is limited.
    let resources = &world.storages().resources;
    let resource_ids = resources.iter().map(|(id, _)| id);
    let resource_names: Vec<(ComponentId, String)> = resource_ids
        .filter_map(|id| {
            let name = world.components().get_name(id)?;
            let processed_name = name.shortname().to_string().trim().to_lowercase();
            Some((id, processed_name))
        })
        .collect();

    // PERF: it is almost certainly more efficient to build an accelerated structure
    // across all possible names once, rather than re-computing distances each time.
    let mut best_match: Option<(ComponentId, usize)> = None;
    for (id, name) in resource_names {
        let distance = levenshtein(&processed_fuzzy_name, &name);
        if best_match.is_none() || distance < best_match.as_ref().unwrap().1 {
            best_match = Some((id, distance));
        }
    }

    best_match.map(|(id, _)| id)
}
