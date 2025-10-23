//! Pretty printing for type registrations.
//!
//! These utilities are primarily intended for inspecting and debugging purposes.
//!
//! This cannot implement the [`Display`] trait directly on [`TypeRegistration`]
//! and its contained types because those types are defined in Bevy itself.

use bevy::prelude::*;
use bevy::reflect::{
    ArrayInfo, EnumInfo, ListInfo, MapInfo, OpaqueInfo, SetInfo, StructInfo, TupleInfo,
    TupleStructInfo, TypeInfo, TypeRegistration,
};

/// I can't believe it's not [`Display`]!
pub trait PrettyPrint {
    fn print(&self) -> String;
}

impl PrettyPrint for TypeRegistration {
    fn print(&self) -> String {
        let mut output = String::new();

        output.push_str(&self.type_info().print());

        output
    }
}

impl PrettyPrint for TypeInfo {
    fn print(&self) -> String {
        let output = match self {
            TypeInfo::Array(array_info) => array_info.print(),
            TypeInfo::Enum(enum_info) => enum_info.print(),
            TypeInfo::List(list_info) => list_info.print(),
            TypeInfo::Map(map_info) => map_info.print(),
            TypeInfo::Opaque(opaque_info) => opaque_info.print(),
            TypeInfo::Set(set_info) => set_info.print(),
            TypeInfo::Struct(struct_info) => struct_info.print(),
            TypeInfo::TupleStruct(tuple_struct_info) => tuple_struct_info.print(),
            TypeInfo::Tuple(tuple_info) => tuple_info.print(),
        };

        output
    }
}

impl PrettyPrint for ArrayInfo {
    fn print(&self) -> String {
        let capacity = self.capacity();
        let element_type_name = self.item_ty();

        format!(
            "Array of {} elements of type {}",
            capacity,
            element_type_name.short_path()
        )
    }
}

impl PrettyPrint for EnumInfo {
    fn print(&self) -> String {
        let mut output = String::new();

        output.push_str("Enum {\n");
        for variant_name in self.variant_names() {
            output.push_str(&format!("  Variant: {variant_name}\n"));
        }
        output.push_str("}");

        output
    }
}

impl PrettyPrint for ListInfo {
    fn print(&self) -> String {
        let element_type_name = self.item_ty();
        format!("List of {}", element_type_name.short_path())
    }
}

impl PrettyPrint for MapInfo {
    fn print(&self) -> String {
        let key_type_name = self.key_ty();
        let value_type_name = self.value_ty();

        format!(
            "Map of {} to {}",
            key_type_name.short_path(),
            value_type_name.short_path()
        )
    }
}

impl PrettyPrint for OpaqueInfo {
    fn print(&self) -> String {
        let type_name = self.ty();

        format!("Opaque type: {}", type_name.short_path())
    }
}

impl PrettyPrint for SetInfo {
    fn print(&self) -> String {
        let element_type_name = self.value_ty();

        format!("Set of {}", element_type_name.short_path())
    }
}

impl PrettyPrint for StructInfo {
    fn print(&self) -> String {
        let mut output = String::new();

        let type_name = self.ty();

        output.push_str(&format!("{} {{\n", type_name.short_path()));
        for field_name in self.field_names() {
            output.push_str(&format!("{},\n", field_name));
        }
        output.push_str("}");

        output
    }
}

impl PrettyPrint for TupleInfo {
    fn print(&self) -> String {
        let mut output = String::new();

        let type_name = self.ty();

        output.push_str(&format!("{} (\n", type_name.short_path()));

        for element in self.iter() {
            output.push_str(&format!("  {},\n", element.ty().short_path()));
        }
        output.push_str(")");

        output
    }
}

impl PrettyPrint for TupleStructInfo {
    fn print(&self) -> String {
        let mut output = String::new();

        let type_name = self.ty();
        output.push_str(&format!("{} (\n", type_name.short_path()));
        for element in self.iter() {
            output.push_str(&format!("  {},\n", element.ty().short_path()));
        }
        output.push_str(")");

        output
    }
}
