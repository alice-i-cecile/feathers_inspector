//! Types and traits for inspecting Bevy resources.

use bevy::{ecs::component::ComponentId, prelude::*};
use core::fmt::Display;
use std::any::type_name;
use thiserror::Error;

/// The result of inspecting a resource.
pub struct ResourceInspection {
    /// The type name of the resource.
    pub name: DebugName,
}

impl Display for ResourceInspection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Resource: {}", self.name)?;

        Ok(())
    }
}

/// An error that can occur when attempting to inspect a resource.
#[derive(Debug, Error)]
pub enum ResourceInspectionError {
    /// The resource type was not registered in the world.
    #[error("Resource type {0} not registered in world")]
    ResourceNotRegistered(&'static str),
    /// The resource was not found in the world.
    #[error("Resource with ComponentId {0:?} not found in world")]
    ResourceNotFound(ComponentId),
}

/// An extension trait for inspecting resources.
//////
/// This is required because this crate is not part of Bevy itself.
pub trait ResourceInspectExtensionTrait {
    /// Inspects the provided resource.
    ///
    /// The provided [`ResourceInspection`] contains details about the resource,
    /// and can be logged using the [`Display`] trait.
    fn inspect_resource<R: Resource>(&self) -> Result<ResourceInspection, ResourceInspectionError>;

    /// Inspects a resource by the provided [`ComponentId`].
    ///
    /// This is the dynamically-typed variant of [`inspect_resource`](Self::inspect_resource).
    fn inspect_resource_by_id(
        &self,
        component_id: ComponentId,
    ) -> Result<ResourceInspection, ResourceInspectionError>;
}

impl ResourceInspectExtensionTrait for World {
    fn inspect_resource<R: Resource>(&self) -> Result<ResourceInspection, ResourceInspectionError> {
        let component_id = self.components().resource_id::<R>().ok_or(
            ResourceInspectionError::ResourceNotRegistered(type_name::<R>()),
        )?;
        self.inspect_resource_by_id(component_id)
    }

    fn inspect_resource_by_id(
        &self,
        component_id: ComponentId,
    ) -> Result<ResourceInspection, ResourceInspectionError> {
        let component_info = self
            .components()
            .get_info(component_id)
            .ok_or(ResourceInspectionError::ResourceNotFound(component_id))?;

        let name = component_info.name();

        Ok(ResourceInspection { name })
    }
}

/// An extension trait for inspecting resources via Commands.
pub trait ResourceInspectExtensionCommandsTrait {
    /// Inspects the provided resource type, logging details to the console using [`info!`].
    fn inspect_resource<R: Resource>(&mut self);
}

impl ResourceInspectExtensionCommandsTrait for Commands<'_, '_> {
    fn inspect_resource<R: Resource>(&mut self) {
        self.queue(|world: &mut World| {
            let inspection = world.inspect_resource::<R>();
            match inspection {
                Ok(inspection) => info!("{inspection}"),
                Err(err) => warn!("Failed to inspect resource: {}", err),
            }
        });
    }
}
