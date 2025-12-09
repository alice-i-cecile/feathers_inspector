//! Inspector UI widgets.
//!
//! Provides editable value widgets for the inspector, including:
//! - DragValue: A draggable number input (like ImGui's DragFloat)
//!   - Drag horizontally to change value
//!   - Double-click to enter text input mode

pub mod drag_value;

pub use drag_value::{
    apply_pending_value_changes, DragValue, DragValueChanged, DragValueDragState,
    DragValueEditModeChanged, DragValuePlugin, DragValueProps, FieldPath, FieldPathSegment,
    PendingValueChanges,
};
