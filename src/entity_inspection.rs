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
        ComponentInspectionSettings,
    },
    entity_grouping::EntityGrouping,
    name_resolution,
    reflection_tools::{get_reflected_component_ref, reflected_value_to_string},
};

/// The result of inspecting an entity.
#[derive(Clone, Debug)]
pub struct EntityInspection {
    /// The entity being inspected.
    pub entity: Entity,
    /// The name of the entity, if any.
    pub name: Option<Name>,
    /// The components on the entity, in inspection form.
    pub components: Option<Vec<ComponentInspection>>,
    /// Information about how this entity was spawned.
    pub spawn_details: SpawnDetails,
}

impl Display for EntityInspection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut display_str = String::new();

        // Name and entity ID
        display_str.push_str(&format!(
            "{} ({})",
            self.resolve_name().unwrap_or("Entity".to_string()),
            self.entity
        ));

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

    /// Inspects multiple entities.
    fn inspect_multiple(
        &self,
        entities: impl ExactSizeIterator<Item = Entity>,
        settings: MultipleEntityInspectionSettings,
    ) -> Vec<Result<EntityInspection, EntityInspectionError>>;

    /// Inspects the component corresponding to the provided [`ComponentId`].
    ///
    /// The provided [`ComponentInspection`] contains details about the component,
    /// and can be logged using the [`Display`] trait.
    ///
    /// If you only want to inspect a specific component type, consider using
    /// [`inspect_component::<C>`](Self::inspect_component) instead.
    fn inspect_component_by_id(
        &self,
        component_id: ComponentId,
        entity: Entity,
        settings: ComponentInspectionSettings,
    ) -> Result<ComponentInspection, ComponentInspectionError>;

    /// Inspects the component of type `C`.
    ///
    /// The provided [`ComponentInspection`] contains details about the component,
    /// and can be logged using the [`Display`] trait.
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
        let name = self.get::<Name>(entity).cloned();

        // This unwrap is safe because `SpawnDetails` is always registered.
        let mut spawn_details_query = self.try_query::<SpawnDetails>().unwrap();

        let spawn_details = spawn_details_query.get(self, entity)?;

        // Temporary binding to avoid dropping borrow
        let entity_ref = self.entity(entity);

        let components = if settings.include_components {
            Some(
                entity_ref
                    .archetype()
                    .components()
                    .into_iter()
                    .map(|component_id| {
                        self.inspect_component_by_id(
                            *component_id,
                            entity,
                            settings.component_settings,
                        )
                    })
                    .filter_map(Result::ok)
                    .collect(),
            )
        } else {
            None
        };

        Ok(EntityInspection {
            entity,
            name,
            components,
            spawn_details,
        })
    }

    fn inspect_multiple(
        &self,
        entities: impl ExactSizeIterator<Item = Entity>,
        settings: MultipleEntityInspectionSettings,
    ) -> Vec<Result<EntityInspection, EntityInspectionError>> {
        {
            let entity_grouping = EntityGrouping::generate(self, entities);
            let entity_list = entity_grouping.flatten();

            let mut inspections = Vec::with_capacity(entity_list.len());
            for entity in entity_list {
                let inspection = self.inspect(entity, settings.entity_settings.clone());
                inspections.push(inspection);
            }
            inspections
        }
    }

    fn inspect_component_by_id(
        &self,
        component_id: ComponentId,
        entity: Entity,
        settings: ComponentInspectionSettings,
    ) -> Result<ComponentInspection, ComponentInspectionError> {
        let component_info = self.components().get_info(component_id).ok_or(
            ComponentInspectionError::ComponentNotRegistered(type_name::<ComponentId>()),
        )?;

        let name = component_info.name();
        let type_id = component_info.type_id();
        let type_registration = type_id.and_then(|type_id| {
            self.resource::<AppTypeRegistry>()
                .read()
                .get(type_id)
                .cloned()
        });

        let value = if settings.detail_level == ComponentDetailLevel::Names {
            None
        } else {
            Some(match type_id {
                Some(type_id) => match get_reflected_component_ref(&self, entity, type_id) {
                    Ok(reflected) => reflected_value_to_string(reflected, settings.full_type_names),
                    Err(err) => format!("<Unreflectable: {}>", err),
                },
                None => "Dynamic Type".to_string(),
            })
        };

        let name_definition_priority = type_id.and_then(|type_id| {
            self.resource::<name_resolution::NameResolutionRegistry>()
                .get_priority_by_type_id(type_id)
        });

        Ok(ComponentInspection {
            entity,
            component_id,
            name,
            value,
            name_definition_priority,
            type_id,
            type_registration,
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
        self.inspect_component_by_id(component_id, entity, settings)
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
