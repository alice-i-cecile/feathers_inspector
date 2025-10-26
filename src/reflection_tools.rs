//! Code that makes working with Bevy's reflection system easier.
//!
//! This should go into bevy_reflect or bevy_ecs::reflect eventually.

use bevy::prelude::*;
use core::any::TypeId;

use thiserror::Error;

/// An error that can occur when attempting to fetch reflected data from the ECS.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum ReflectionFetchError {
    /// The type is not registered in the type registry.
    #[error("Type {0:?} not registered in type registry")]
    NotRegistered(TypeId),
    /// The type does not implement the required reflection trait.
    ///
    /// If this is for a component, ensure it implements `ReflectComponent`.
    /// If this is for a resource, ensure it implements `ReflectResource`.
    #[error("Type {0:?} does not implement required reflection trait")]
    MissingReflectTrait(TypeId),
    /// The reflected data could not be retrieved from the world or entity.
    ///
    /// Ensure that the entity/resource exists and is accessible.
    #[error("Could not retrieve reflected data for type {0:?}")]
    ReflectionRetrievalFailed(TypeId),
}

/// Gets a reflected reference to a resource from the world.
// This should be a method on `World` once upstreamed.
pub fn get_reflected_resource_ref(
    world: &World,
    type_id: TypeId,
) -> Result<&dyn PartialReflect, ReflectionFetchError> {
    let type_registry = world.resource::<AppTypeRegistry>();
    let type_registry_read_lock = type_registry.read();
    let Some(type_registration) = type_registry_read_lock.get(type_id) else {
        // TODO: this error variant should return information about the type or component id in question
        return Err(ReflectionFetchError::NotRegistered(type_id));
    };

    let Some(reflect_resource) = type_registration.data::<ReflectResource>() else {
        // TODO: these should be distinct error variants
        return Err(ReflectionFetchError::MissingReflectTrait(type_id));
    };

    let Ok(reflected) = reflect_resource.reflect(world) else {
        return Err(ReflectionFetchError::ReflectionRetrievalFailed(type_id));
    };

    Ok(reflected)
}

/// Gets a reflected reference to a component from an entity in the world.
// This should be a method on `EntityRef` once upstreamed.
// We should be able to access the AppTypeRegistry from the EntityRef directly safely
// once upstreamed by using private world access tools.
pub fn get_reflected_component_ref<'a>(
    world: &'a World,
    entity: Entity,
    type_id: TypeId,
) -> Result<&'a dyn PartialReflect, ReflectionFetchError> {
    let app_type_registry = world.resource::<AppTypeRegistry>();
    let entity_ref = world.entity(entity);

    let type_registry_read_lock = app_type_registry.read();
    let Some(type_registration) = type_registry_read_lock.get(type_id) else {
        return Err(ReflectionFetchError::NotRegistered(type_id));
    };

    let Some(reflect_component) = type_registration.data::<ReflectComponent>() else {
        return Err(ReflectionFetchError::MissingReflectTrait(type_id));
    };

    let Some(reflected) = reflect_component.reflect(entity_ref) else {
        return Err(ReflectionFetchError::ReflectionRetrievalFailed(type_id));
    };

    Ok(reflected)
}

/// Converts a reflected value to a string for debugging purposes.
// When upstreamed, this should be a method on `PartialReflect`.
pub fn reflected_value_to_string(_reflected: &dyn PartialReflect) -> String {
    "Unimplemented".to_string()
}
