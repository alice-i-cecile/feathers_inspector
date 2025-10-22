//! Types and traits for inspecting Bevy entities.

use bevy::prelude::*;
use core::fmt::Display;

/// The result of inspecting an entity.
pub struct EntityInspection {
    /// The entity being inspected.
    pub entity: Entity,
    /// The name of the entity, if any.
    pub name: Option<Name>,
}

impl Display for EntityInspection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Entity: {:?}", self.entity)?;
        let name_str = match &self.name {
            Some(name) => name.as_str(),
            None => "Entity",
        };

        write!(f, "{name_str} ({})", self.entity)?;

        Ok(())
    }
}

/// An extension trait for inspecting entities.
///
/// This is required because this crate is not part of Bevy itself.
pub trait InspectExtensionTrait {
    /// Inspects the provided entity.
    ///
    /// The provided [`EntityInspection`] contains details about the entity,
    /// and can be logged using the [`Display`] trait.
    fn inspect(&self) -> EntityInspection;
}

impl InspectExtensionTrait for EntityRef<'_> {
    fn inspect(&self) -> EntityInspection {
        let name = self.get::<Name>().cloned();
        EntityInspection {
            entity: self.id(),
            name,
        }
    }
}

/// An extension trait for inspecting entities via Commands.
pub trait InspectExtensionCommandsTrait {
    /// Inspects the provided entity, logging details to the console using [`info!`].
    fn inspect(&mut self);
}

impl InspectExtensionCommandsTrait for EntityCommands<'_> {
    fn inspect(&mut self) {
        self.queue(|entity_world_mut: EntityWorldMut| {
            let entity_ref: EntityRef<'_> = entity_world_mut.into();
            let inspection = entity_ref.inspect();
            info!("{}", inspection);
        });
    }
}
