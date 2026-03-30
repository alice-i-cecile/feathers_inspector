//! Inspector UI widgets.
//!
//! Provides editable value widgets for the inspector, including:
//! - DragValue: A draggable number input (like ImGui's DragFloat)
//!   - Drag horizontally to change value
//!   - Double-click to enter text input mode

pub mod drag_value;
pub mod reflected;
pub mod registry;
pub mod tabs;

use bevy::ecs::entity::Entity;
use std::any::TypeId;
/// Describes how to locate a field within a component for write-back.
#[derive(Clone, Debug)]
pub struct FieldPath {
    /// The entity containing the component.
    pub entity: Entity,
    /// The TypeId of the component.
    pub component_type_id: TypeId,
    /// The path segments to navigate to the field.
    pub path: Vec<FieldPathSegment>,
}

/// A segment in a field path.
#[derive(Clone, Debug)]
pub enum FieldPathSegment {
    /// Named struct field: e.g., "translation"
    Named(String),
    /// Indexed tuple/array field: e.g., 0, 1, 2
    Index(usize),
}
