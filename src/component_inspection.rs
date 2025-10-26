//! Types and traits for inspecting Bevy entities.

use bevy::{ecs::component::ComponentId, prelude::*, reflect::TypeRegistration};
use core::any::TypeId;
use core::fmt::Display;
use thiserror::Error;

use crate::display_type_registration::PrettyPrint;

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
    pub value: String,
    /// Is this component "name-defining"?
    ///
    /// If so, it will be prioritized for [name resolution](crate::name_resolution).
    pub is_name_defining: bool,
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
        write!(f, "{}", self.name.shortname())?;

        if let Some(type_registration) = &self.type_registration {
            let type_info_str = type_registration.print();
            write!(f, "\n{}", type_info_str)?;
        } else {
            write!(f, "\n<unregistered type>")?;
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
