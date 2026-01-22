//! A panel that lists the objects that can be inspected in the current view,
//! typically shown on the left side of the inspector.
//!
//! See [`ObjectListTab`](crate::gui::state::ObjectListTab) for the different tabs available in this panel,
//! which is used to switch between different object types (e.g., entities, resources).

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::ecs::observer::On;
use bevy::ecs::relationship::Relationship;
use bevy::feathers::controls::{ButtonProps, button};
use bevy::prelude::*;
use bevy::ui::Val::*;
use bevy::ui_widgets::{Activate, ControlOrientation, CoreScrollbarThumb, Scrollbar};

use crate::component_inspection::ComponentMetadataMap;
use crate::entity_inspection::{MultipleEntityInspectionSettings, NameFilter};
use crate::entity_name_resolution::EntityName;
use crate::extension_methods::WorldInspectionExtensionTrait;
use crate::gui::config::InspectorConfig;
use crate::gui::state::{
    InspectableObject, InspectorCache, InspectorInternal, InspectorState, ObjectListEntry,
};
use crate::memory_size::MemorySize;

/// Marker component for the object list panel container.
#[derive(Component)]
pub struct ObjectListPanel;

/// Marker component for the scrollable object list content.
#[derive(Component)]
pub struct ObjectListContent;

/// Marker for object rows. Stores the object this row represents.
#[derive(Component)]
pub struct ObjectRow {
    pub selected_object: Entity,
}

/// Marker for the search input.
#[derive(Component)]
pub struct SearchInput;

/// Exclusive system that refreshes the [`InspectorCache`].
/// We can't sensibly cache results from one call to the next,
/// as the world may have changed substantially in between.
///
/// Uses exclusive world access to avoid resource conflicts.
pub fn refresh_object_cache(world: &mut World) {
    // Check if we need to refresh - extract state info first
    // TODO: we don't seem to actually check if the state changed?
    let state = world.resource::<InspectorState>();

    // Extract data we need now to avoid borrow check problems
    let filter_text = state.filter_text.clone();
    let mandatory_components = state.mandatory_components.clone();
    let metadata_map = world.resource_mut::<InspectorCache>().metadata_map.take();

    // Generate if needed, otherwise reuse and update
    let mut metadata_map = match metadata_map {
        Some(mut mm) => {
            mm.update(world);
            mm
        }
        None => ComponentMetadataMap::generate(world),
    };

    // Query all entities (excluding UI nodes, windows, and inspector-internal entities)
    // TODO: This should include UI nodes and windows, as long as they're not spawned by the inspector
    let mut query = world.query::<EntityRef>();
    let entities: Vec<Entity> = query
        .iter(world)
        .filter(|e| {
            !e.contains::<Node>()
                && !e.contains::<Window>()
                && !e.contains::<InspectorInternal>()
                // TODO: these should be shown in a separate object list tab
                // TODO: this inspector generates its own observers, which should be excluded
                // using InspectorInternal
                && !e.contains::<Observer>()
        })
        .map(|e| e.id())
        .collect();

    // Build inspection settings with filter
    let mut settings = MultipleEntityInspectionSettings::default();
    if !filter_text.is_empty() {
        settings.name_filter = Some(NameFilter::from(&filter_text));
    }
    if !mandatory_components.is_empty() {
        settings.with_component_filter = mandatory_components;
    }

    // Inspect entities
    let inspections = world.inspect_multiple(entities.iter().copied(), settings, &mut metadata_map);

    // Build filtered list - use object from each inspection since inspect_multiple reorders
    // TODO: inspect_multiple's ordering should be stable and sufficient for this purpose
    let filtered_entities: Vec<ObjectListEntry> = inspections
        .into_iter()
        .filter_map(|result| {
            let inspection = result.ok()?;
            let entity = inspection.entity;
            let name = inspection.name.unwrap_or(EntityName::generated(
                format!("Entity {:?}", entity).as_str(),
            ));

            // Apply text filter if set, excluding non-matching names
            if !filter_text.is_empty() && !name.to_lowercase().contains(&filter_text.to_lowercase())
            {
                return None;
            }

            Some(ObjectListEntry {
                entity,
                display_name: name.to_string(),
                component_count: inspection.components.as_ref().map(|c| c.len()).unwrap_or(0),
                memory_size: inspection.total_memory_size.unwrap_or(MemorySize::new(0)),
            })
        })
        .collect();

    // Put metadata_map back and update cache
    let mut cache = world.resource_mut::<InspectorCache>();
    cache.metadata_map = Some(metadata_map);
    cache.filtered_entities = filtered_entities;

    // Sort by entity for consistent display
    // TODO: this should use the ordering returned by inspect_multiple
    cache.filtered_entities.sort_by_key(|e| e.entity.index());
}

/// System that syncs the object list display with the cache.
pub fn sync_object_list(
    mut commands: Commands,
    cache: ResMut<InspectorCache>,
    state: Res<InspectorState>,
    config: Res<InspectorConfig>,
    list_content: Query<Entity, With<ObjectListContent>>,
    existing_rows: Query<Entity, With<ObjectRow>>,
) {
    let Ok(content_entity) = list_content.iter().next().ok_or(()) else {
        return;
    };

    // Clear existing rows
    // TODO: can we reuse existing rows instead of despawning all?
    for row_entity in existing_rows.iter() {
        commands.entity(row_entity).despawn();
    }

    // Spawn new rows
    commands.entity(content_entity).with_children(|list| {
        for entry in &cache.filtered_entities {
            let is_selected =
                state.selected_object == Some(InspectableObject::Entity(entry.entity));
            spawn_object_row(list, entry, is_selected, &config);
        }
    });
}

/// Spawns a single object row button.
fn spawn_object_row(
    parent: &mut ChildSpawnerCommands<'_>,
    entry: &ObjectListEntry,
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

    parent.spawn((button(
        ButtonProps::default(),
        ObjectRow {
            selected_object: entry.entity,
        },
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
    ),));
}

/// Global observer for object row clicks.
/// Added in [`InspectorWindowPlugin`](crate::gui::InspectorWindowPlugin).
///
/// Traverses up the parent hierarchy to find the [`ObjectRow`] component.
pub fn on_object_row_click(
    activate: On<Activate>,
    mut state: ResMut<InspectorState>,
    rows: Query<&ObjectRow>,
    parents: Query<&ChildOf>,
) {
    // Traverse up the hierarchy to find ObjectRow
    let mut current = activate.entity;
    // Make sure the event is actually targeting an object row
    if !rows.contains(current) {
        return;
    }

    loop {
        if let Ok(row) = rows.get(current) {
            state.selected_object = Some(InspectableObject::Entity(row.selected_object));
            return;
        }
        if let Ok(child_of) = parents.get(current) {
            current = child_of.get();
        } else {
            break;
        }
    }
    warn!("Could not find ObjectRow in hierarchy!");
}

/// Spawns the object list panel structure.
pub fn spawn_object_list_panel(parent: &mut ChildSpawnerCommands<'_>, config: &InspectorConfig) {
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
            ObjectListPanel,
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
                            ObjectListContent,
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

/// A message that drives a refresh of the object list panel.
///
/// This will cause [`refresh_object_cache`] and [`sync_object_list`] to run when seen,
/// via the use of run conditions added as part of [`InspectorWindowPlugin`](crate::gui::plugin::InspectorWindowPlugin).
///
/// This is a public message to allow users to trigger (or cancel!) refreshes manually if desired.
#[derive(Message)]
pub struct RefreshObjectList;

/// A system which periodically sends a [`RefreshObjectList`] message.
///
/// The frequency is controlled by the `refresh_interval` field in [`InspectorConfig`].
pub fn periodic_object_list_refresh(
    mut message_writer: MessageWriter<RefreshObjectList>,
    time: Res<Time>,
    mut timer: Local<Timer>,
    config: Res<InspectorConfig>,
) {
    // Skip if no refresh interval is set;
    // This disables auto-refreshing.
    if config.refresh_interval.is_none() {
        return;
    }

    // Initialize timer if needed
    if timer.duration().is_zero() {
        *timer = Timer::new(config.refresh_interval.unwrap(), TimerMode::Repeating);
    }

    // Tick timer
    timer.tick(time.delta());

    // Send message if timer finished
    if timer.just_finished() {
        message_writer.write(RefreshObjectList);
    }
}
