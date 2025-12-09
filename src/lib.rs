//! An experimental entity and world inspector for Bevy.
//!
//! Built using bevy_feathers, powered by bevy_reflect.

pub mod archetype_similarity_grouping;
pub mod component_inspection;
pub mod entity_grouping;
pub mod entity_inspection;
pub mod entity_name_resolution;
pub mod extension_methods;
pub mod fuzzy_name_mapping;
pub mod hierarchy_grouping;
pub mod inspectable;
pub mod inspector;
pub mod memory_size;
pub mod reflection_tools;
pub mod resource_inspection;

// Re-export the main plugin for convenience
pub use inspector::{InspectorConfig, InspectorWindowPlugin};
pub mod summary;
