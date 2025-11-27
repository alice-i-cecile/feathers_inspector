//! Entity list panel for the left side of the inspector.

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::ecs::observer::On;
use bevy::ecs::relationship::Relationship;
use bevy::feathers::controls::{button, ButtonProps};
use bevy::prelude::*;
use bevy::ui::Val::*;
use bevy::ui_widgets::{observe, Activate, ControlOrientation, CoreScrollbarThumb, Scrollbar};

use crate::component_inspection::ComponentMetadataMap;
use crate::entity_inspection::{EntityInspectExtensionTrait, MultipleEntityInspectionSettings};
use crate::inspector::config::InspectorConfig;
use crate::inspector::state::{EntityListEntry, InspectorCache, InspectorInternal, InspectorState};
use crate::memory_size::MemorySize;

/// Marker component for the entity list panel container.
#[derive(Component)]
pub struct EntityListPanel;

/// Marker component for the scrollable entity list content.
#[derive(Component)]
pub struct EntityListContent;

/// Marker for entity rows. Stores the entity this row represents.
#[derive(Component)]
pub struct EntityRow(pub Entity);

/// Marker for the search input.
#[derive(Component)]
pub struct SearchInput;

/// Exclusive system that refreshes the entity cache when state changes.
/// Uses exclusive world access to avoid resource conflicts.
pub fn refresh_entity_cache(world: &mut World) {
    // Check if we need to refresh - extract state info first
    let state = world.resource::<InspectorState>();
    let cache = world.resource::<InspectorCache>();

    let needs_refresh = cache.stale;
    let filter_text = state.filter_text.clone();
    let required_components = state.required_components.clone();

    if !needs_refresh {
        return;
    }

    // Take metadata map out to avoid borrow conflicts
    let mut metadata_map = world.resource_mut::<InspectorCache>().metadata_map.take();

    // Generate if needed
    if metadata_map.is_none() {
        metadata_map = Some(ComponentMetadataMap::generate(world));
    }

    // Update existing metadata map
    if let Some(ref mut mm) = metadata_map {
        mm.update(world);
    }

    // Query all entities (excluding UI nodes, windows, and inspector-internal entities)
    let mut query = world.query::<EntityRef>();
    let entities: Vec<Entity> = query
        .iter(world)
        .filter(|e| {
            !e.contains::<Node>()
                && !e.contains::<Window>()
                && !e.contains::<InspectorInternal>()
        })
        .map(|e| e.id())
        .collect();

    // Build inspection settings with filter
    let mut settings = MultipleEntityInspectionSettings::default();
    if !filter_text.is_empty() {
        settings.name_filter = Some(filter_text.clone());
    }
    if !required_components.is_empty() {
        settings.with_component_filter = required_components;
    }

    // Inspect entities
    let inspections = if let Some(ref mut mm) = metadata_map {
        world.inspect_multiple(entities.iter().copied(), settings, mm)
    } else {
        vec![]
    };

    // Build filtered list - use entity from each inspection since inspect_multiple reorders
    let filtered_entities: Vec<EntityListEntry> = inspections
        .into_iter()
        .filter_map(|result| {
            let inspection = result.ok()?;
            let entity = inspection.entity;
            let name = metadata_map
                .as_ref()
                .and_then(|mm| inspection.resolve_name(&mm.map))
                .unwrap_or_else(|| format!("Entity {:?}", entity));

            // Apply text filter
            if !filter_text.is_empty()
                && !name.to_lowercase().contains(&filter_text.to_lowercase())
            {
                return None;
            }

            Some(EntityListEntry {
                entity,
                display_name: name,
                component_count: inspection.components.as_ref().map(|c| c.len()).unwrap_or(0),
                memory_size: inspection.total_memory_size.unwrap_or(MemorySize::new(0)),
            })
        })
        .collect();

    // Put metadata_map back and update cache
    let mut cache = world.resource_mut::<InspectorCache>();
    cache.metadata_map = metadata_map;
    cache.filtered_entities = filtered_entities;

    // Sort by entity for consistent display
    cache.filtered_entities.sort_by_key(|e| e.entity.index());
    cache.stale = false;
}

/// System that syncs the entity list display with the cache.
pub fn sync_entity_list(
    mut commands: Commands,
    cache: ResMut<InspectorCache>,
    state: Res<InspectorState>,
    config: Res<InspectorConfig>,
    list_content: Query<Entity, With<EntityListContent>>,
    existing_rows: Query<Entity, With<EntityRow>>,
) {
    // Only update when cache or selection changes
    if !cache.is_changed() && !state.is_changed() {
        return;
    }

    let Ok(content_entity) = list_content.iter().next().ok_or(()) else {
        return;
    };

    // Clear existing rows
    for row_entity in existing_rows.iter() {
        commands.entity(row_entity).despawn();
    }

    // Spawn new rows
    commands.entity(content_entity).with_children(|list| {
        for entry in &cache.filtered_entities {
            let is_selected = state.selected_entity == Some(entry.entity);
            spawn_entity_row(list, entry, is_selected, &config);
        }
    });
}

/// Spawns a single entity row button.
fn spawn_entity_row(
    parent: &mut ChildSpawnerCommands<'_>,
    entry: &EntityListEntry,
    is_selected: bool,
    config: &InspectorConfig,
) {
    // Truncate long names
    let display_name = if entry.display_name.len() > 20 {
        format!("{}...", &entry.display_name[..17])
    } else {
        entry.display_name.clone()
    };

    let label = format!(
        "{:20} {} comp | {}",
        display_name, entry.component_count, entry.memory_size
    );

    parent.spawn((
        button(
            ButtonProps::default(),
            EntityRow(entry.entity),
            bevy::prelude::Spawn((
                Text::new(label),
                TextFont {
                    font_size: config.small_font_size,
                    ..default()
                },
                TextColor(if is_selected {
                    Color::WHITE
                } else {
                    Color::srgba(0.9, 0.9, 0.9, 1.0)
                }),
            )),
        ),
        observe(on_entity_row_click),
    ));
}

/// Observer for entity row clicks.
/// Traverses up the parent hierarchy to find the EntityRow component.
fn on_entity_row_click(
    activate: On<Activate>,
    mut state: ResMut<InspectorState>,
    rows: Query<&EntityRow>,
    parents: Query<&ChildOf>,
) {
    // Traverse up the hierarchy to find EntityRow
    let mut current = activate.entity;
    loop {
        if let Ok(row) = rows.get(current) {
            state.selected_entity = Some(row.0);
            return;
        }
        if let Ok(child_of) = parents.get(current) {
            current = child_of.get();
        } else {
            break;
        }
    }
    warn!("Could not find EntityRow in hierarchy!");
}

/// System that updates selection highlight without respawning rows.
/// Note: Selection highlighting is handled during row spawning in sync_entity_list.
/// This system is a placeholder for future improvements.
pub fn sync_selection_highlight(
    _state: Res<InspectorState>,
    _rows: Query<&EntityRow>,
) {
    // Selection highlighting is handled during row spawning in sync_entity_list.
    // This system is a no-op placeholder for future improvements.
}

/// Spawns the entity list panel structure.
pub fn spawn_entity_list_panel(parent: &mut ChildSpawnerCommands<'_>, config: &InspectorConfig) {
    parent
        .spawn((
            Node {
                width: config.left_panel_width,
                height: Percent(100.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Px(1.0)),
                ..default()
            },
            BorderColor::all(config.border_color),
            EntityListPanel,
        ))
        .with_children(|panel| {
            // Search bar placeholder
            panel
                .spawn((
                    Node {
                        width: Percent(100.0),
                        padding: config.panel_padding,
                        border: UiRect::bottom(Px(1.0)),
                        ..default()
                    },
                    BorderColor::all(config.border_color),
                    SearchInput,
                ))
                .with_children(|search| {
                    search.spawn((
                        Text::new("Search entities..."),
                        TextFont {
                            font_size: config.body_font_size,
                            ..default()
                        },
                        TextColor(config.muted_text_color),
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
                            EntityListContent,
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
