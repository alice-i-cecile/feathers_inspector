//! Types and traits for inspecting Bevy resources.

use crate::memory_size::MemorySize;
use bevy::reflect::TypeRegistration;
use bevy::{ecs::component::ComponentId, prelude::*};
use core::any::TypeId;
use core::fmt::{Debug, Display};
use thiserror::Error;

/// The result of inspecting a resource.
///
/// Log this using the [`Display`] trait to see details about the resource.
/// [`Debug`] can also be used for more detailed but harder to-read output.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ResourceInspection {
    /// The [`ComponentId`] of the resource.
    #[cfg_attr(
        feature = "serde",
        serde(with = "crate::serde_conversions::component_id")
    )]
    pub component_id: ComponentId,
    /// The type name of the resource.
    #[cfg_attr(
        feature = "serde",
        serde(with = "crate::serde_conversions::debug_name")
    )]
    pub name: DebugName,
    /// The value of the resource as a string.
    ///
    /// This information is gathered via reflection,
    /// and used for debugging purposes.
    pub value: String,
    /// The [`TypeId`] of the resource.
    ///
    /// Note that dynamic types will not have a [`TypeId`].
    #[cfg_attr(feature = "serde", serde(skip))]
    pub type_id: Option<TypeId>,
    /// The size of the resource in memory.
    ///
    /// This is computed using [`core::mem::size_of_val`], and requires reflection of the resource value.
    pub memory_size: Option<MemorySize>,
    /// The type information of the resource.
    ///
    /// This contains metadata about the resource's type,
    /// such as its fields and methods,
    /// as well as any reflected traits it implements.
    ///
    /// If Bevy's `reflect_documentation` feature is enabled,
    /// this also contains documentation comments for the type and its members.
    ///
    /// Note: this may be `None` if the type is not reflected and registered in the type registry.
    /// Currently, generic types need to be manually registered,
    /// and dynamically-typed resources cannot be registered.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub type_registration: Option<TypeRegistration>,
}

impl Display for ResourceInspection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let short_name = self.name.shortname();
        match &self.memory_size {
            Some(size) => write!(f, "{} ({}): {}", short_name, size, self.value)?,
            None => write!(f, "{}: {}", short_name, self.value)?,
        }

        Ok(())
    }
}

/// An error that can occur when attempting to inspect a resource.
#[derive(Debug, Error)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ResourceInspectionError {
    /// The resource type was not registered in the world.
    #[error("Resource type {0} not registered in world")]
    ResourceNotRegistered(&'static str),
    /// The resource was not found in the world.
    #[error("Resource with ComponentId {0:?} not found in world")]
    #[cfg_attr(
        feature = "serde",
        serde(with = "crate::serde_conversions::component_id")
    )]
    ResourceNotFound(ComponentId),
}

/// Settings that can be used to customize resource inspection,
/// changing how [`ResourceInspection`] is generated and displayed.
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ResourceInspectionSettings {
    /// Whether or not full type names should be displayed.
    ///
    /// Defaults to `false`.
    pub full_type_names: bool,
}
