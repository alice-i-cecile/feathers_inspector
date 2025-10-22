//! Types and traits for inspecting Bevy entities.
//!
//! Entities are composed of components, but this module focuses on
//! inspecting the entity as a whole.
//!
//! See the [`component_inspection`](crate::component_inspection) module
//! for information about inspecting and displaying components.

use bevy::{ecs::component::ComponentId, prelude::*};
use core::any::type_name;
use core::fmt::Display;

use crate::component_inspection::{ComponentInspection, ComponentInspectionError};

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
///
/// Ideally these methods would be on `EntityRef` and friends,
/// but accessing metadata about components requires access to the `World`,
/// which is not available, especially externally.
pub trait EntityInspectExtensionTrait {
    /// Inspects the provided entity.
    ///
    /// The provided [`EntityInspection`] contains details about the entity,
    /// and can be logged using the [`Display`] trait.
    fn inspect(&self, entity: Entity) -> EntityInspection;

    /// Inspects the component corresponding to the provided [`ComponentId`].
    ///
    /// The provided [`ComponentInspection`] contains details about the component,
    /// and can be logged using the [`Display`] trait.
    ///
    /// If you only want to inspect a specific component type, consider using
    /// [`inspect_component::<C>`](Self::inspect_component) instead.
    fn inspect_component_by_id(
        &self,
        component_id: ComponentId,
        entity: Entity,
    ) -> Result<ComponentInspection, ComponentInspectionError>;

    /// Inspects the component of type `C`.
    ///
    /// The provided [`ComponentInspection`] contains details about the component,
    /// and can be logged using the [`Display`] trait.
    fn inspect_component<C: Component>(
        &self,
        entity: Entity,
    ) -> Result<ComponentInspection, ComponentInspectionError>;
}

impl EntityInspectExtensionTrait for World {
    fn inspect(&self, entity: Entity) -> EntityInspection {
        let name = self.get::<Name>(entity).cloned();
        EntityInspection { entity, name }
    }

    fn inspect_component_by_id(
        &self,
        component_id: ComponentId,
        entity: Entity,
    ) -> Result<ComponentInspection, ComponentInspectionError> {
        let component_info = self.components().get_info(component_id).ok_or(
            ComponentInspectionError::ComponentNotRegistered(type_name::<ComponentId>()),
        )?;

        let name = component_info.name();

        Ok(ComponentInspection {
            entity,
            component_id,
            name,
        })
    }

    fn inspect_component<C: Component>(
        &self,
        entity: Entity,
    ) -> Result<ComponentInspection, ComponentInspectionError> {
        let component_id = self.components().valid_component_id::<C>().ok_or(
            ComponentInspectionError::ComponentNotRegistered(type_name::<C>()),
        )?;
        self.inspect_component_by_id(component_id, entity)
    }
}

/// An extension trait for inspecting entities via Commands.
pub trait InspectExtensionCommandsTrait {
    /// Inspects the provided entity, logging details to the console using [`info!`].
    ///
    /// To inspect only a specific component on the entity, use
    /// [`inspect_component::<C>`](Self::inspect_component) instead.
    fn inspect(&mut self);

    /// Inspects the component of type `C` on the entity, logging details to the console using [`info!`].
    ///
    /// To inspect all components on the entity, use [`inspect`](Self::inspect) instead.
    fn inspect_component<C: Component>(&mut self) {}
}

impl InspectExtensionCommandsTrait for EntityCommands<'_> {
    fn inspect(&mut self) {
        let entity = self.id();

        self.queue(move |entity_world_mut: EntityWorldMut| {
            let world = entity_world_mut.world();
            let inspection = world.inspect(entity);
            info!("{}", inspection);
        });
    }

    fn inspect_component<C: Component>(&mut self) {
        let entity = self.id();

        self.queue(move |entity_world_mut: EntityWorldMut| {
            let world = entity_world_mut.world();
            match world.inspect_component::<C>(entity) {
                Ok(inspection) => info!("{}", inspection),
                Err(err) => info!("Failed to inspect component: {}", err),
            }
        });
    }
}
