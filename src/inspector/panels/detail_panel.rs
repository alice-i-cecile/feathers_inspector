//! Detail panel for the right side of the inspector.
//! Contains tabs for Components and Relationships.

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::ecs::observer::On;
use bevy::ecs::relationship::Relationship;
use bevy::feathers::controls::{button, ButtonProps};
use bevy::feathers::theme::ThemeBackgroundColor;
use bevy::feathers::tokens;
use bevy::prelude::*;
use bevy::reflect::{ReflectRef, VariantType};
use bevy::ui::Val::*;
use bevy::ui_widgets::{observe, Activate, ControlOrientation, CoreScrollbarThumb, Scrollbar};

use core::any::TypeId;

use crate::component_inspection::{
    ComponentDetailLevel, ComponentInspectionSettings, ComponentMetadataMap,
};
use crate::entity_inspection::{EntityInspectExtensionTrait, EntityInspectionSettings};
use crate::inspector::config::InspectorConfig;
use crate::inspector::semantic_names::SemanticFieldNames;
use crate::inspector::state::{DetailTab, InspectorCache, InspectorState};
use crate::inspector::widgets::{DragValue, DragValueDragState, FieldPath, FieldPathSegment};
use crate::reflection_tools::get_reflected_component_ref;

/// Marker component for the detail panel container.
#[derive(Component)]
pub struct DetailPanel;

/// Marker for tab buttons.
#[derive(Component)]
pub struct TabButton(pub DetailTab);

/// Marker for the scrollable detail content area.
#[derive(Component)]
pub struct DetailContent;

/// Marker for component cards.
#[derive(Component)]
pub struct ComponentCard;

/// Marker for hierarchy nodes (parent/child entities).
#[derive(Component)]
pub struct HierarchyNode(pub Entity);

/// Observer for tab button clicks.
fn on_tab_button_click(
    activate: On<Activate>,
    mut state: ResMut<InspectorState>,
    tabs: Query<&TabButton>,
) {
    if let Ok(tab) = tabs.get(activate.entity) {
        state.active_tab = tab.0;
    }
}

/// Observer for hierarchy node clicks (navigate to parent/child).
fn on_hierarchy_node_click(
    activate: On<Activate>,
    mut state: ResMut<InspectorState>,
    nodes: Query<&HierarchyNode>,
) {
    if let Ok(node) = nodes.get(activate.entity) {
        state.selected_entity = Some(node.0);
    }
}

/// Exclusive system that syncs the detail panel with the current selection.
/// Uses exclusive world access to avoid resource conflicts.
/// Only rebuilds UI when selection or tab changes.
pub fn sync_detail_panel(world: &mut World) {
    // Extract state info first and check for changes
    let state = world.resource::<InspectorState>();
    let selected_entity = state.selected_entity;
    let active_tab = state.active_tab;
    let previous_selection = state.previous_selection;
    let previous_tab = state.previous_tab;

    // Skip if nothing has changed
    let selection_changed = selected_entity != previous_selection;
    let tab_changed = active_tab != previous_tab;
    if !selection_changed && !tab_changed {
        return;
    }

    // Update previous values for next frame comparison
    {
        let mut state = world.resource_mut::<InspectorState>();
        state.previous_selection = selected_entity;
        state.previous_tab = active_tab;
    }

    // Find the detail content entity
    let mut query = world.query_filtered::<Entity, With<DetailContent>>();
    let content_entity = match query.iter(world).next() {
        Some(e) => e,
        None => return,
    };

    // Despawn all children of the content entity
    // Collect children first, then despawn them (despawning a parent also despawns children)
    let children_to_despawn: Vec<Entity> = world
        .get::<Children>(content_entity)
        .map(|c| c.iter().collect())
        .unwrap_or_default();

    for child in children_to_despawn {
        if world.entities().contains(child) {
            // Despawning a parent with ChildOf relationship automatically despawns descendants
            world.entity_mut(child).despawn();
        }
    }

    // Get config (clone values we need)
    let config = world.resource::<InspectorConfig>().clone();

    // Show empty state if no entity selected
    let Some(entity) = selected_entity else {
        spawn_empty_state_exclusive(world, content_entity, &config, "Select an entity to view details");
        return;
    };

    // Check if entity still exists
    if !world.entities().contains(entity) {
        spawn_error_state_exclusive(world, content_entity, &config, "Selected entity no longer exists");
        return;
    }

    // Ensure we have a metadata map - take it out to avoid borrow conflicts
    let mut metadata_map = world.resource_mut::<InspectorCache>().metadata_map.take();

    if metadata_map.is_none() {
        metadata_map = Some(ComponentMetadataMap::generate(world));
    }

    // Update metadata map
    if let Some(ref mut mm) = metadata_map {
        mm.update(world);
    }

    // Render based on active tab
    match active_tab {
        DetailTab::Components => {
            if let Some(ref mut mm) = metadata_map {
                spawn_components_tab_exclusive(world, content_entity, entity, mm, &config);
            }
        }
        DetailTab::Relationships => {
            if let Some(ref mm) = metadata_map {
                spawn_relationships_tab_exclusive(world, content_entity, entity, mm, &config);
            }
        }
    }

    // Put metadata_map back
    world.resource_mut::<InspectorCache>().metadata_map = metadata_map;
}

// ============================================================================
// Exclusive system helper functions (use World directly instead of Commands)
// ============================================================================

fn spawn_empty_state_exclusive(
    world: &mut World,
    parent: Entity,
    config: &InspectorConfig,
    message: &str,
) {
    let body_font_size = config.body_font_size;
    let muted_text_color = config.muted_text_color;
    let message = message.to_string();

    world.entity_mut(parent).with_children(|p| {
        p.spawn((
            Text::new(message),
            TextFont {
                font_size: body_font_size,
                ..default()
            },
            TextColor(muted_text_color),
            Node {
                padding: UiRect::all(Px(16.0)),
                ..default()
            },
        ));
    });
}

fn spawn_error_state_exclusive(
    world: &mut World,
    parent: Entity,
    config: &InspectorConfig,
    message: &str,
) {
    let body_font_size = config.body_font_size;
    let error_text_color = config.error_text_color;
    let message = message.to_string();

    world.entity_mut(parent).with_children(|p| {
        p.spawn((
            Text::new(message),
            TextFont {
                font_size: body_font_size,
                ..default()
            },
            TextColor(error_text_color),
            Node {
                padding: UiRect::all(Px(16.0)),
                ..default()
            },
        ));
    });
}

/// Represents a field extracted from a reflected component
struct ReflectedField {
    name: String,
    value: String,
    indent: u8,
    /// If this is an editable numeric field, contains the numeric value and path segments
    editable: Option<EditableFieldInfo>,
}

/// Information needed to make a field editable
struct EditableFieldInfo {
    /// The numeric value (as f64 for generality)
    numeric_value: f64,
    /// Path segments to reach this field from the component root
    path: Vec<FieldPathSegment>,
}

/// Extracts fields from a reflected value into a flat list of label/value pairs.
/// Uses `SemanticFieldNames` to provide better field names for tuple structs (e.g., x/y/z instead of .0/.1/.2).
/// Tracks the path to each field for write-back support.
fn extract_fields_from_reflect(
    reflected: &dyn PartialReflect,
    fields: &mut Vec<ReflectedField>,
    indent: u8,
    semantic_names: &SemanticFieldNames,
    current_path: &[FieldPathSegment],
) {
    // Get the TypeId of this reflected value for semantic name lookup
    let type_id = reflected
        .get_represented_type_info()
        .map(|info| info.type_id());

    match reflected.reflect_ref() {
        ReflectRef::Struct(s) => {
            for i in 0..s.field_len() {
                let field_name = s.name_at(i).unwrap_or("?");
                let field_value = s.field_at(i).unwrap();

                // Build path to this field
                let mut field_path = current_path.to_vec();
                field_path.push(FieldPathSegment::Named(field_name.to_string()));

                // Check if this is an editable numeric field
                let editable = try_extract_numeric(field_value).map(|num| EditableFieldInfo {
                    numeric_value: num,
                    path: field_path.clone(),
                });

                let value_str = format_simple_value(field_value);
                if let Some(val) = value_str {
                    fields.push(ReflectedField {
                        name: field_name.to_string(),
                        value: val,
                        indent,
                        editable,
                    });
                } else {
                    // Complex nested type - add header and recurse
                    let type_name = field_value
                        .get_represented_type_info()
                        .map(|t| ShortName::from(t.type_path()).to_string())
                        .unwrap_or_else(|| "?".to_string());
                    fields.push(ReflectedField {
                        name: field_name.to_string(),
                        value: format!("[{}]", type_name),
                        indent,
                        editable: None,
                    });
                    extract_fields_from_reflect(
                        field_value,
                        fields,
                        indent + 1,
                        semantic_names,
                        &field_path,
                    );
                }
            }
        }
        ReflectRef::TupleStruct(ts) => {
            for i in 0..ts.field_len() {
                let field_value = ts.field(i).unwrap();

                // Build path to this field (use Index for tuple structs)
                let mut field_path = current_path.to_vec();
                field_path.push(FieldPathSegment::Index(i));

                // Check if this is an editable numeric field
                let editable = try_extract_numeric(field_value).map(|num| EditableFieldInfo {
                    numeric_value: num,
                    path: field_path.clone(),
                });

                // Try to get semantic name (e.g., "x", "y", "z") for this field index
                let field_name = type_id
                    .and_then(|tid| semantic_names.get_field_name(tid, i))
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!(".{}", i));

                let value_str = format_simple_value(field_value);
                if let Some(val) = value_str {
                    fields.push(ReflectedField {
                        name: field_name,
                        value: val,
                        indent,
                        editable,
                    });
                } else {
                    let type_name = field_value
                        .get_represented_type_info()
                        .map(|t| ShortName::from(t.type_path()).to_string())
                        .unwrap_or_else(|| "?".to_string());
                    fields.push(ReflectedField {
                        name: field_name,
                        value: format!("[{}]", type_name),
                        indent,
                        editable: None,
                    });
                    extract_fields_from_reflect(
                        field_value,
                        fields,
                        indent + 1,
                        semantic_names,
                        &field_path,
                    );
                }
            }
        }
        ReflectRef::Enum(e) => {
            let variant_name = e.variant_name();
            match e.variant_type() {
                VariantType::Unit => {
                    fields.push(ReflectedField {
                        name: "variant".to_string(),
                        value: variant_name.to_string(),
                        indent,
                        editable: None,
                    });
                }
                VariantType::Tuple => {
                    fields.push(ReflectedField {
                        name: "variant".to_string(),
                        value: variant_name.to_string(),
                        indent,
                        editable: None,
                    });
                    for i in 0..e.field_len() {
                        let field_value = e.field_at(i).unwrap();
                        let value_str = format_simple_value(field_value);
                        if let Some(val) = value_str {
                            fields.push(ReflectedField {
                                name: format!(".{}", i),
                                value: val,
                                indent: indent + 1,
                                editable: None, // TODO: enum field editing
                            });
                        }
                    }
                }
                VariantType::Struct => {
                    fields.push(ReflectedField {
                        name: "variant".to_string(),
                        value: variant_name.to_string(),
                        indent,
                        editable: None,
                    });
                    for i in 0..e.field_len() {
                        let field_name = e.name_at(i).unwrap_or("?");
                        let field_value = e.field_at(i).unwrap();
                        let value_str = format_simple_value(field_value);
                        if let Some(val) = value_str {
                            fields.push(ReflectedField {
                                name: field_name.to_string(),
                                value: val,
                                indent: indent + 1,
                                editable: None, // TODO: enum field editing
                            });
                        }
                    }
                }
            }
        }
        _ => {
            // For other types (List, Map, etc), just show a simple representation
            if let Some(val) = format_simple_value(reflected) {
                fields.push(ReflectedField {
                    name: "value".to_string(),
                    value: val,
                    indent,
                    editable: None,
                });
            }
        }
    }
}

/// Tries to extract a numeric value from a reflected type.
/// Returns the value as f64 if it's a supported numeric type.
fn try_extract_numeric(reflected: &dyn PartialReflect) -> Option<f64> {
    // Try f32
    if let Some(val) = reflected.try_downcast_ref::<f32>() {
        return Some(*val as f64);
    }
    // Try f64
    if let Some(val) = reflected.try_downcast_ref::<f64>() {
        return Some(*val);
    }
    // Try i32
    if let Some(val) = reflected.try_downcast_ref::<i32>() {
        return Some(*val as f64);
    }
    // Try i64
    if let Some(val) = reflected.try_downcast_ref::<i64>() {
        return Some(*val as f64);
    }
    // Try u32
    if let Some(val) = reflected.try_downcast_ref::<u32>() {
        return Some(*val as f64);
    }
    // Try u64
    if let Some(val) = reflected.try_downcast_ref::<u64>() {
        return Some(*val as f64);
    }
    None
}

/// Tries to format a value as a simple string, returns None if it's a complex type
fn format_simple_value(reflected: &dyn PartialReflect) -> Option<String> {
    match reflected.reflect_ref() {
        ReflectRef::Struct(_) | ReflectRef::TupleStruct(_) | ReflectRef::Enum(_) => None,
        ReflectRef::Tuple(t) => {
            // Small tuples can be shown inline
            if t.field_len() <= 4 {
                let parts: Vec<String> = (0..t.field_len())
                    .filter_map(|i| format_simple_value(t.field(i).unwrap()))
                    .collect();
                if parts.len() == t.field_len() {
                    return Some(format!("({})", parts.join(", ")));
                }
            }
            None
        }
        ReflectRef::List(l) => Some(format!("[{} items]", l.len())),
        ReflectRef::Array(a) => Some(format!("[{} items]", a.len())),
        ReflectRef::Map(m) => Some(format!("{{{} entries}}", m.len())),
        ReflectRef::Set(s) => Some(format!("{{{} items}}", s.len())),
        ReflectRef::Opaque(o) => Some(format!("{:?}", o)),
    }
}

/// Data for a component card with extracted fields
struct ComponentCardData {
    name: String,
    size: String,
    fields: Vec<ReflectedField>,
    /// The entity this component belongs to (for write-back)
    entity: Entity,
    /// The TypeId of this component (for write-back)
    component_type_id: Option<TypeId>,
}

fn spawn_components_tab_exclusive(
    world: &mut World,
    parent: Entity,
    entity: Entity,
    metadata_map: &mut ComponentMetadataMap,
    config: &InspectorConfig,
) {
    let settings = EntityInspectionSettings {
        include_components: true,
        component_settings: ComponentInspectionSettings {
            detail_level: ComponentDetailLevel::Names, // We'll extract values ourselves
            full_type_names: false,
        },
    };

    let inspection_result = world.inspect_cached(entity, &settings, metadata_map);

    // Get semantic names resource for better tuple struct field names
    let semantic_names = world.resource::<SemanticFieldNames>();

    match inspection_result {
        Ok(inspection) => {
            // Resolve name using metadata map
            let resolved_name = inspection
                .resolve_name(&metadata_map.map)
                .unwrap_or_else(|| format!("Entity {:?}", entity));

            let component_count = inspection.components.as_ref().map(|c| c.len()).unwrap_or(0);
            let memory_display = inspection
                .total_memory_size
                .map(|m| m.to_string())
                .unwrap_or_else(|| "?".to_string());

            // Collect component IDs for reflection access
            let component_ids: Vec<_> = inspection
                .components
                .as_ref()
                .map(|c| c.iter().map(|comp| comp.component_id).collect())
                .unwrap_or_default();

            // Clone config values needed in closure
            let title_font_size = config.title_font_size;
            let body_font_size = config.body_font_size;
            let small_font_size = config.small_font_size;
            let panel_padding = config.panel_padding;
            let item_gap = config.item_gap;
            let border_color = config.border_color;
            let muted_text_color = config.muted_text_color;
            let field_name_color = Color::srgba(0.6, 0.8, 1.0, 1.0); // Light blue for field names

            // Extract fields for each component using reflection
            let mut component_cards: Vec<ComponentCardData> = Vec::new();

            for comp_id in &component_ids {
                // Get metadata for this component
                let meta = metadata_map.map.get(comp_id);
                let name = meta
                    .map(|m| m.name.shortname().to_string())
                    .unwrap_or_else(|| "?".to_string());
                let size = meta
                    .map(|m| m.memory_size.to_string())
                    .unwrap_or_else(|| "?".to_string());
                let component_type_id = meta.and_then(|m| m.type_id);

                // Try to get reflected component data
                let mut fields = Vec::new();
                if let Some(type_id) = component_type_id {
                    if let Ok(reflected) = get_reflected_component_ref(world, entity, type_id) {
                        extract_fields_from_reflect(reflected, &mut fields, 0, semantic_names, &[]);
                    }
                }

                component_cards.push(ComponentCardData {
                    name,
                    size,
                    fields,
                    entity,
                    component_type_id,
                });
            }

            world.entity_mut(parent).with_children(|p| {
                // Header with entity name and memory
                p.spawn((
                    Text::new(format!(
                        "{} | {} components | {}",
                        resolved_name, component_count, memory_display
                    )),
                    TextFont {
                        font_size: title_font_size,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Node {
                        margin: UiRect::bottom(Px(12.0)),
                        ..default()
                    },
                ));

                // Component cards
                for card_data in component_cards {
                    p.spawn((
                        Node {
                            width: Percent(100.0),
                            padding: panel_padding,
                            margin: UiRect::bottom(item_gap),
                            display: Display::Flex,
                            flex_direction: FlexDirection::Column,
                            border: UiRect::all(Px(1.0)),
                            ..default()
                        },
                        ThemeBackgroundColor(tokens::WINDOW_BG),
                        BorderColor::all(border_color),
                        ComponentCard,
                    ))
                    .with_children(|card| {
                        // Component name and size header
                        card.spawn((
                            Text::new(format!("{} | {}", card_data.name, card_data.size)),
                            TextFont {
                                font_size: body_font_size,
                                ..default()
                            },
                            TextColor(Color::srgba(0.9, 0.9, 0.9, 1.0)),
                            Node {
                                margin: UiRect::bottom(Px(4.0)),
                                ..default()
                            },
                        ));

                        // Field rows (dear imgui style)
                        for field in &card_data.fields {
                            let indent_px = field.indent as f32 * 12.0;

                            // Row container for label: value
                            card.spawn(Node {
                                display: Display::Flex,
                                flex_direction: FlexDirection::Row,
                                column_gap: Px(8.0),
                                margin: UiRect::left(Px(indent_px)),
                                align_items: AlignItems::Center,
                                ..default()
                            })
                            .with_children(|row| {
                                // Field name (light blue)
                                row.spawn((
                                    Text::new(format!("{}:", field.name)),
                                    TextFont {
                                        font_size: small_font_size,
                                        ..default()
                                    },
                                    TextColor(field_name_color),
                                ));

                                // Check if this field is editable
                                if let (Some(editable), Some(component_type_id)) =
                                    (&field.editable, card_data.component_type_id)
                                {
                                    // Spawn DragValue widget for editable numeric fields
                                    let field_path = FieldPath {
                                        entity: card_data.entity,
                                        component_type_id,
                                        path: editable.path.clone(),
                                    };

                                    row.spawn((
                                        Node {
                                            min_width: Px(60.0),
                                            padding: UiRect::horizontal(Px(4.0)),
                                            border: UiRect::all(Px(1.0)),
                                            ..default()
                                        },
                                        BorderColor::all(Color::srgba(0.3, 0.3, 0.3, 1.0)),
                                        BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 1.0)),
                                        DragValue {
                                            field_path,
                                            drag_speed: 0.1,
                                            precision: 2,
                                            min: None,
                                            max: None,
                                        },
                                        DragValueDragState::default(),
                                        Interaction::default(),
                                    ))
                                    .with_child((
                                        Text::new(format!("{:.2}", editable.numeric_value)),
                                        TextFont {
                                            font_size: small_font_size,
                                            ..default()
                                        },
                                        TextColor(Color::srgba(0.9, 0.9, 0.6, 1.0)), // Yellow for editable
                                    ));
                                } else {
                                    // Field value (muted) - non-editable
                                    row.spawn((
                                        Text::new(field.value.clone()),
                                        TextFont {
                                            font_size: small_font_size,
                                            ..default()
                                        },
                                        TextColor(muted_text_color),
                                    ));
                                }
                            });
                        }

                        // Show placeholder if no fields extracted
                        if card_data.fields.is_empty() {
                            card.spawn((
                                Text::new("<no reflected data>"),
                                TextFont {
                                    font_size: small_font_size,
                                    ..default()
                                },
                                TextColor(muted_text_color),
                            ));
                        }
                    });
                }
            });
        }
        Err(e) => {
            spawn_error_state_exclusive(world, parent, config, &format!("Error: {:?}", e));
        }
    }
}

fn spawn_relationships_tab_exclusive(
    world: &mut World,
    parent: Entity,
    entity: Entity,
    _metadata_map: &ComponentMetadataMap,
    config: &InspectorConfig,
) {
    // Get parent (ChildOf component)
    let parent_entity = world.get::<ChildOf>(entity).map(|c| c.get());

    // Get children
    let children: Vec<Entity> = world
        .get::<Children>(entity)
        .map(|c| c.iter().collect())
        .unwrap_or_default();

    // Collect hierarchy node data before mutable world access
    let parent_node_data = parent_entity.map(|e| {
        let name = world
            .get::<Name>(e)
            .map(|n| n.as_str().to_string())
            .unwrap_or_else(|| format!("Entity {:?}", e));
        let component_count = world
            .inspect(e, EntityInspectionSettings::default())
            .ok()
            .and_then(|i| i.components.as_ref().map(|c| c.len()))
            .unwrap_or(0);
        (e, name, component_count)
    });

    let children_node_data: Vec<(Entity, String, usize)> = children
        .iter()
        .map(|&e| {
            let name = world
                .get::<Name>(e)
                .map(|n| n.as_str().to_string())
                .unwrap_or_else(|| format!("Entity {:?}", e));
            let component_count = world
                .inspect(e, EntityInspectionSettings::default())
                .ok()
                .and_then(|i| i.components.as_ref().map(|c| c.len()))
                .unwrap_or(0);
            (e, name, component_count)
        })
        .collect();

    // Clone config values
    let title_font_size = config.title_font_size;
    let body_font_size = config.body_font_size;
    let muted_text_color = config.muted_text_color;
    let item_gap = config.item_gap;
    let children_len = children.len();

    world.entity_mut(parent).with_children(|p| {
        // Parent section
        p.spawn((
            Text::new("Parent"),
            TextFont {
                font_size: title_font_size,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::bottom(Px(8.0)),
                ..default()
            },
        ));

        if let Some((ent, name, comp_count)) = parent_node_data {
            let label = format!("{} ({} components)", name, comp_count);
            // Wrap button in container to handle margin (button() already includes Node)
            p.spawn(Node {
                margin: UiRect::bottom(item_gap),
                ..default()
            })
            .with_children(|wrapper| {
                wrapper.spawn((
                    button(
                        ButtonProps::default(),
                        HierarchyNode(ent),
                        bevy::prelude::Spawn((
                            Text::new(label),
                            TextFont {
                                font_size: body_font_size,
                                ..default()
                            },
                            TextColor(Color::srgba(0.9, 0.9, 0.9, 1.0)),
                        )),
                    ),
                    observe(on_hierarchy_node_click),
                ));
            });
        } else {
            p.spawn((
                Text::new("No parent (root entity)"),
                TextFont {
                    font_size: body_font_size,
                    ..default()
                },
                TextColor(muted_text_color),
                Node {
                    margin: UiRect::bottom(Px(16.0)),
                    ..default()
                },
            ));
        }

        // Children section
        p.spawn((
            Text::new(format!("Children ({})", children_len)),
            TextFont {
                font_size: title_font_size,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::new(Px(0.0), Px(0.0), Px(16.0), Px(8.0)),
                ..default()
            },
        ));

        if children_node_data.is_empty() {
            p.spawn((
                Text::new("No children"),
                TextFont {
                    font_size: body_font_size,
                    ..default()
                },
                TextColor(muted_text_color),
            ));
        } else {
            for (ent, name, comp_count) in children_node_data {
                let label = format!("{} ({} components)", name, comp_count);
                // Wrap button in container to handle margin (button() already includes Node)
                p.spawn(Node {
                    margin: UiRect::bottom(item_gap),
                    ..default()
                })
                .with_children(|wrapper| {
                    wrapper.spawn((
                        button(
                            ButtonProps::default(),
                            HierarchyNode(ent),
                            bevy::prelude::Spawn((
                                Text::new(label),
                                TextFont {
                                    font_size: body_font_size,
                                    ..default()
                                },
                                TextColor(Color::srgba(0.9, 0.9, 0.9, 1.0)),
                            )),
                        ),
                        observe(on_hierarchy_node_click),
                    ));
                });
            }
        }
    });
}

/// Spawns the detail panel structure.
pub fn spawn_detail_panel(parent: &mut ChildSpawnerCommands<'_>, config: &InspectorConfig) {
    parent
        .spawn((
            Node {
                flex_grow: 1.0,
                height: Percent(100.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Px(1.0)),
                ..default()
            },
            BorderColor::all(config.border_color),
            DetailPanel,
        ))
        .with_children(|panel| {
            // Tab bar
            panel
                .spawn((
                    Node {
                        width: Percent(100.0),
                        height: config.tab_bar_height,
                        display: Display::Flex,
                        align_items: AlignItems::Center,
                        padding: config.panel_padding,
                        column_gap: config.column_gap,
                        border: UiRect::bottom(Px(1.0)),
                        ..default()
                    },
                    BorderColor::all(config.border_color),
                ))
                .with_children(|tabs| {
                    // Components tab
                    tabs.spawn((
                        button(
                            ButtonProps::default(),
                            TabButton(DetailTab::Components),
                            bevy::prelude::Spawn((
                                Text::new("Components"),
                                TextFont {
                                    font_size: config.body_font_size,
                                    ..default()
                                },
                            )),
                        ),
                        observe(on_tab_button_click),
                    ));

                    // Relationships tab
                    tabs.spawn((
                        button(
                            ButtonProps::default(),
                            TabButton(DetailTab::Relationships),
                            bevy::prelude::Spawn((
                                Text::new("Relationships"),
                                TextFont {
                                    font_size: config.body_font_size,
                                    ..default()
                                },
                            )),
                        ),
                        observe(on_tab_button_click),
                    ));
                });

            // Scrollable area with scrollbar - use Grid layout
            let scrollbar_width = 8.0;
            panel
                .spawn(Node {
                    width: Percent(100.0),
                    flex_grow: 1.0,
                    display: Display::Grid,
                    grid_template_columns: vec![GridTrack::fr(1.0), GridTrack::px(scrollbar_width)],
                    ..default()
                })
                .with_children(|scroll_area| {
                    // Scroll content
                    let content_id = scroll_area
                        .spawn((
                            Node {
                                display: Display::Flex,
                                flex_direction: FlexDirection::Column,
                                row_gap: config.item_gap,
                                padding: config.panel_padding,
                                overflow: Overflow::scroll_y(),
                                ..default()
                            },
                            ScrollPosition::default(),
                            DetailContent,
                        ))
                        .id();

                    // Scrollbar
                    scroll_area
                        .spawn((
                            Scrollbar {
                                target: content_id,
                                orientation: ControlOrientation::Vertical,
                                min_thumb_length: 20.0,
                            },
                            Node {
                                width: Px(scrollbar_width),
                                height: Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.5)),
                        ))
                        .with_children(|sb| {
                            sb.spawn((
                                CoreScrollbarThumb,
                                Node {
                                    width: Percent(100.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.5, 0.5, 0.5, 0.8)),
                            ));
                        });
                });
        });
}
