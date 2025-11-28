//! Drag value widget - a draggable number input like ImGui's DragFloat.
//!
//! This widget allows editing numeric values by:
//! 1. Horizontal dragging to increment/decrement the value
//! 2. Double-clicking to enter text input mode for direct value entry

use bevy::ecs::entity::Entity;
use bevy::ecs::event::Event;
use bevy::ecs::observer::On;
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input_focus::{FocusedInput, InputFocus};
use bevy::picking::events::{Click, Drag, DragEnd, DragStart, Pointer};
use bevy::prelude::*;
use bevy::reflect::ReflectMut;
use core::any::TypeId;
use std::time::{Duration, Instant};

use crate::reflection_tools::get_reflected_component_mut;

/// Double-click detection threshold (in milliseconds)
const DOUBLE_CLICK_THRESHOLD_MS: u64 = 300;

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

/// Props for spawning a DragValue widget.
pub struct DragValueProps {
    /// The field path for write-back.
    pub field_path: FieldPath,
    /// Current value.
    pub value: f64,
    /// How fast dragging changes the value.
    pub drag_speed: f64,
    /// Precision (decimal places for display).
    pub precision: usize,
    /// Minimum value (optional).
    pub min: Option<f64>,
    /// Maximum value (optional).
    pub max: Option<f64>,
}

impl Default for DragValueProps {
    fn default() -> Self {
        Self {
            field_path: FieldPath {
                entity: Entity::PLACEHOLDER,
                component_type_id: TypeId::of::<()>(),
                path: vec![],
            },
            value: 0.0,
            drag_speed: 0.1,
            precision: 2,
            min: None,
            max: None,
        }
    }
}

/// Marker component for a drag value widget.
/// Contains the field path for write-back and configuration.
#[derive(Component, Clone)]
#[require(DragValueDragState)]
pub struct DragValue {
    /// The field path for identifying which field to update.
    pub field_path: FieldPath,
    /// How fast dragging changes the value (units per pixel).
    pub drag_speed: f64,
    /// Number of decimal places for display.
    pub precision: usize,
    /// Minimum allowed value.
    pub min: Option<f64>,
    /// Maximum allowed value.
    pub max: Option<f64>,
}

/// Tracks the drag state of a DragValue widget.
#[derive(Component)]
pub struct DragValueDragState {
    /// Whether currently dragging.
    pub dragging: bool,
    /// The value when dragging started.
    pub start_value: f64,
    /// Whether in text editing mode.
    pub editing: bool,
    /// Text buffer for editing mode.
    pub edit_buffer: String,
    /// Last click time for double-click detection.
    pub last_click_time: Option<Instant>,
    /// The original value before editing (for cancellation).
    pub original_value: f64,
}

impl Default for DragValueDragState {
    fn default() -> Self {
        Self {
            dragging: false,
            start_value: 0.0,
            editing: false,
            edit_buffer: String::new(),
            last_click_time: None,
            original_value: 0.0,
        }
    }
}

/// Event emitted when a DragValue changes.
/// Contains the field path and new value for write-back.
#[derive(Event, Clone, Debug)]
pub struct DragValueChanged {
    /// The UI entity that triggered this change.
    pub source: Entity,
    /// The field path for write-back.
    pub field_path: FieldPath,
    /// The new value.
    pub new_value: f64,
}

// Observer: handle click for double-click detection
fn drag_value_on_click(
    mut click: On<Pointer<Click>>,
    mut q_drag_value: Query<(&DragValue, &mut DragValueDragState, &Children)>,
    q_text: Query<&Text>,
    mut input_focus: ResMut<InputFocus>,
    mut commands: Commands,
) {
    if let Ok((drag_value, mut drag_state, children)) = q_drag_value.get_mut(click.entity) {
        click.propagate(false);

        let now = Instant::now();

        // Check for double-click
        let is_double_click = drag_state
            .last_click_time
            .map(|last| now.duration_since(last) < Duration::from_millis(DOUBLE_CLICK_THRESHOLD_MS))
            .unwrap_or(false);

        if is_double_click && !drag_state.editing {
            // Enter edit mode
            drag_state.editing = true;

            // Get current value and populate edit buffer
            let current_value = children
                .iter()
                .find_map(|child| {
                    q_text
                        .get(child)
                        .ok()
                        .and_then(|text| text.0.parse::<f64>().ok())
                })
                .unwrap_or(0.0);

            drag_state.original_value = current_value;
            drag_state.edit_buffer =
                format!("{:.prec$}", current_value, prec = drag_value.precision);

            // Set input focus to this widget
            input_focus.set(click.entity);

            // Trigger a visual update to show the edit buffer (with cursor indicator)
            commands.trigger(DragValueEditModeChanged {
                entity: click.entity,
                editing: true,
            });

            drag_state.last_click_time = None; // Reset to prevent triple-click
        } else {
            drag_state.last_click_time = Some(now);
        }
    }
}

/// Event emitted when edit mode changes
#[derive(Event, Clone, Debug)]
pub struct DragValueEditModeChanged {
    pub entity: Entity,
    pub editing: bool,
}

// Observer: handle drag start (skip if in edit mode)
fn drag_value_on_drag_start(
    mut drag_start: On<Pointer<DragStart>>,
    mut q_drag_value: Query<(&DragValue, &mut DragValueDragState, &Children)>,
    q_text: Query<&Text>,
) {
    if let Ok((_drag_value, mut drag_state, children)) = q_drag_value.get_mut(drag_start.entity) {
        // Skip dragging if in edit mode
        if drag_state.editing {
            return;
        }

        drag_start.propagate(false);

        // Get current value from Text child
        let current_value = children
            .iter()
            .find_map(|child| {
                q_text
                    .get(child)
                    .ok()
                    .and_then(|text| text.0.parse::<f64>().ok())
            })
            .unwrap_or(0.0);

        drag_state.dragging = true;
        drag_state.start_value = current_value;
    }
}

// Observer: handle drag
fn drag_value_on_drag(
    mut drag: On<Pointer<Drag>>,
    q_drag_value: Query<(&DragValue, &DragValueDragState)>,
    mut commands: Commands,
) {
    if let Ok((drag_value, drag_state)) = q_drag_value.get(drag.entity) {
        drag.propagate(false);

        if drag_state.dragging {
            // Horizontal drag distance in pixels
            let delta_x = drag.distance.x as f64;

            // Calculate new value
            let delta_value = delta_x * drag_value.drag_speed;
            let mut new_value = drag_state.start_value + delta_value;

            // Apply constraints
            if let Some(min) = drag_value.min {
                new_value = new_value.max(min);
            }
            if let Some(max) = drag_value.max {
                new_value = new_value.min(max);
            }

            // Emit change event
            commands.trigger(DragValueChanged {
                source: drag.entity,
                field_path: drag_value.field_path.clone(),
                new_value,
            });
        }
    }
}

// Observer: handle drag end
fn drag_value_on_drag_end(
    mut drag_end: On<Pointer<DragEnd>>,
    mut q_drag_value: Query<&mut DragValueDragState>,
) {
    if let Ok(mut drag_state) = q_drag_value.get_mut(drag_end.entity) {
        drag_end.propagate(false);
        drag_state.dragging = false;
    }
}

// System: update Text display when DragValueChanged is triggered
fn update_drag_value_display(
    trigger: On<DragValueChanged>,
    q_drag_value: Query<(&DragValue, &Children)>,
    mut q_text: Query<&mut Text>,
) {
    if let Ok((drag_value, children)) = q_drag_value.get(trigger.source) {
        // Find and update the Text child
        for child in children.iter() {
            if let Ok(mut text) = q_text.get_mut(child) {
                text.0 = format!("{:.prec$}", trigger.new_value, prec = drag_value.precision);
            }
        }
    }
}

/// Navigates a field path and sets the value using reflection.
/// Returns true on success, false on failure.
fn set_field_value_recursive(
    reflected: &mut dyn PartialReflect,
    path: &[FieldPathSegment],
    new_value: f64,
) -> bool {
    if path.is_empty() {
        // At the target field - try to set the value
        return apply_value_to_partial_reflect(reflected, new_value);
    }

    let segment = &path[0];
    let remaining = &path[1..];

    match reflected.reflect_mut() {
        ReflectMut::Struct(s) => {
            if let FieldPathSegment::Named(name) = segment
                && let Some(field) = s.field_mut(name)
            {
                return set_field_value_recursive(field, remaining, new_value);
            }
        }
        ReflectMut::TupleStruct(ts) => {
            if let FieldPathSegment::Index(idx) = segment
                && let Some(field) = ts.field_mut(*idx)
            {
                return set_field_value_recursive(field, remaining, new_value);
            }
        }
        ReflectMut::Tuple(t) => {
            if let FieldPathSegment::Index(idx) = segment
                && let Some(field) = t.field_mut(*idx)
            {
                return set_field_value_recursive(field, remaining, new_value);
            }
        }

        _ => {}
    }

    false
}

/// Applies a numeric value to a reflected field.
fn apply_value_to_partial_reflect(reflected: &mut dyn PartialReflect, new_value: f64) -> bool {
    // Try to apply to f32
    if let Some(f32_val) = reflected.try_downcast_mut::<f32>() {
        *f32_val = new_value as f32;
        return true;
    }

    // Try to apply to f64
    if let Some(f64_val) = reflected.try_downcast_mut::<f64>() {
        *f64_val = new_value;
        return true;
    }

    // Try to apply to i32
    if let Some(i32_val) = reflected.try_downcast_mut::<i32>() {
        *i32_val = new_value as i32;
        return true;
    }

    // Try to apply to i64
    if let Some(i64_val) = reflected.try_downcast_mut::<i64>() {
        *i64_val = new_value as i64;
        return true;
    }

    // Try to apply to u32
    if let Some(u32_val) = reflected.try_downcast_mut::<u32>() {
        *u32_val = new_value.max(0.0) as u32;
        return true;
    }

    // Try to apply to u64
    if let Some(u64_val) = reflected.try_downcast_mut::<u64>() {
        *u64_val = new_value.max(0.0) as u64;
        return true;
    }

    false
}

/// Resource to queue value changes for the write-back system
#[derive(Resource, Default)]
pub struct PendingValueChanges {
    pub changes: Vec<DragValueChanged>,
}

/// Observer that queues value changes for later processing
fn queue_value_change(trigger: On<DragValueChanged>, mut pending: ResMut<PendingValueChanges>) {
    pending.changes.push(trigger.event().clone());
}

/// Observer: handle keyboard input during text edit mode
fn drag_value_on_keyboard_input(
    trigger: On<FocusedInput<KeyboardInput>>,
    mut q_drag_value: Query<(&DragValue, &mut DragValueDragState, &Children)>,
    mut q_text: Query<&mut Text>,
    mut input_focus: ResMut<InputFocus>,
    mut commands: Commands,
) {
    // Only process key presses
    if trigger.input.state != ButtonState::Pressed {
        return;
    }

    // Check if the focused entity is a DragValue in edit mode
    let entity = trigger.focused_entity;
    if let Ok((drag_value, mut drag_state, children)) = q_drag_value.get_mut(entity) {
        if !drag_state.editing {
            return;
        }

        match &trigger.input.logical_key {
            Key::Enter => {
                // Commit the value
                if let Ok(new_value) = drag_state.edit_buffer.parse::<f64>() {
                    // Apply min/max constraints
                    let mut constrained_value = new_value;
                    if let Some(min) = drag_value.min {
                        constrained_value = constrained_value.max(min);
                    }
                    if let Some(max) = drag_value.max {
                        constrained_value = constrained_value.min(max);
                    }

                    // Emit change event
                    commands.trigger(DragValueChanged {
                        source: entity,
                        field_path: drag_value.field_path.clone(),
                        new_value: constrained_value,
                    });
                }

                // Exit edit mode
                exit_edit_mode(&mut drag_state, &mut input_focus, entity, &mut commands);
            }
            Key::Escape => {
                // Revert to original value
                for child in children.iter() {
                    if let Ok(mut text) = q_text.get_mut(child) {
                        text.0 = format!(
                            "{:.prec$}",
                            drag_state.original_value,
                            prec = drag_value.precision
                        );
                    }
                }

                // Exit edit mode
                exit_edit_mode(&mut drag_state, &mut input_focus, entity, &mut commands);
            }
            Key::Backspace => {
                // Remove last character
                drag_state.edit_buffer.pop();
                update_edit_display(&drag_state.edit_buffer, children, &mut q_text);
            }
            Key::Character(c) => {
                // Only allow numeric characters, decimal point, and minus sign
                let valid = c.chars().all(|ch| {
                    ch.is_ascii_digit()
                        || ch == '.'
                        || ch == '-'
                        || ch == 'e'
                        || ch == 'E'
                        || ch == '+'
                });
                if valid {
                    drag_state.edit_buffer.push_str(c);
                    update_edit_display(&drag_state.edit_buffer, children, &mut q_text);
                }
            }
            _ => {}
        }
    }
}

/// Helper: exit edit mode
fn exit_edit_mode(
    drag_state: &mut DragValueDragState,
    input_focus: &mut ResMut<InputFocus>,
    entity: Entity,
    commands: &mut Commands,
) {
    drag_state.editing = false;
    drag_state.edit_buffer.clear();
    input_focus.clear();
    commands.trigger(DragValueEditModeChanged {
        entity,
        editing: false,
    });
}

/// Helper: update the text display during editing
fn update_edit_display(buffer: &str, children: &Children, q_text: &mut Query<&mut Text>) {
    for child in children.iter() {
        if let Ok(mut text) = q_text.get_mut(child) {
            // Show edit buffer with cursor indicator
            text.0 = format!("{}|", buffer);
        }
    }
}

/// Observer: handle edit mode visual changes
fn update_edit_mode_display(
    trigger: On<DragValueEditModeChanged>,
    q_drag_value: Query<(&DragValue, &DragValueDragState, &Children)>,
    mut q_text: Query<&mut Text>,
) {
    if let Ok((drag_value, drag_state, children)) = q_drag_value.get(trigger.entity) {
        for child in children.iter() {
            if let Ok(mut text) = q_text.get_mut(child) {
                if trigger.editing {
                    // Show edit buffer with cursor
                    text.0 = format!("{}|", drag_state.edit_buffer);
                } else {
                    // Show formatted value
                    if let Ok(val) = drag_state.edit_buffer.parse::<f64>() {
                        text.0 = format!("{:.prec$}", val, prec = drag_value.precision);
                    }
                }
            }
        }
    }
}

/// Exclusive system that writes queued value changes back to ECS components.
pub fn apply_pending_value_changes(world: &mut World) {
    // Take pending changes to avoid borrow issues
    let changes = {
        let mut pending = world.resource_mut::<PendingValueChanges>();
        std::mem::take(&mut pending.changes)
    };

    for change in changes {
        let field_path = &change.field_path;

        // Get mutable access to the component and apply the change
        if let Ok(mut reflected) =
            get_reflected_component_mut(world, field_path.entity, field_path.component_type_id)
        {
            let success = set_field_value_recursive(
                reflected.bypass_change_detection().as_partial_reflect_mut(),
                &field_path.path,
                change.new_value,
            );
            if !success {
                warn!(
                    "Failed to set field value at path {:?} for entity {:?}",
                    field_path.path, field_path.entity
                );
            }
        }
    }
}

/// Plugin that adds the DragValue widget observers.
pub struct DragValuePlugin;

impl Plugin for DragValuePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingValueChanges>()
            // Drag behavior
            .add_observer(drag_value_on_drag_start)
            .add_observer(drag_value_on_drag)
            .add_observer(drag_value_on_drag_end)
            // Click for double-click detection
            .add_observer(drag_value_on_click)
            // Keyboard input for text editing
            .add_observer(drag_value_on_keyboard_input)
            // Display updates
            .add_observer(update_drag_value_display)
            .add_observer(update_edit_mode_display)
            // Value change processing
            .add_observer(queue_value_change)
            .add_systems(Update, apply_pending_value_changes);
    }
}
