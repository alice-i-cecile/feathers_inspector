//! Types and traits for inspecting Bevy entities.

use bevy::{ecs::component::ComponentId, prelude::*, reflect::TypeRegistration};
use core::any::TypeId;
use core::fmt::Display;
use thiserror::Error;

/// The result of inspecting a component.
///
/// Log this using the [`Display`] trait to see details about the component.
/// [`Debug`] can also be used for more detailed but harder to-read output.
// PERF: much of this information is duplicated across all components of the same type.
// TypeRegistration is particularly heavy.
// Should we create a shared `ComponentRegistry` type to store this info once per type?
#[derive(Clone, Debug)]
pub struct ComponentInspection {
    /// The entity that owns the component.
    pub entity: Entity,
    /// The ComponentId of the component.
    pub component_id: ComponentId,
    /// The type name of the component.
    pub name: DebugName,
    /// The value of the component as a string.
    ///
    /// This information is gathered via reflection,
    /// and used for debugging purposes.
    pub value: Option<String>,
    /// Is this component "name-defining"?
    ///
    /// If so, it will be prioritized for [name resolution](crate::name_resolution).
    pub name_definition_priority: Option<i8>,
    /// The [`TypeId`] of the resource.
    ///
    /// Note that dynamic types will not have a [`TypeId`].
    pub type_id: Option<TypeId>,
    /// The type information of the resource.
    ///
    /// This contains metadata about the resource's type,
    /// such as its fields and methods,
    /// as well as any reflected traits it implements.
    ///
    /// Note: this may be `None` if the type is not reflected and registered in the type registry.
    /// Currently, generic types need to be manually registered,
    /// and dynamically-typed resources cannot be registered.
    pub type_registration: Option<TypeRegistration>,
}

impl Display for ComponentInspection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let shortname = self.name.shortname();

        match &self.value {
            Some(value) => write!(f, "{shortname}: {value}")?,
            None => write!(f, "{shortname}")?,
        }

        Ok(())
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
