//! Types and traits for inspecting Bevy entities.

use bevy::ecs::component::StorageType;
use bevy::platform::collections::HashMap;
use bevy::{ecs::component::ComponentId, prelude::*, reflect::TypeRegistration};
use core::any::TypeId;
use core::fmt::Display;
use thiserror::Error;

use crate::entity_name_resolution::NameResolutionRegistry;
use crate::memory_size::MemorySize;

/// The result of inspecting a component.
///
/// Log this using the [`Display`] trait to see details about the component.
/// [`Debug`] can also be used for more detailed but harder to-read output.
///
/// This should be paired with [`ComponentTypeMetadata`] to get full type information.
/// [`ComponentTypeMetadata`] can be retrieved via [`ComponentTypeMetadata::new`],
/// and is relatively heavy to compute and store. You should cache it if inspecting many
/// components of the same type.
///
/// To inspect a component type itself, see [`ComponentTypeInspection`].
#[derive(Clone, Debug)]
pub struct ComponentInspection {
    /// The entity that owns the component.
    pub entity: Entity,
    /// The [`ComponentId`] of the component.
    pub component_id: ComponentId,
    /// The type name of the component.
    ///
    /// This is duplicated from the metadata for convenience and [`Display`] printing.
    pub name: DebugName,
    /// The size, in bytes, of the component value.
    ///
    /// Note that this may differ from the size of the component type
    /// if the component is a dynamically-sized type: heap-allocated data is not included.
    ///
    /// Computing this value requires reflection of the component value.
    /// As a result, it may be `None` if the component type is not reflected and registered,
    /// or if [`ComponentDetailLevel::Names`] was specified when inspecting the component.
    pub memory_size: Option<MemorySize>,
    /// The value of the component as a string.
    ///
    /// This information is gathered via reflection,
    /// and used for debugging purposes.
    pub value: Option<String>,
}

impl Display for ComponentInspection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let shortname = self.name.shortname();

        match &self.value {
            Some(value) => match &self.memory_size {
                Some(size) => write!(f, "{shortname} ({}): {value}", size)?,
                None => write!(f, "{shortname}: {value}")?,
            },
            None => match &self.memory_size {
                Some(size) => write!(f, "{shortname} ({})", size)?,
                None => write!(f, "{shortname}")?,
            },
        }

        Ok(())
    }
}

/// Metadata about a specific component type, designed to be transmitted and inspected.
///
/// For information about the specific value of a component on an entity,
/// see [`ComponentInspection`].
///
/// This is the major component of [`ComponentTypeInspection`].
/// Log this using the [`Display`] trait to see details about the component type.
///
/// These are relatively heavy to compute and store,
/// consider caching them in a [`ComponentMetadataMap`] if inspecting many entities or components.
///
/// Notably, this type does not include [`ComponentInfo`](bevy::ecs::component::ComponentInfo)
/// directly, as that type is not `Send + Sync` and cannot be stored in many contexts.
/// Instead, this type extracts all relevant information that can be stored and used later.
///
#[derive(Clone, Debug)]
pub struct ComponentTypeMetadata {
    /// The [`ComponentId`] of the component type.
    ///
    /// This is generally stored as a key in [`ComponentMetadataMap`],
    /// but is duplicated here for convenience.
    pub component_id: ComponentId,
    /// The type name of the component.
    pub name: DebugName,
    /// The [`TypeId`] of the component type.
    ///
    /// Note that dynamic types will not have a [`TypeId`].
    pub type_id: Option<TypeId>,
    /// The minimum size in bytes of the component type.
    ///
    /// This is computed via [`core::alloc::Layout`], and does not include any heap allocations.
    /// For dynamically-sized types, this is the size of the pointer or handle stored in the ECS.
    pub memory_size: MemorySize,
    /// The name definition priority of the component type.
    /// Higher values indicate higher priority.
    /// `None` indicates that the component does not define names.
    pub name_definition_priority: Option<i8>,
    /// Returns true if the component type is mutable while in the ECS.
    pub mutable: bool,
    /// The storage type of this component.
    pub storage_type: StorageType,
    /// Returns true if the underlying component type can freely be shared across threads.
    pub is_send_and_sync: bool,
    /// The list of components required by this component,
    /// which will automatically be added when this component is added to an entity.
    pub required_components: Vec<ComponentId>,
    /// The type information of the component.
    ///
    /// This contains metadata about the component's type,
    /// such as its fields and methods,
    /// as well as any reflected traits it implements.
    ///
    /// Note: this may be `None` if the type is not reflected and registered in the type registry.
    /// Currently, generic types need to be manually registered,
    /// and dynamically-typed components cannot be registered.
    pub type_registration: Option<TypeRegistration>,
}

impl ComponentTypeMetadata {
    /// Extracts the required metadata from the world for the given component ID.
    pub fn new(world: &World, component_id: ComponentId) -> Result<Self, ComponentInspectionError> {
        let component_info = world.components().get_info(component_id).ok_or(
            ComponentInspectionError::ComponentIdNotRegistered(component_id),
        )?;

        let name = component_info.name();
        let type_id = component_info.type_id();
        let type_registration = type_id.and_then(|type_id| {
            world
                .resource::<AppTypeRegistry>()
                .read()
                .get(type_id)
                .cloned()
        });

        let memory_size = MemorySize::new(component_info.layout().size());

        let name_definition_priority = match type_id {
            Some(type_id) => world
                .get_resource::<NameResolutionRegistry>()
                .expect("`NameResolutionPlugin` must be present")
                .get_priority_by_type_id(type_id),
            None => None,
        };

        Ok(Self {
            component_id,
            name,
            type_id,
            name_definition_priority,
            memory_size,
            mutable: component_info.mutable(),
            storage_type: component_info.storage_type(),
            is_send_and_sync: component_info.is_send_and_sync(),
            required_components: component_info.required_components().iter_ids().collect(),
            type_registration,
        })
    }
}

impl Display for ComponentTypeMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} (Size: {}, Storage: {:?})\n Type Registration: {:?}",
            self.name.shortname(),
            self.memory_size,
            self.storage_type,
            // TODO: `TypeRegistration` and `TypeInfo` don't implement `Display`,
            // so we just use `Debug` for now.
            self.type_registration,
        )
    }
}

/// The result of inspecting a component type.
///
/// This is distinct from [`ComponentInspection`], which inspects a specific component on an entity.
///
/// Call [`World::inspect_component_type`] to get this information.
///
/// [`World::inspect_component_type`]: crate::extension_methods::WorldInspectionExtensionTrait::inspect_component_type
#[derive(Clone, Debug)]
pub struct ComponentTypeInspection {
    /// The number of entities that have a component of this type.
    pub entity_count: usize,
    /// Metadata about the component type.
    ///
    /// This information does not vary based on the state of the world,
    /// and can safely be cached and reused.
    pub metadata: ComponentTypeMetadata,
}

impl Display for ComponentTypeInspection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\n Entities with this component: {}",
            self.metadata, self.entity_count
        )
    }
}

/// A map of component IDs to their corresponding metadata.
///
/// This is useful for caching component metadata
/// when inspecting multiple entities or components.
#[derive(Clone, Debug, Deref, DerefMut)]
pub struct ComponentMetadataMap {
    pub map: HashMap<ComponentId, ComponentTypeMetadata>,
}

impl ComponentMetadataMap {
    /// Creates a new [`ComponentMetadataMap`] by generating metadata for all registered component types in the world.
    ///
    /// This can be an expensive operation, so it is recommended to cache the resulting map.
    ///
    /// This method is used in the [`FromWorld`] implementation for [`ComponentMetadataMap`].
    pub fn generate(world: &World) -> Self {
        let mut map = HashMap::new();

        for component_info in world.components().iter_registered() {
            let component_id = component_info.id();
            if let Ok(metadata) = ComponentTypeMetadata::new(world, component_id) {
                map.insert(component_id, metadata);
            }
        }

        Self { map }
    }

    /// Creates an empty [`ComponentMetadataMap`].
    ///
    /// This can be useful when you want to start with an empty map,
    /// and only populate it with specific component metadata as needed.
    pub const fn empty() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Creates a new [`ComponentMetadataMap`] with data for the components of the specified entity.
    ///
    /// Be wary of time-of-check to time-of-use issues if the entity's components change after this is called.
    pub fn for_entity(world: &World, entity: Entity) -> Self {
        let mut map = HashMap::new();
        if let Ok(entity_ref) = world.get_entity(entity) {
            for component_id in entity_ref.archetype().components() {
                if let Ok(metadata) = ComponentTypeMetadata::new(world, *component_id) {
                    map.insert(*component_id, metadata);
                }
            }
        }
        Self { map }
    }

    /// Updates the metadata, scanning for any new component types that do not yet have metadata entries.
    ///
    /// Existing entries will not be modified.
    pub fn update(&mut self, world: &World) {
        for component_info in world.components().iter_registered() {
            let component_id = component_info.id();
            if !self.map.contains_key(&component_id)
                && let Ok(metadata) = ComponentTypeMetadata::new(world, component_id)
            {
                self.map.insert(component_id, metadata);
            }
        }
    }
}

impl FromWorld for ComponentMetadataMap {
    fn from_world(world: &mut World) -> Self {
        ComponentMetadataMap::generate(world)
    }
}

/// An error that can occur when attempting to inspect a component.
#[derive(Debug, Error)]
pub enum ComponentInspectionError {
    /// The component was not found on the entity.
    #[error("Component with ComponentId {0:?} not found on entity")]
    ComponentNotFound(ComponentId),
    /// The component type was not registered in the world.
    #[error("Component type {0} not registered in world")]
    ComponentNotRegistered(&'static str),
    /// The component ID provided was not registered in the world.
    #[error("ComponentId {0:?} not registered in world")]
    ComponentIdNotRegistered(ComponentId),
}

/// Settings for inspecting a component.
#[derive(Clone, Copy, Debug)]
pub struct ComponentInspectionSettings {
    /// How much detail to include when inspecting component values.
    ///
    /// Defaults to [`ComponentDetailLevel::Values`].
    pub detail_level: ComponentDetailLevel,
    /// Should full type names be used when displaying component values?
    ///
    /// Defaults to `false`.
    pub full_type_names: bool,
}

/// The amount of component information to include when inspecting an entity.
///
/// This impacts the values held in the `components` field of [`EntityInspection`](crate::entity_inspection::EntityInspection),
/// or inside of [`ComponentInspection`] if inspecting a single component.
///
/// Gathering full component values can be expensive,
/// so this setting allows users to limit the amount of information gathered
/// when inspecting many entities or components at once.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ComponentDetailLevel {
    /// Only component type names are provided.
    Names,
    /// Full component information, including values, is provided.
    #[default]
    Values,
}

impl Default for ComponentInspectionSettings {
    fn default() -> Self {
        Self {
            detail_level: ComponentDetailLevel::Values,
            full_type_names: false,
        }
    }
}
