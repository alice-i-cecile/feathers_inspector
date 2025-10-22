//! Types and traits for inspecting Bevy resources.

use bevy::prelude::*;
use core::fmt::Display;

/// The result of inspecting a resource.
pub struct ResourceInspection {
    /// The type name of the resource.
    pub type_name: Option<&'static str>,
}

impl Display for ResourceInspection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_name = match &self.type_name {
            Some(name) => name,
            None => "Dynamically-Typed Resource",
        };

        write!(f, "{type_name}")?;

        Ok(())
    }
}

/// An extension trait for inspecting resources.
//////
/// This is required because this crate is not part of Bevy itself.
pub trait ResourceInspectExtensionTrait {
    /// Inspects the provided resource.
    ///
    /// The provided [`ResourceInspection`] contains details about the resource,
    /// and can be logged using the [`Display`] trait.
    fn inspect_resource<R: Resource>(&self) -> ResourceInspection;
}

impl ResourceInspectExtensionTrait for World {
    fn inspect_resource<R>(&self) -> ResourceInspection {
        let type_name = std::any::type_name::<R>();
        ResourceInspection {
            type_name: Some(type_name),
        }
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
            info!("{inspection}");
        });
    }
}
