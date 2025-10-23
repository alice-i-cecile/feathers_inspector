//! Types and traits for inspecting Bevy entities.

use bevy::{ecs::component::ComponentId, prelude::*};
use core::fmt::Display;
use thiserror::Error;

/// The result of inspecting a component.
pub struct ComponentInspection {
    /// The entity that owns the component.
    pub entity: Entity,
    /// The ComponentId of the component.
    pub component_id: ComponentId,
    /// The type name of the component.
    pub name: DebugName,
    /// How is this component involved in [name resolution](crate::name_resolution)?
    ///
    /// If [`Some`], the name of this component can be used to provide a friendly name in an inspector for this entity.
    /// The higher the value, the higher the priority.
    /// This is determined by the [`NameDefining`](crate::name_resolution::NameDefining)
    /// trait implementation for the component type.
    pub is_name_defining: Option<i32>,
}

impl Display for ComponentInspection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.shortname())
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
