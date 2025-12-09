//! Inspector window plugin and UI scaffold.

use bevy::camera::RenderTarget;
use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::ecs::relationship::Relationship;
use bevy::feathers::dark_theme::create_dark_theme;
use bevy::feathers::theme::{ThemeBackgroundColor, UiTheme};
use bevy::feathers::tokens;
use bevy::feathers::FeathersPlugins;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::hover::HoverMap;
use bevy::prelude::*;
use bevy::ui::Val::*;
use bevy::window::{WindowRef, WindowResolution};

use super::config::InspectorConfig;
use super::panels::{
    refresh_entity_cache, spawn_detail_panel, spawn_entity_list_panel, sync_detail_panel,
    sync_entity_list, sync_selection_highlight,
};
use super::semantic_names::SemanticFieldNames;
use super::state::{InspectorCache, InspectorInternal, InspectorState, InspectorWindowState};
use super::widgets::DragValuePlugin;

/// Marker component for the inspector window.
#[derive(Component)]
pub struct InspectorWindow;

/// Marker to indicate UI has been initialized.
#[derive(Component)]
struct InspectorUiInitialized;

/// System sets for organizing inspector systems.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum InspectorSet {
    /// Handle input events.
    Input,
    /// Refresh cached data.
    RefreshCache,
    /// Sync UI with state.
    SyncUI,
}

/// Plugin that manages the inspector window lifecycle.
pub struct InspectorWindowPlugin;

impl Plugin for InspectorWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FeathersPlugins)
            .add_plugins(DragValuePlugin)
            .insert_resource(UiTheme(create_dark_theme()))
            // State resources
            .init_resource::<InspectorState>()
            .init_resource::<InspectorCache>()
            .init_resource::<InspectorConfig>()
            .init_resource::<InspectorWindowState>()
            .init_resource::<SemanticFieldNames>()
            // System ordering
            .configure_sets(
                Update,
                (
                    InspectorSet::Input,
                    InspectorSet::RefreshCache,
                    InspectorSet::SyncUI,
                )
                    .chain(),
            )
            // Startup
            .add_systems(Startup, setup_inspector_window)
            // Update systems
            .add_systems(
                Update,
                (
                    // Input handling
                    handle_mouse_wheel_scroll.in_set(InspectorSet::Input),
                    // Cache refresh
                    refresh_entity_cache.in_set(InspectorSet::RefreshCache),
                    // UI sync - chain these to avoid resource conflicts
                    (
                        setup_inspector_ui,
                        sync_entity_list,
                        sync_detail_panel,
                        sync_selection_highlight,
                    )
                        .chain()
                        .in_set(InspectorSet::SyncUI),
                    // Cleanup
                    handle_window_close,
                ),
            );
    }
}

/// Spawns the inspector window on startup.
fn setup_inspector_window(mut commands: Commands, mut window_state: ResMut<InspectorWindowState>) {
    let window_entity = commands
        .spawn((
            Window {
                title: "Feathers Inspector".to_string(),
                resolution: WindowResolution::new(900, 650),
                ..default()
            },
            InspectorWindow,
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ))
        .id();

    window_state.window_entity = Some(window_entity);
    window_state.is_open = true;

    info!("Inspector window created: {:?}", window_entity);
}

/// Sets up the UI scaffold once the window exists.
fn setup_inspector_ui(
    mut commands: Commands,
    window_state: Res<InspectorWindowState>,
    config: Res<InspectorConfig>,
    mut cache: ResMut<InspectorCache>,
    inspector_windows: Query<Entity, (With<InspectorWindow>, Without<InspectorUiInitialized>)>,
) {
    let Some(window_entity) = window_state.window_entity else {
        return;
    };

    if inspector_windows.get(window_entity).is_err() {
        return;
    }

    // Mark window as initialized
    commands.entity(window_entity).insert(InspectorUiInitialized);

    // Create camera for the inspector window (marked as internal to exclude from entity list)
    let camera_entity = commands
        .spawn((
            Camera2d,
            Camera {
                target: RenderTarget::Window(WindowRef::Entity(window_entity)),
                ..default()
            },
            InspectorInternal,
        ))
        .id();

    // Build UI hierarchy
    commands
        .spawn((
            Node {
                width: Percent(100.0),
                height: Percent(100.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ThemeBackgroundColor(tokens::WINDOW_BG),
            UiTargetCamera(camera_entity),
        ))
        .with_children(|root| {
            // Title bar
            spawn_title_bar(root, &config);

            // Main content area
            root.spawn((
                Node {
                    width: Percent(100.0),
                    flex_grow: 1.0,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    padding: config.panel_padding,
                    column_gap: config.column_gap,
                    ..default()
                },
            ))
            .with_children(|content| {
                // Left panel: Entity list
                spawn_entity_list_panel(content, &config);

                // Right panel: Detail view
                spawn_detail_panel(content, &config);
            });
        });

    // Trigger initial cache refresh
    cache.stale = true;

    info!("Inspector UI initialized");
}

fn spawn_title_bar(parent: &mut ChildSpawnerCommands<'_>, config: &InspectorConfig) {
    parent
        .spawn((
            Node {
                width: Percent(100.0),
                height: config.title_bar_height,
                display: Display::Flex,
                align_items: AlignItems::Center,
                padding: config.panel_padding,
                border: UiRect::bottom(Px(1.0)),
                ..default()
            },
            BorderColor::all(config.border_color),
        ))
        .with_children(|bar| {
            bar.spawn((
                Text::new("Feathers Inspector"),
                TextFont {
                    font_size: config.title_font_size + 2.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

/// Handles cleanup when the inspector window is closed.
fn handle_window_close(
    mut window_state: ResMut<InspectorWindowState>,
    mut removed_windows: RemovedComponents<Window>,
) {
    for entity in removed_windows.read() {
        if window_state.window_entity == Some(entity) {
            window_state.window_entity = None;
            window_state.is_open = false;
            info!("Inspector window closed");
        }
    }
}

/// Handles mouse wheel scrolling by traversing up from hovered entities to find scrollable containers.
fn handle_mouse_wheel_scroll(
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    parents: Query<&ChildOf>,
    mut scrollables: Query<(&mut ScrollPosition, &Node, &ComputedNode)>,
) {
    for event in mouse_wheel_reader.read() {
        let mut delta = Vec2::new(event.x, event.y);
        if event.unit == MouseScrollUnit::Line {
            delta *= 20.0; // Convert lines to pixels
        }
        delta = -delta; // Invert for natural scrolling

        // Find any hovered entity
        for pointer_map in hover_map.values() {
            for &hovered_entity in pointer_map.keys() {
                // Traverse up to find scrollable ancestor
                let mut current = hovered_entity;
                loop {
                    if let Ok((mut scroll_pos, node, computed)) = scrollables.get_mut(current) {
                        // Found a scrollable container
                        if node.overflow.y == OverflowAxis::Scroll && delta.y != 0.0 {
                            let max_y = (computed.content_size().y - computed.size().y)
                                .max(0.0)
                                * computed.inverse_scale_factor();
                            scroll_pos.y = (scroll_pos.y + delta.y).clamp(0.0, max_y);
                        }
                        if node.overflow.x == OverflowAxis::Scroll && delta.x != 0.0 {
                            let max_x = (computed.content_size().x - computed.size().x)
                                .max(0.0)
                                * computed.inverse_scale_factor();
                            scroll_pos.x = (scroll_pos.x + delta.x).clamp(0.0, max_x);
                        }
                        return; // Stop after finding first scrollable ancestor
                    }

                    // Move up the hierarchy
                    if let Ok(child_of) = parents.get(current) {
                        current = child_of.get();
                    } else {
                        break;
                    }
                }
            }
        }
    }
}
