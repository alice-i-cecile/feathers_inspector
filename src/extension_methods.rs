//! Methods that should exist on existing Bevy types.

use bevy::{
    ecs::{component::ComponentId, query::SpawnDetails},
    prelude::*,
};
use core::any::type_name;

use crate::{
    entity_grouping::EntityGrouping,
    entity_name_resolution::resolve_name,
    inspection::component_inspection::{
        ComponentDetailLevel, ComponentInspection, ComponentInspectionError,
        ComponentInspectionSettings, ComponentMetadataMap, ComponentTypeInspection,
        ComponentTypeMetadata,
    },
    inspection::entity_inspection::{
        EntityInspection, EntityInspectionError, EntityInspectionSettings,
        MultipleEntityInspectionSettings, filter_entity_list_for_inspection,
    },
    inspection::resource_inspection::{
        ResourceInspection, ResourceInspectionError, ResourceInspectionSettings,
    },
    memory_size::MemorySize,
    reflection_tools::{
        get_reflected_component_ref, get_reflected_resource_ref, reflected_value_to_string,
    },
};

/// An extension trait for inspecting ECS objects, for methods that should belong on [`World`].
///
/// This is required because this crate is not part of Bevy itself.
pub trait WorldInspectionExtensionTrait {
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
        entities: impl IntoIterator<Item = Entity>,
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

    /// Inspects the provided resource type.
    ///
    /// The provided [`ResourceInspection`] contains details about the resource,
    /// and can be logged using the [`Display`] trait.
    ///
    /// The default values for [`ResourceInspectionSettings`] will be used.
    fn inspect_resource<R: Resource>(
        &self,
        settings: ResourceInspectionSettings,
    ) -> Result<ResourceInspection, ResourceInspectionError>;

    /// Inspects a resource by the provided [`ComponentId`].
    ///
    /// This is the dynamically-typed variant of [`inspect_resource`](Self::inspect_resource).
    fn inspect_resource_by_id(
        &self,
        component_id: ComponentId,
        settings: ResourceInspectionSettings,
    ) -> Result<ResourceInspection, ResourceInspectionError>;

    /// Inspects all resources in the world.
    ///
    /// Returns a vector of [`ResourceInspection`]s for all resources found.
    fn inspect_all_resources(
        &self,
        settings: ResourceInspectionSettings,
    ) -> Vec<ResourceInspection>;

    /// Inspects the provided component type `C`, providing information about the type itself.
    ///
    /// For a dynamically-typed variant, use [`inspect_component_by_id`](Self::inspect_component_by_id).
    // These methods require `&mut World` because `QueryBuilder` currently requires it in all cases.
    fn inspect_component_type<C: Component>(
        &self,
    ) -> Result<ComponentTypeInspection, ComponentInspectionError>;

    /// Inspects the provided component by its [`ComponentId`], providing information about the type itself.
    ///
    /// This is the dynamically-typed variant of [`inspect_component_type`](Self::inspect_component_type).
    fn inspect_component_type_by_id(
        &self,
        component_id: ComponentId,
    ) -> Result<ComponentTypeInspection, ComponentInspectionError>;
}

impl WorldInspectionExtensionTrait for World {
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
        // This unwrap is safe because `SpawnDetails` is always registered.
        let mut spawn_details_query = self.try_query::<SpawnDetails>().unwrap();

        let spawn_details = Some(spawn_details_query.get(self, entity)?);

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
        let name = resolve_name(self, entity, &components, metadata_map);

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
        entities: impl IntoIterator<Item = Entity>,
        settings: MultipleEntityInspectionSettings,
        metadata_map: &mut ComponentMetadataMap,
    ) -> Vec<Result<EntityInspection, EntityInspectionError>> {
        {
            metadata_map.update(self);

            let entity_grouping =
                EntityGrouping::generate(self, entities, settings.grouping_strategy);
            let mut entity_list = entity_grouping.flatten();

            filter_entity_list_for_inspection(self, &mut entity_list, &settings);

            let mut inspections = Vec::with_capacity(entity_list.len());
            for entity in entity_list {
                let inspection =
                    self.inspect_cached(entity, &settings.entity_settings, metadata_map);
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
            ComponentInspectionError::ComponentIdNotRegistered(component_id),
        )?;

        if !self.entity(entity).contains_id(component_id) {
            return Err(ComponentInspectionError::ComponentNotFound(component_id));
        }

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

    fn inspect_resource<R: Resource>(
        &self,
        settings: ResourceInspectionSettings,
    ) -> Result<ResourceInspection, ResourceInspectionError> {
        let component_id = self.components().resource_id::<R>().ok_or(
            ResourceInspectionError::ResourceNotRegistered(type_name::<R>()),
        )?;
        self.inspect_resource_by_id(component_id, settings)
    }

    fn inspect_resource_by_id(
        &self,
        component_id: ComponentId,
        settings: ResourceInspectionSettings,
    ) -> Result<ResourceInspection, ResourceInspectionError> {
        let component_info = self
            .components()
            .get_info(component_id)
            .ok_or(ResourceInspectionError::ResourceNotFound(component_id))?;

        let name = component_info.name();
        let type_id = component_info.type_id();

        let type_registry = self.resource::<AppTypeRegistry>();
        let type_registration = match type_id {
            Some(type_id) => type_registry.read().get(type_id).cloned(),
            None => None,
        };

        let (value, memory_size) = match type_id {
            Some(type_id) => match get_reflected_resource_ref(self, type_id) {
                Ok(reflected) => (
                    reflected_value_to_string(reflected, settings.full_type_names),
                    Some(MemorySize::new(size_of_val(reflected))),
                ),
                Err(err) => (format!("<Unreflectable: {}>", err), None),
            },
            None => ("Dynamic Type".to_string(), None),
        };

        Ok(ResourceInspection {
            component_id,
            name,
            value,
            type_id,
            memory_size,
            type_registration,
        })
    }

    fn inspect_all_resources(
        &self,
        settings: ResourceInspectionSettings,
    ) -> Vec<ResourceInspection> {
        let mut inspections = Vec::new();

        for (component_info, _ptr) in self.iter_resources() {
            let component_id = component_info.id();
            if let Ok(inspection) = self.inspect_resource_by_id(component_id, settings) {
                inspections.push(inspection);
            }
        }

        inspections
    }

    fn inspect_component_type<C: Component>(
        &self,
    ) -> Result<ComponentTypeInspection, ComponentInspectionError> {
        let component_id = self.components().valid_component_id::<C>().ok_or(
            ComponentInspectionError::ComponentNotRegistered(type_name::<C>()),
        )?;

        self.inspect_component_type_by_id(component_id)
    }

    fn inspect_component_type_by_id(
        &self,
        component_id: ComponentId,
    ) -> Result<ComponentTypeInspection, ComponentInspectionError> {
        let metadata = ComponentTypeMetadata::new(self, component_id)?;

        // TODO: this should use the component index cache on `Archetypes` when that becomes public.
        let mut entity_count = 0usize;
        for archetype in self.archetypes().iter() {
            if archetype.contains(component_id) {
                entity_count += archetype.len() as usize;
            }
        }

        Ok(ComponentTypeInspection {
            entity_count,
            metadata,
        })
    }
}

/// An extension trait for inspection methods that belong on [`EntityCommands`].
pub trait EntityCommandsInspectionTrait {
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

impl EntityCommandsInspectionTrait for EntityCommands<'_> {
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
                Err(err) => warn!("Failed to inspect component: {}", err),
            }
        });
    }
}

/// An extension trait for inspection methods that belong on [`Commands`].
pub trait CommandsExtensionTrait {
    /// Inspects the provided resource type, logging details to the console using [`info!`].
    fn inspect_resource<R: Resource>(&mut self, settings: ResourceInspectionSettings);

    /// Inspects all resources in the world, logging details to the console using [`info!`].
    fn inspect_all_resources(&mut self, settings: ResourceInspectionSettings);
}

impl CommandsExtensionTrait for Commands<'_, '_> {
    fn inspect_resource<R: Resource>(&mut self, settings: ResourceInspectionSettings) {
        self.queue(move |world: &mut World| {
            let inspection = world.inspect_resource::<R>(settings);
            match inspection {
                Ok(inspection) => info!("{inspection}"),
                Err(err) => warn!("Failed to inspect resource: {}", err),
            }
        });
    }

    fn inspect_all_resources(&mut self, settings: ResourceInspectionSettings) {
        self.queue(move |world: &mut World| {
            let mut inspections = world.inspect_all_resources(settings);
            // Alphabetically sort the inspections by resource name
            inspections.sort_by(|a, b| {
                a.name
                    .shortname()
                    .to_string()
                    .cmp(&b.name.shortname().to_string())
            });

            let mut log_string = format!("Inspecting all resources ({} found):", inspections.len());
            for inspection in &inspections {
                // PERF: we can probably reduce allocations by constructing this better
                log_string.push_str(&format!("\n- {}", inspection));
            }

            info!("{log_string}");
        });
    }
}
