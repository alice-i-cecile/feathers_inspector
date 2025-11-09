//! Code that makes working with Bevy's reflection system easier.
//!
//! This should go into bevy_reflect or bevy_ecs::reflect eventually.

use bevy::{
    prelude::*,
    reflect::{Array, Enum, List, Map, ReflectRef, Set, Tuple, VariantType},
};
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

/// Gets a reflected mutable reference to a resource from the world.
// This should be a method on `World` once upstreamed.
pub fn get_reflected_resource_mut<'w>(
    world: &'w mut World,
    type_id: TypeId,
) -> Result<Mut<'w, dyn Reflect>, ReflectionFetchError> {
    let type_registry = world.resource::<AppTypeRegistry>();
    let type_registry_read_lock = type_registry.read();
    let Some(type_registration) = type_registry_read_lock.get(type_id) else {
        return Err(ReflectionFetchError::NotRegistered(type_id));
    };

    // We must explicitly drop the read lock in order to acquire a mutable borrow of the world.
    // To do this, we must clone the `TypeRegistration` that we need.
    let type_registration = type_registration.clone();
    drop(type_registry_read_lock);

    let Some(reflect_resource) = type_registration.data::<ReflectResource>() else {
        return Err(ReflectionFetchError::MissingReflectTrait(type_id));
    };

    let Ok(reflected) = reflect_resource.reflect_mut(world) else {
        return Err(ReflectionFetchError::ReflectionRetrievalFailed(type_id));
    };
    Ok(reflected)
}

/// Gets a reflected reference to a component from an entity in the world.
// This should be a method on `EntityRef` once upstreamed,
// and `World::get_reflect_mut` should be removed.
// We should be able to access the AppTypeRegistry from the EntityRef directly safely
// once upstreamed by using private world access tools.
pub fn get_reflected_component_ref(
    world: &World,
    entity: Entity,
    type_id: TypeId,
) -> Result<&dyn PartialReflect, ReflectionFetchError> {
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

/// Gets a reflected mutable reference to a component from an entity in the world.
// This should be a method on `EntityMut` once upstreamed,
// and `World::get_reflect_mut` should be removed.
pub fn get_reflected_component_mut<'w>(
    world: &'w mut World,
    entity: Entity,
    type_id: TypeId,
) -> Result<Mut<'w, dyn Reflect>, ReflectionFetchError> {
    let app_type_registry = world.resource::<AppTypeRegistry>();

    let type_registry_read_lock = app_type_registry.read();
    let Some(type_registration) = type_registry_read_lock.get(type_id) else {
        return Err(ReflectionFetchError::NotRegistered(type_id));
    };

    // We must explicitly drop the read lock in order to acquire a mutable borrow of the world.
    // To do this, we must clone the `TypeRegistration` that we need.
    let type_registration = type_registration.clone();
    drop(type_registry_read_lock);

    let Some(reflect_component) = type_registration.data::<ReflectComponent>() else {
        return Err(ReflectionFetchError::MissingReflectTrait(type_id));
    };

    let entity_mut = world.entity_mut(entity);
    let Some(reflected) = reflect_component.reflect_mut(entity_mut) else {
        return Err(ReflectionFetchError::ReflectionRetrievalFailed(type_id));
    };

    Ok(reflected)
}

/// Converts a reflected value to a string for debugging purposes.
// When upstreamed, this should be a method on `PartialReflect`,
// although much of it should be a `Display` impl on `ReflectRef`.
pub fn reflected_value_to_string(reflected: &dyn PartialReflect, full_type_names: bool) -> String {
    let reflect_ref = reflected.reflect_ref();
    match reflect_ref {
        ReflectRef::Struct(dyn_struct) => {
            pretty_print_reflected_struct(dyn_struct, full_type_names)
        }
        ReflectRef::TupleStruct(tuple_struct) => {
            pretty_print_reflected_tuple_struct(tuple_struct, full_type_names)
        }
        ReflectRef::Tuple(tuple) => pretty_print_reflected_tuple(tuple, full_type_names),
        ReflectRef::List(list) => pretty_print_reflected_list(list, full_type_names),
        ReflectRef::Array(array) => pretty_print_reflected_array(array, full_type_names),
        ReflectRef::Map(map) => pretty_print_reflected_map(map, full_type_names),
        ReflectRef::Set(set) => pretty_print_reflected_set(set, full_type_names),
        ReflectRef::Enum(dyn_enum) => pretty_print_reflected_enum(dyn_enum, full_type_names),
        ReflectRef::Opaque(opaque_partial_reflect) => {
            pretty_print_reflected_opaque(opaque_partial_reflect)
        }
    }
}

pub fn pretty_print_reflected_struct(dyn_struct: &dyn Struct, full_type_names: bool) -> String {
    let mut result = String::new();
    let represented_type_info = dyn_struct.get_represented_type_info();
    let type_name = match represented_type_info {
        Some(info) => info.type_path(),
        None => "<Unknown Struct>",
    };

    if full_type_names {
        result.push_str(&format!("{type_name} {{\n"));
    } else {
        let short_type_name = ShortName::from(type_name);
        result.push_str(&format!("{short_type_name} {{\n"));
    }

    for i in 0..dyn_struct.field_len() {
        let field_name = dyn_struct.name_at(i).unwrap_or("<Unknown Field>");
        let field_value = get_value_string(dyn_struct.field_at(i), full_type_names);
        result.push_str(&format!("  {field_name}: {field_value},\n"));
    }
    result.push('}');
    result
}

pub fn pretty_print_reflected_tuple_struct(
    dyn_tuple_struct: &dyn TupleStruct,
    full_type_names: bool,
) -> String {
    let mut result = String::new();
    let represented_type_info = dyn_tuple_struct.get_represented_type_info();
    let type_name = match represented_type_info {
        Some(info) => info.type_path(),
        None => "<Unknown TupleStruct>",
    };

    if full_type_names {
        result.push_str(&format!("{type_name}(\n"));
    } else {
        let short_type_name = ShortName::from(type_name);
        result.push_str(&format!("{short_type_name}(\n"));
    }

    for i in 0..dyn_tuple_struct.field_len() {
        let field_value = get_value_string(dyn_tuple_struct.field(i), full_type_names);
        result.push_str(&format!("  {field_value},\n"));
    }
    result.push(')');
    result
}

pub fn pretty_print_reflected_tuple(dyn_tuple: &dyn Tuple, full_type_names: bool) -> String {
    let mut result = String::new();
    result.push_str("(\n");

    for i in 0..dyn_tuple.field_len() {
        let field_value = get_value_string(dyn_tuple.field(i), full_type_names);
        result.push_str(&format!("  {field_value},\n"));
    }
    result.push(')');
    result
}

pub fn pretty_print_reflected_list(dyn_list: &dyn List, full_type_names: bool) -> String {
    let mut result = String::new();
    result.push_str("[\n");

    for i in 0..dyn_list.len() {
        let element = get_value_string(dyn_list.get(i), full_type_names);
        result.push_str(&format!("  {element},\n"));
    }
    result.push(']');
    result
}

pub fn pretty_print_reflected_array(dyn_array: &dyn Array, full_type_names: bool) -> String {
    let mut result = String::new();
    result.push_str("[\n");

    for i in 0..dyn_array.len() {
        let element = get_value_string(dyn_array.get(i), full_type_names);
        result.push_str(&format!("  {element},\n"));
    }
    result.push(']');
    result
}
pub fn pretty_print_reflected_map(dyn_map: &dyn Map, full_type_names: bool) -> String {
    let mut result = String::new();
    result.push_str("{\n");

    for (key, value) in dyn_map.iter() {
        let key = reflected_value_to_string(key, full_type_names);
        let value = reflected_value_to_string(value, full_type_names);
        result.push_str(&format!("  {key}: {value},\n"));
    }

    result.push('}');
    result
}

pub fn pretty_print_reflected_set(dyn_set: &dyn Set, full_type_names: bool) -> String {
    let mut result = String::new();
    result.push_str("{\n");

    for element in dyn_set.iter() {
        let element_str = reflected_value_to_string(element, full_type_names);
        result.push_str(&format!("  {element_str},\n"));
    }

    result.push('}');
    result
}

pub fn pretty_print_reflected_enum(dyn_enum: &dyn Enum, full_type_names: bool) -> String {
    let mut result = String::new();
    let type_name = match dyn_enum.get_represented_type_info() {
        Some(info) => info.type_path(),
        None => "<Unknown Enum>",
    };

    let variant = dyn_enum.variant_name();

    if full_type_names {
        result.push_str(&format!("{type_name}::{variant}"));
    } else {
        result.push_str(&format!(
            "{short_type_name}::{variant}",
            short_type_name = ShortName::from(type_name)
        ));
    }

    match dyn_enum.variant_type() {
        VariantType::Struct => {
            result.push_str(" {\n");
            for i in 0..dyn_enum.field_len() {
                let field_name = dyn_enum.name_at(i).unwrap_or("<Unknown Field>");
                let field_value = get_value_string(dyn_enum.field_at(i), full_type_names);
                result.push_str(&format!("  {field_name}: {field_value},\n"));
            }
            result.push('}');
        }
        VariantType::Tuple => {
            result.push_str("(\n");
            for i in 0..dyn_enum.field_len() {
                let field_value = get_value_string(dyn_enum.field_at(i), full_type_names);
                result.push_str(&format!("  {field_value},\n"));
            }
            result.push(')');
        }
        VariantType::Unit => (),
    }
    result
}

pub fn pretty_print_reflected_opaque(opaque_partial_reflect: &dyn PartialReflect) -> String {
    // Fallback to the debug representation for opaque types
    format!("{:?}", opaque_partial_reflect)
}

fn get_value_string(partial_reflect: Option<&dyn PartialReflect>, full_type_names: bool) -> String {
    if let Some(value) = partial_reflect {
        reflected_value_to_string(value, full_type_names)
    } else {
        String::from("<Unknown Value>")
    }
}
