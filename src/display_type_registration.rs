//! Pretty printing for type registrations.
//!
//! These utilities are primarily intended for inspecting and debugging purposes.
//!
//! This cannot implement the [`Display`] trait directly on [`TypeRegistration`]
//! and its contained types because those types are defined in Bevy itself.

use bevy::prelude::*;
use bevy::reflect::{TypeInfo, TypeRegistration};

pub fn pretty_print_type_registration(type_registration: &TypeRegistration) -> String {
    let mut output = String::new();

    output.push_str(&pretty_print_type_info(&type_registration.type_info()));

    output
}

pub fn pretty_print_type_info(type_info: &TypeInfo) -> String {
    let mut output = String::new();

    output.push_str(&format!("{:?}\n", type_info));

    output
}
