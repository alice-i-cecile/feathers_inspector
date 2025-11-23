//! Types and traits for inspecting Bevy entities.
//!
//! Entities are composed of components, but this module focuses on
//! inspecting the entity as a whole.
//!
//! See the [`component_inspection`](crate::component_inspection) module
//! for information about inspecting and displaying components.

use bevy::{
    ecs::{
        component::ComponentId,
        entity::EntityDoesNotExistError,
        query::{QueryEntityError, SpawnDetails},
    },
    prelude::*,
};
use core::fmt::Display;
use thiserror::Error;

use crate::{
    component_inspection::{
        ComponentDetailLevel, ComponentInspection, ComponentInspectionSettings,
    },
    entity_grouping::GroupingStrategy,
    entity_name_resolution::EntityName,
    memory_size::MemorySize,
};

/// The result of inspecting an entity.
#[derive(Clone, Debug)]
pub struct EntityInspection {
    /// The entity being inspected.
    pub entity: Entity,
    /// The name of the entity, if any.
    pub name: Option<EntityName>,
    /// The total size of the entity in memory.
    ///
    /// This is computed as the sum of the sizes of all its components,
    /// and is likely to be an underestimate as non-reflected components
    /// will not contribute to the total size.
    ///
    /// If [`include_components`](EntityInspectionSettings::include_components) is false,
    /// this will always be [`None`].
    pub total_memory_size: Option<MemorySize>,
    /// The components on the entity, in inspection form.
    pub components: Option<Vec<ComponentInspection>>,
    /// Information about how this entity was spawned.
    pub spawn_details: SpawnDetails,
}

impl Display for EntityInspection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut display_str = String::new();

        let name_str = match &self.name {
            Some(name) => name.as_str(),
            None => "Entity",
        };
        display_str.push_str(&format!("{name_str} ({})", self.entity));
        if let Some(total_size) = &self.total_memory_size {
            display_str.push_str(&format!("\nMemory Size: {}", total_size));
        }

        let maybe_location = &self.spawn_details.spawned_by();
        let tick = &self.spawn_details.spawn_tick();

        if let Some(location) = maybe_location.into_option() {
            display_str.push_str(&format!(
                "\nSpawned by: {location} on system tick {}",
                tick.get()
            ));
        } else {
            warn_once!(
                "Entity {:?} has no spawn location information available. Consider enabling \
                 the `track_location` feature for better debugging.",
                self.entity
            );
        }

        if let Some(components) = &self.components {
            display_str.push_str("\nComponents:");
            for component in components {
                display_str.push_str(&format!("\n- {}", component));
            }
        }
        write!(f, "{display_str}")?;

        Ok(())
    }
}

/// An error that can occur when attempting to inspect an entity.
#[derive(Debug, Error)]
pub enum EntityInspectionError {
    /// The entity does not exist in the world.
    #[error("Entity not found: {0}")]
    EntityNotFound(EntityDoesNotExistError),
    /// A catch-all variant for inspection errors that should never happen
    /// when just querying an entity and its metadata.
    #[error("Unexpected QueryEntityError: {0}")]
    UnexpectedQueryError(QueryEntityError),
}

impl From<QueryEntityError> for EntityInspectionError {
    fn from(err: QueryEntityError) -> Self {
        match err {
            QueryEntityError::EntityDoesNotExist(error) => {
                EntityInspectionError::EntityNotFound(error)
            }
            _ => {
                error!("Unexpected QueryEntityError variant when inspecting an entity: {err:?}");
                EntityInspectionError::UnexpectedQueryError(err)
            }
        }
    }
}

/// Settings for inspecting an individual entity.
#[derive(Clone, Debug)]
pub struct EntityInspectionSettings {
    /// Should component information be included in the inspection?
    ///
    /// Note that component-based name resolution will not work if components are not included.
    ///
    /// The detail level of component information can be further configured
    /// using [`ComponentInspectionSettings::detail_level`].
    pub include_components: bool,
    /// Settings used when inspecting components on the entity.
    pub component_settings: ComponentInspectionSettings,
}

impl Default for EntityInspectionSettings {
    fn default() -> Self {
        Self {
            include_components: true,
            component_settings: ComponentInspectionSettings::default(),
        }
    }
}

/// Settings for inspecting multiple entities at once.
#[derive(Clone, Debug)]
pub struct MultipleEntityInspectionSettings {
    /// A [`NameFilter`] to search for within entity names.
    ///
    /// Only entities whose name matches this filter will be inspected.
    /// If `None`, all entities will be inspected.
    ///
    /// Defaults to `None`.
    pub name_filter: Option<NameFilter>,
    /// Components that must be present on each entity to be inspected.
    /// If empty, no component presence filtering will be applied.
    ///
    /// Defaults to an empty list.
    pub with_component_filter: Vec<ComponentId>,
    /// Components that must not be present on each entity to be inspected.
    /// If empty, no component absence filtering will be applied.
    ///
    /// Defaults to an empty list.
    pub without_component_filter: Vec<ComponentId>,
    /// Settings used when inspecting each individual entity.
    ///
    /// Note that the default values are not the same as [`EntityInspectionSettings::default`].
    ///
    /// By default, only component names are included to improve performance
    /// and improve readability when inspecting many entities at once.
    pub entity_settings: EntityInspectionSettings,
    /// Specifies how entities should be grouped.
    pub grouping_strategy: GroupingStrategy,
}

impl Default for MultipleEntityInspectionSettings {
    fn default() -> Self {
        Self {
            name_filter: None,
            with_component_filter: Vec::new(),
            without_component_filter: Vec::new(),
            entity_settings: EntityInspectionSettings {
                component_settings: ComponentInspectionSettings {
                    detail_level: ComponentDetailLevel::Names,
                    ..Default::default()
                },
                ..Default::default()
            },
            grouping_strategy: GroupingStrategy::Hierarchy,
        }
    }
}

/// A filter for named entities.
///
/// For convenience, the [`From`] trait has been implemented
/// for converting from [`String`], `&String` and [`&str`],
/// so you can construct using `NameFilter::from("name")`.
/// Keep in mind that in this case the matches will be case-insensitive.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NameFilter {
    /// The substring that entities have to match to be included.
    query: String,
    /// Whether case matters.
    case_sensitive: bool,
}

impl NameFilter {
    /// Constructs a new [`NameFilter`].
    ///
    /// The given `query` is pre-lowercased if `case_sensitive` is `false`.
    pub fn new(mut query: String, case_sensitive: bool) -> Self {
        if !case_sensitive {
            query = query.to_lowercase();
        }
        Self {
            query,
            case_sensitive,
        }
    }

    /// Whether the given `name` matches this filter.
    pub fn matches(&self, name: &str) -> bool {
        if self.case_sensitive {
            name.contains(&self.query)
        } else {
            name.to_lowercase().contains(&self.query)
        }
    }
}

impl From<String> for NameFilter {
    fn from(value: String) -> Self {
        Self {
            query: value.to_lowercase(),
            case_sensitive: false,
        }
    }
}

impl From<&String> for NameFilter {
    fn from(value: &String) -> Self {
        Self {
            query: value.to_lowercase(),
            case_sensitive: false,
        }
    }
}

impl From<&str> for NameFilter {
    fn from(value: &str) -> Self {
        Self {
            query: value.to_lowercase(),
            case_sensitive: false,
        }
    }
}

/// Filters the provided entity list in-place according to the provided [`MultipleEntityInspectionSettings`].
// PERF: this might be faster if you build a dynamic query instead of checking each entity individually.
pub fn filter_entity_list_for_inspection(
    world: &World,
    entities: &mut Vec<Entity>,
    settings: &MultipleEntityInspectionSettings,
) {
    entities.retain(|entity| does_entity_match_inspection_filter(world, *entity, settings));
}

/// Checks if a single entity matches the provided [`MultipleEntityInspectionSettings`].
fn does_entity_match_inspection_filter(
    world: &World,
    entity: Entity,
    settings: &MultipleEntityInspectionSettings,
) -> bool {
    let entity_ref = match world.get_entity(entity) {
        Ok(entity_ref) => entity_ref,
        Err(_) => return false,
    };

    if let Some(name_filter) = &settings.name_filter {
        let name_matches = world
            .get::<Name>(entity)
            .map(|name| name_filter.matches(name.as_str()))
            .unwrap_or(false);
        if !name_matches {
            return false;
        }
    }

    for component_id in &settings.with_component_filter {
        if !entity_ref.contains_id(*component_id) {
            return false;
        }
    }

    for component_id in &settings.without_component_filter {
        if entity_ref.contains_id(*component_id) {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use crate::entity_inspection::NameFilter;

    #[test]
    fn case_insensitive() {
        let filter = NameFilter::from("Foo");
        let names = ["foobar", "FOOBAR", "barfoo", "bar"];
        for (i, name) in names.iter().enumerate() {
            let matches = filter.matches(name);
            let expected = match i {
                0 => true,
                1 => true,
                2 => true,
                3 => false,
                _ => unreachable!(),
            };
            assert_eq!(matches, expected)
        }
    }

    #[test]
    fn case_sensitive() {
        let filter = NameFilter::new("Foo".into(), true);
        let names = ["foobar", "FooBar", "bar"];
        for (i, name) in names.iter().enumerate() {
            let matches = filter.matches(name);
            let expected = match i {
                0 => false,
                1 => true,
                2 => false,
                _ => unreachable!(),
            };
            assert_eq!(matches, expected)
        }
    }
}
