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
use core::any::type_name;
use core::fmt::Display;
use thiserror::Error;

use crate::{
    component_inspection::{
        ComponentDetailLevel, ComponentInspection, ComponentInspectionError,
        ComponentInspectionSettings, ComponentMetadataMap, ComponentTypeMetadata,
    },
    entity_grouping::EntityGrouping,
    memory_size::MemorySize,
    reflection_tools::{get_reflected_component_ref, reflected_value_to_string},
};

/// The result of inspecting an entity.
#[derive(Clone, Debug)]
pub struct EntityInspection {
    /// The entity being inspected.
    pub entity: Entity,
    /// The name of the entity, if any.
    pub name: Option<Name>,
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
}

impl From<QueryEntityError> for EntityInspectionError {
    fn from(err: QueryEntityError) -> Self {
        match err {
            QueryEntityError::EntityDoesNotExist(error) => {
                EntityInspectionError::EntityNotFound(error)
            }
            _ => panic!(
                "Unexpected QueryEntityError variant when inspecting an entity: {:?}",
                err
            ),
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
    /// A string to search for within entity names.
    ///
    /// Only entities with names containing this substring will be inspected.
    /// If `None`, all entities will be inspected.
    ///
    /// Defaults to `None`.
    pub name_filter: Option<String>,
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
        }
    }
}

/// An extension trait for inspecting entities.
///
/// This is required because this crate is not part of Bevy itself.
///
/// Ideally these methods would be on `EntityRef` and friends,
/// but accessing metadata about components requires access to the `World`,
/// which is not available, especially externally.
pub trait EntityInspectExtensionTrait {
    /// Inspects the provided entity.
    ///
    /// The provided [`EntityInspection`] contains details about the entity,
    /// and can be logged using the [`Display`] trait.
    fn inspect(
        &self,
        entity: Entity,
        settings: EntityInspectionSettings,
    ) -> Result<EntityInspection, EntityInspectionError>;

    /// Inspects the provided entity, using cached [`ComponentTypeMetadata`] for component type information.
    ///
    /// This method should be preferred when inspecting many entities,
    /// as it avoids re-computing component type metadata for each entity.
    fn inspect_cached(
        &self,
        entity: Entity,
        settings: &EntityInspectionSettings,
        metadata_map: &ComponentMetadataMap,
    ) -> Result<EntityInspection, EntityInspectionError>;

    /// Inspects multiple entities.
    ///
    /// The metadata_map parameter is mutable to allow caching of component metadata
    /// as needed during the inspection process. Any previously cached metadata will be reused,
    /// while new metadata will be added to the map for future use.
    ///
    /// If you need to update the metadata of component types between inspections,
    /// you should clear or modify the `metadata_map` before calling this method.
    fn inspect_multiple(
        &self,
        entities: impl ExactSizeIterator<Item = Entity>,
        settings: MultipleEntityInspectionSettings,
        metadata_map: &mut ComponentMetadataMap,
    ) -> Vec<Result<EntityInspection, EntityInspectionError>>;

    /// Inspects the component corresponding to the provided [`ComponentId`].
    ///
    /// The provided [`ComponentInspection`] contains details about the component,
    /// and can be logged using the [`Display`] trait.
    ///
    /// This is a low-level method; you will need to provide
    /// the appropriate [`ComponentTypeMetadata`] for the component being inspected.
    /// This can be obtained using [`ComponentTypeMetadata::new`], and should be cached
    /// if inspecting many components of the same type.
    ///
    /// If you only want to inspect a specific component type, consider using
    /// [`inspect_component::<C>`](Self::inspect_component) instead.
    fn inspect_component_by_id(
        &self,
        component_id: ComponentId,
        entity: Entity,
        metadata: &ComponentTypeMetadata,
        settings: ComponentInspectionSettings,
    ) -> Result<ComponentInspection, ComponentInspectionError>;

    /// Inspects the component of type `C`.
    ///
    /// The provided [`ComponentInspection`] contains details about the component,
    /// and can be logged using the [`Display`] trait.
    ///
    /// If you intend to call this method multiple times for the same component type,
    /// consider using [`inspect_component_by_id`](Self::inspect_component_by_id)
    /// with cached [`ComponentTypeMetadata`] instead for better performance.
    fn inspect_component<C: Component>(
        &self,
        entity: Entity,
        settings: ComponentInspectionSettings,
    ) -> Result<ComponentInspection, ComponentInspectionError>;
}

impl EntityInspectExtensionTrait for World {
    // When upstreamed, this should be a method on `EntityRef`.
    // It can't easily be one for now as we need access to the `World`
    // to get information from the `AppTypeRegistry` and `NameResolutionRegistry`.
    fn inspect(
        &self,
        entity: Entity,
        settings: EntityInspectionSettings,
    ) -> Result<EntityInspection, EntityInspectionError> {
        let metadata_map = ComponentMetadataMap::for_entity(self, entity);
        self.inspect_cached(entity, &settings, &metadata_map)
    }

    fn inspect_cached(
        &self,
        entity: Entity,
        settings: &EntityInspectionSettings,
        metadata_map: &ComponentMetadataMap,
    ) -> Result<EntityInspection, EntityInspectionError> {
        let name = self.get::<Name>(entity).cloned();

        // This unwrap is safe because `SpawnDetails` is always registered.
        let mut spawn_details_query = self.try_query::<SpawnDetails>().unwrap();

        let spawn_details = spawn_details_query.get(self, entity)?;

        // Temporary binding to avoid dropping borrow
        let entity_ref = self.entity(entity);

        let (components, total_memory_size) = if settings.include_components {
            let components: Vec<ComponentInspection> = entity_ref
                .archetype()
                .components()
                .iter()
                .map(|component_id| match metadata_map.get(component_id) {
                    Some(metadata) => self.inspect_component_by_id(
                        *component_id,
                        entity,
                        metadata,
                        settings.component_settings,
                    ),
                    None => Err(ComponentInspectionError::ComponentIdNotRegistered(
                        *component_id,
                    )),
                })
                .filter_map(Result::ok)
                .collect();

            let total_bytes = components
                .iter()
                .filter_map(|comp| comp.memory_size.as_ref())
                .fold(0usize, |acc, size| acc + size.as_bytes());
            let total_memory_size = MemorySize::new(total_bytes);

            (Some(components), Some(total_memory_size))
        } else {
            (None, None)
        };

        Ok(EntityInspection {
            entity,
            name,
            total_memory_size,
            components,
            spawn_details,
        })
    }

    fn inspect_multiple(
        &self,
        entities: impl ExactSizeIterator<Item = Entity>,
        settings: MultipleEntityInspectionSettings,
        metadata_map: &mut ComponentMetadataMap,
    ) -> Vec<Result<EntityInspection, EntityInspectionError>> {
        {
            metadata_map.update(self);

            let entity_grouping = EntityGrouping::generate(self, entities);
            let mut entity_list = entity_grouping.flatten();

            filter_entity_list_for_inspection(self, &mut entity_list, &settings);

            let mut inspections = Vec::with_capacity(entity_list.len());
            for entity in entity_list {
                let inspection =
                    self.inspect_cached(entity, &settings.entity_settings, &metadata_map);
                inspections.push(inspection);
            }
            inspections
        }
    }

    fn inspect_component_by_id(
        &self,
        component_id: ComponentId,
        entity: Entity,
        metadata: &ComponentTypeMetadata,
        settings: ComponentInspectionSettings,
    ) -> Result<ComponentInspection, ComponentInspectionError> {
        let component_info = self.components().get_info(component_id).ok_or(
            ComponentInspectionError::ComponentNotRegistered(type_name::<ComponentId>()),
        )?;

        let name = component_info.name();

        let (value, memory_size) = if settings.detail_level == ComponentDetailLevel::Names {
            (None, None)
        } else {
            match metadata.type_id {
                Some(type_id) => match get_reflected_component_ref(self, entity, type_id) {
                    Ok(reflected) => (
                        Some(reflected_value_to_string(
                            reflected,
                            settings.full_type_names,
                        )),
                        Some(MemorySize::new(size_of_val(reflected))),
                    ),
                    Err(err) => (Some(format!("<Unreflectable: {}>", err)), None),
                },
                None => (Some("Dynamic Type".to_string()), None),
            }
        };

        Ok(ComponentInspection {
            entity,
            component_id,
            name,
            memory_size,
            value,
        })
    }

    fn inspect_component<C: Component>(
        &self,
        entity: Entity,
        settings: ComponentInspectionSettings,
    ) -> Result<ComponentInspection, ComponentInspectionError> {
        let component_id = self.components().valid_component_id::<C>().ok_or(
            ComponentInspectionError::ComponentNotRegistered(type_name::<C>()),
        )?;

        // We're generating this on the fly; this is fine for a convenience method
        // intended for occasional logging.
        let metadata = ComponentTypeMetadata::new(self, component_id)?;

        self.inspect_component_by_id(component_id, entity, &metadata, settings)
    }
}

/// An extension trait for inspecting entities via Commands.
pub trait InspectExtensionCommandsTrait {
    /// Inspects the provided entity, logging details to the console using [`info!`].
    ///
    /// To inspect only a specific component on the entity, use
    /// [`inspect_component::<C>`](Self::inspect_component) instead.
    fn inspect(&mut self, settings: EntityInspectionSettings);

    /// Inspects the component of type `C` on the entity, logging details to the console using [`info!`].
    ///
    /// To inspect all components on the entity, use [`inspect`](Self::inspect) instead.
    fn inspect_component<C: Component>(&mut self, settings: ComponentInspectionSettings);
}

impl InspectExtensionCommandsTrait for EntityCommands<'_> {
    fn inspect(&mut self, settings: EntityInspectionSettings) {
        let entity = self.id();

        self.queue(move |entity_world_mut: EntityWorldMut| {
            let world = entity_world_mut.world();
            let inspection = world.inspect(entity, settings);
            match inspection {
                Ok(inspection) => info!("{}", inspection),
                Err(err) => warn!("Failed to inspect entity: {}", err),
            }
        });
    }

    fn inspect_component<C: Component>(&mut self, settings: ComponentInspectionSettings) {
        let entity = self.id();

        self.queue(move |entity_world_mut: EntityWorldMut| {
            let world = entity_world_mut.world();
            match world.inspect_component::<C>(entity, settings) {
                Ok(inspection) => info!("{}", inspection),
                Err(err) => info!("Failed to inspect component: {}", err),
            }
        });
    }
}

/// Filters the provided entity list in-place according to the provided [`MultipleEntityInspectionSettings`].
///
/// Calls [`does_entity_match_filter_for_inspection`] for each entity.
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
            .map(|name| name.contains(name_filter))
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
