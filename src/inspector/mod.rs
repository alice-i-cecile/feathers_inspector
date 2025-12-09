//! Inspector UI module.
//!
//! Provides a separate window for inspecting entities, components, and relationships
//! in a Bevy application using bevy_ui and bevy_experimental_feathers.

pub mod config;
pub mod panels;
pub mod plugin;
pub mod semantic_names;
pub mod state;
pub mod widgets;

pub use config::InspectorConfig;
pub use plugin::{InspectorSet, InspectorWindow, InspectorWindowPlugin};
pub use semantic_names::SemanticFieldNames;
pub use state::{DetailTab, EntityListEntry, InspectorCache, InspectorState, InspectorWindowState};
pub use widgets::{DragValue, DragValueChanged, DragValuePlugin, FieldPath, FieldPathSegment};
