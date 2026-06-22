//! Code that makes working with Bevy's reflection system easier.

use bevy::{
    prelude::*,
    reflect::{
        ReflectRef,
        array::Array,
        enums::{Enum, VariantType},
        list::List,
        map::Map,
        set::Set,
        tuple::Tuple,
    },
};

/// Checks if a reflected value is safe to be converted to a dynamic representation.
/// It recursively traverses the object and checks if Map keys and Set values
/// implement `reflect_hash` and `reflect_partial_eq`.
pub fn is_dynamic_safe(val: &dyn PartialReflect) -> bool {
    match val.reflect_ref() {
        ReflectRef::Struct(s) => {
            for i in 0..s.field_len() {
                if let Some(field) = s.field_at(i)
                    && !is_dynamic_safe(field)
                {
                    return false;
                }
            }
            true
        }
        ReflectRef::TupleStruct(s) => {
            for i in 0..s.field_len() {
                if let Some(field) = s.field(i)
                    && !is_dynamic_safe(field)
                {
                    return false;
                }
            }
            true
        }
        ReflectRef::Tuple(s) => {
            for i in 0..s.field_len() {
                if let Some(field) = s.field(i)
                    && !is_dynamic_safe(field)
                {
                    return false;
                }
            }
            true
        }
        ReflectRef::List(s) => {
            for i in 0..s.len() {
                if let Some(field) = s.get(i)
                    && !is_dynamic_safe(field)
                {
                    return false;
                }
            }
            true
        }
        ReflectRef::Array(s) => {
            for i in 0..s.len() {
                if let Some(field) = s.get(i)
                    && !is_dynamic_safe(field)
                {
                    return false;
                }
            }
            true
        }
        ReflectRef::Map(s) => {
            for (k, v) in s.iter() {
                if k.reflect_hash().is_none() {
                    return false;
                }
                if k.reflect_partial_eq(k).is_none() {
                    return false;
                }
                if !is_dynamic_safe(k) {
                    return false;
                }
                if !is_dynamic_safe(v) {
                    return false;
                }
            }
            true
        }
        ReflectRef::Set(s) => {
            for v in s.iter() {
                if v.reflect_hash().is_none() {
                    return false;
                }
                if v.reflect_partial_eq(v).is_none() {
                    return false;
                }
                if !is_dynamic_safe(v) {
                    return false;
                }
            }
            true
        }
        ReflectRef::Enum(s) => {
            for i in 0..s.field_len() {
                if let Some(field) = s.field_at(i)
                    && !is_dynamic_safe(field)
                {
                    return false;
                }
            }
            true
        }
        ReflectRef::Opaque(_) => true,
    }
}

/// Safely clones a [`PartialReflect`] value into a boxed dynamic representation.
///
/// This handles the distinction between "dynamic-safe" types (which can use `.to_dynamic()`)
/// and opaque/unsafe types (which must use `.reflect_clone()`).
pub fn clone_partial_reflect(reflected: &dyn PartialReflect) -> Option<Box<dyn PartialReflect>> {
    if is_dynamic_safe(reflected) {
        match reflected.reflect_ref() {
            bevy::reflect::ReflectRef::Opaque(value) => value
                .reflect_clone()
                .ok()
                .map(|boxed| boxed.into_partial_reflect()),
            _ => Some(reflected.to_dynamic()),
        }
    } else {
        reflected
            .reflect_clone()
            .ok()
            .map(|boxed| boxed.into_partial_reflect())
    }
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
