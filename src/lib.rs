//! An experimental entity and world inspector for Bevy.
//!
//! Built using bevy_feathers, powered by bevy_reflect.
//!
//! ## Optional Cargo features
//!
//! - `serde`: Adds the `serde` crate
//!   and implements `Serialize` and `Deserialize` on relevant types.
//! - `remote`: Enables BRP server functionality.
//!   Adds the `serde_json` crate,
//!   and enables `serde` and `bevy/bevy_remote` features.

pub mod archetype_similarity_grouping;
#[cfg(feature = "remote")]
pub mod brp_methods;
pub mod component_inspection;
pub mod entity_grouping;
pub mod entity_inspection;
pub mod entity_name_resolution;
pub mod extension_methods;
pub mod fuzzy_name_mapping;
pub mod hierarchy_grouping;
pub mod inspectable;
pub mod memory_size;
pub mod reflection_tools;
pub mod resource_inspection;
pub mod summary;
