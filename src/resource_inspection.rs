//! Types and traits for inspecting Bevy resources.

use crate::display_type_registration::PrettyPrint;
use crate::reflection_tools::{get_reflected_resource_ref, reflected_value_to_string};
use bevy::reflect::TypeRegistration;
use bevy::{ecs::component::ComponentId, prelude::*};
use core::any::{TypeId, type_name};
use core::fmt::{Debug, Display};
use thiserror::Error;

/// The result of inspecting a resource.
///
/// Log this using the [`Display`] trait to see details about the resource.
/// [`Debug`] can also be used for more detailed but harder to-read output.
#[derive(Clone, Debug)]
pub struct ResourceInspection {
    /// The [`ComponentId`] of the resource.
    pub component_id: ComponentId,
    /// The type name of the resource.
    pub name: DebugName,
    /// The value of the resource as a string.
    ///
    /// This information is gathered via reflection,
    /// and used for debugging purposes.
    pub value: String,
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

impl Display for ResourceInspection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let short_name = self.name.shortname();
        write!(f, "{short_name}")?;

        if let Some(type_registration) = &self.type_registration {
            let type_info_str = type_registration.print();
            write!(f, "\nType Information:\n{}", type_info_str)?;
        } else {
            write!(f, "\nType Information: <unregistered type>")?;
        }

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

    /// Inspects all resources in the world.
    ///
    /// Returns a vector of [`ResourceInspection`]s for all resources found.
    fn inspect_all_resources(&self) -> Vec<ResourceInspection>;
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
        let type_id = component_info.type_id();

        let type_registry = self.resource::<AppTypeRegistry>();
        let type_registration = match type_id {
            Some(type_id) => type_registry.read().get(type_id).cloned(),
            None => None,
        };

        let maybe_reflected = match type_id {
            Some(type_id) => match get_reflected_resource_ref(self, type_id) {
                Ok(reflected) => Some(reflected),
                Err(_) => None,
            },
            None => None,
        };

        let value = if let Some(reflected) = maybe_reflected {
            reflected_value_to_string(reflected)
        } else {
            "<unreflectable>".to_string()
        };

        Ok(ResourceInspection {
            component_id,
            name,
            value,
            type_id,
            type_registration,
        })
    }

    fn inspect_all_resources(&self) -> Vec<ResourceInspection> {
        let mut inspections = Vec::new();

        for (component_info, _ptr) in self.iter_resources() {
            let component_id = component_info.id();
            if let Ok(inspection) = self.inspect_resource_by_id(component_id) {
                inspections.push(inspection);
            }
        }

        inspections
    }
}

/// An extension trait for inspecting resources via Commands.
pub trait ResourceInspectExtensionCommandsTrait {
    /// Inspects the provided resource type, logging details to the console using [`info!`].
    fn inspect_resource<R: Resource>(&mut self);

    /// Inspects all resources in the world, logging details to the console using [`info!`].
    fn inspect_all_resources(&mut self);
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

    fn inspect_all_resources(&mut self) {
        self.queue(|world: &mut World| {
            let mut inspections = world.inspect_all_resources();
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
