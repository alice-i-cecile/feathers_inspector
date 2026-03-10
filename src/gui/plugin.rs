//! Inspector window plugin and UI scaffold.

use bevy::camera::RenderTarget;
use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::ecs::relationship::Relationship;
use bevy::feathers::FeathersPlugins;
use bevy::feathers::controls::{ButtonProps, button};
use bevy::feathers::dark_theme::create_dark_theme;
use bevy::feathers::theme::{ThemeBackgroundColor, UiTheme};
use bevy::feathers::tokens;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::hover::HoverMap;
use bevy::prelude::*;
use bevy::ui::Val::*;
use bevy::ui_widgets::Activate;
use bevy::window::{PrimaryWindow, WindowCloseRequested, WindowRef, WindowResolution};

use crate::gui::cache::{InspectorCache, update_inspector_cache};
use crate::gui::panels::{
    RefreshCache, RefreshUI, on_object_row_click, periodically_refresh_cache,
    update_active_objects_tab_on_tab_activated,
};

use super::config::InspectorConfig;
use super::panels::{
    render_detail_panel, render_object_list, spawn_detail_panel, spawn_object_list_panel,
};
use super::semantic_names::SemanticFieldNames;
use super::state::{InspectorInternal, InspectorState};
use super::widgets::drag_value::DragValuePlugin;
use super::widgets::tabs::TabPlugin;

/// Marker component for the inspector window.
#[derive(Component)]
pub struct InspectorWindow;

/// Marker to indicate UI has been initialized.
#[derive(Component)]
struct InspectorUiInitialized;

/// Marker component for the pause button.
#[derive(Component)]
pub struct PauseButton;

/// Marker component for the refresh button.
#[derive(Component)]
pub struct RefreshButton;

/// System sets for organizing inspector systems.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum InspectorSet {
    /// Handle input events.
    Input,
    /// Refresh cached data.
    ///
    /// Cache is needed because the [`World`] could mutate unpredictably,
    /// therefore, while not guaranteed to be up to date,
    /// it allows to take a snapshot of it.
    CacheUpdate,
    /// Sync UI with cache.
    Render,
    /// Sync UI with state.
    SyncUI,
}

/// Plugin that manages the inspector window lifecycle.
pub struct InspectorWindowPlugin;

impl Plugin for InspectorWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FeathersPlugins)
            .add_plugins(DragValuePlugin)
            .add_plugins(TabPlugin)
            .insert_resource(UiTheme(create_dark_theme()))
            // Resources
            .init_resource::<InspectorState>()
            .init_resource::<InspectorCache>()
            .init_resource::<InspectorConfig>()
            .init_resource::<SemanticFieldNames>()
            // Messages
            .add_message::<SetInspectorWindow>()
            .add_message::<RefreshCache>()
            .add_message::<RefreshUI>()
            // System ordering
            .configure_sets(
                Update,
                (
                    InspectorSet::Input,
                    InspectorSet::CacheUpdate,
                    InspectorSet::SyncUI,
                    InspectorSet::Render,
                )
                    .chain(),
            )
            // Limit refreshes
            .configure_sets(
                Update,
                (InspectorSet::CacheUpdate, InspectorSet::SyncUI)
                    .run_if(on_message::<RefreshCache>.or_else(on_message::<SetInspectorWindow>)),
            )
            .configure_sets(
                Update,
                InspectorSet::Render.run_if(
                    on_message::<RefreshCache>
                        .or_else(on_message::<RefreshUI>)
                        .or_else(on_message::<SetInspectorWindow>),
                ),
            )
            // Startup
            .add_systems(Startup, order_inspector_window_creation)
            // PreUpdate systems
            .add_systems(PreUpdate, periodically_refresh_cache)
            // Update systems
            .add_systems(
                Update,
                (
                    // Input handling
                    (handle_mouse_wheel_scroll, handle_toggle_key).in_set(InspectorSet::Input),
                    // Cache refresh
                    update_inspector_cache.in_set(InspectorSet::CacheUpdate),
                    // UI sync - chain these to avoid resource conflicts
                    (toggle_inspector_window, setup_inspector_ui)
                        .chain()
                        .in_set(InspectorSet::SyncUI),
                    // Render systems (Unconditional)
                    (
                        render_object_list,
                        render_detail_panel,
                        update_toolbar_buttons,
                    )
                        .chain()
                        .in_set(InspectorSet::Render),
                ),
            )
            .add_observer(toggle_is_paused_on_activate)
            .add_observer(manual_refresh_on_activate)
            .add_observer(on_object_row_click)
            .add_observer(update_active_objects_tab_on_tab_activated);
    }
}

/// Signals the plugin to open or close the [`InspectorWindow`].
#[derive(Message, Debug)]
pub enum SetInspectorWindow {
    // Creates the window if it is not present,
    // and destroys it if already present.
    Toggle,
    // Opens the window.
    //
    // If the window is already open, a warning message will be emitted.
    Open,
    // Closes the window.
    //
    // If there is no window, a warning message will be emitted.
    Close,
}

/// Sends a message to create an [`InspectorWindow`] if not already present.
fn order_inspector_window_creation(
    inspector_window_query: Query<Entity, With<InspectorWindow>>,
    mut window_messages: MessageWriter<SetInspectorWindow>,
    config: Res<InspectorConfig>,
) {
    if config.open_on_startup && inspector_window_query.iter().next().is_none() {
        window_messages.write(SetInspectorWindow::Open);
    }
}

/// Listens for [`SetInspectorWindow`] to create or destroy an [`InspectorWindow`].
fn toggle_inspector_window(
    mut action: MessageReader<SetInspectorWindow>,
    mut close_window: MessageWriter<WindowCloseRequested>,
    primary_window_query: Query<Entity, With<PrimaryWindow>>,
    window_query: Query<Entity, With<InspectorWindow>>,
    commands: Commands,
) {
    use SetInspectorWindow::{Close, Open, Toggle};
    let Some(action) = action.read().last() else {
        return;
    };
    let window_opt = window_query.iter().next();

    match (action, window_opt) {
        (Toggle, window) => {
            if let Some(window) = window {
                close_window.write(WindowCloseRequested { window });
            } else {
                spawn_inspector_window(primary_window_query, commands);
            }
        }
        (Open, None) => spawn_inspector_window(primary_window_query, commands),
        (Close, Some(window)) => {
            close_window.write(WindowCloseRequested { window });
        }
        (action, window_opt) => {
            warn!("Invalid operation: action: {action:?}, window: {window_opt:?}")
        }
    }
}

/// Handles the keyboard input for toggling the [`InspectorWindow`].
fn handle_toggle_key(
    button_input: Res<ButtonInput<KeyCode>>,
    mut window_messages: MessageWriter<SetInspectorWindow>,
    config: Res<InspectorConfig>,
) {
    if let Some(toggle_key) = config.toggle_key
        && button_input.just_pressed(toggle_key)
    {
        window_messages.write(SetInspectorWindow::Toggle);
    }
}

fn spawn_inspector_window(
    primary_window: Query<Entity, With<PrimaryWindow>>,
    mut commands: Commands,
) {
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
            InspectorInternal,
        ))
        .id();

    // Inserting the inspector window as a child of the primary window
    // allows the app to close when closing the primary window.
    // Otherwise, you would need to close both windows.
    if let Ok(primary_window_entity) = primary_window.single() {
        commands
            .entity(window_entity)
            .insert(ChildOf(primary_window_entity));
    }

    info!("Inspector window created: {:?}", window_entity);
}

/// Sets up the UI scaffold once the window exists.
fn setup_inspector_ui(
    mut commands: Commands,
    config: Res<InspectorConfig>,
    state: Res<InspectorState>,
    inspector_windows: Query<Entity, (With<InspectorWindow>, Without<InspectorUiInitialized>)>,
) {
    let Some(window_entity) = inspector_windows.iter().next() else {
        return;
    };

    // Mark window as initialized
    commands
        .entity(window_entity)
        .insert(InspectorUiInitialized);

    // Create camera for the inspector window (marked as internal to exclude from entity list)
    let camera_entity = commands
        .spawn((
            Camera2d,
            Camera::default(),
            RenderTarget::Window(WindowRef::Entity(window_entity)),
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
            spawn_title_bar(root, &config, &state);

            // Main content area
            root.spawn((Node {
                width: Percent(100.0),
                flex_grow: 1.0,
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                padding: config.panel_padding,
                column_gap: config.column_gap,
                ..default()
            },))
                .with_children(|content| {
                    // Left panel: Object list
                    spawn_object_list_panel(content, &config);
                    // Right panel: Detail view
                    spawn_detail_panel(content, &config);
                });
        });

    info!("Inspector UI initialized");
}

fn spawn_title_bar(
    parent: &mut ChildSpawnerCommands<'_>,
    config: &InspectorConfig,
    state: &InspectorState,
) {
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
                    font_size: FontSize::Px(config.title_font_size + 2.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Flexible spacer
            bar.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });

            // Toolbar action container
            bar.spawn(Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(5.0),
                flex_grow: 0.0,
                ..default()
            })
            .with_children(|actions| {
                // Wrapper because `Node` on `button` triggers segfault.
                actions
                    .spawn(Node {
                        width: Val::Px(80.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    })
                    .with_children(|wrapper| {
                        wrapper.spawn(button(
                            ButtonProps::default(),
                            RefreshButton,
                            bevy::prelude::Spawn((
                                Text::new("Refresh"),
                                TextFont {
                                    font_size: FontSize::Px(config.body_font_size),
                                    ..default()
                                },
                            )),
                        ));
                    });

                // Wrapper because `Node` on `button` triggers segfault.
                actions
                    .spawn(Node {
                        width: Val::Px(80.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    })
                    .with_children(|wrapper| {
                        wrapper.spawn(button(
                            ButtonProps::default(),
                            PauseButton,
                            bevy::prelude::Spawn((
                                Text::new(if state.is_paused { "Resume" } else { "Pause" }),
                                TextFont {
                                    font_size: FontSize::Px(config.body_font_size),
                                    ..default()
                                },
                            )),
                        ));
                    });
            });
        });
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
                            let max_y = (computed.content_size().y - computed.size().y).max(0.0)
                                * computed.inverse_scale_factor();
                            scroll_pos.y = (scroll_pos.y + delta.y).clamp(0.0, max_y);
                        }
                        if node.overflow.x == OverflowAxis::Scroll && delta.x != 0.0 {
                            let max_x = (computed.content_size().x - computed.size().x).max(0.0)
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

/// Observes [`Activate`] events to toggle `is_paused` on [`InspectorState`].
fn toggle_is_paused_on_activate(
    activate: On<Activate>,
    mut state: ResMut<InspectorState>,
    pause_button_query: Query<Entity, With<PauseButton>>,
    mut writer: MessageWriter<RefreshCache>,
) {
    let Some(_pause_button) = pause_button_query.get(activate.entity).ok() else {
        return;
    };

    state.is_paused = !state.is_paused;

    // Forces a cache refresh to get the freshest data.
    writer.write(RefreshCache);
}

/// Observes [`Activate`] events to trigger manual refresh.
fn manual_refresh_on_activate(
    activate: On<Activate>,
    refresh_button_query: Query<Entity, With<RefreshButton>>,
    mut writer: MessageWriter<RefreshCache>,
    mut cache: ResMut<InspectorCache>,
) {
    let Some(_refresh_button) = refresh_button_query.get(activate.entity).ok() else {
        return;
    };

    cache.is_dirty = true;
    writer.write(RefreshCache);
}

/// Syncs the text and visibility of the toolbar buttons with the current [`InspectorState`].
fn update_toolbar_buttons(
    state: Res<InspectorState>,
    mut refresh_buttons: Query<&mut Node, With<RefreshButton>>,
    pause_buttons: Query<&Children, With<PauseButton>>,
    mut text_query: Query<&mut Text>,
) {
    if !state.is_changed() {
        return;
    }

    let is_paused = state.is_paused;

    // Update Refresh Button visibility
    for mut node in &mut refresh_buttons {
        node.display = if is_paused {
            Display::Flex
        } else {
            Display::None
        };
    }

    // Update Pause Button text
    for children in &pause_buttons {
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                text.0 = if is_paused { "Resume" } else { "Pause" }.to_string();
            }
        }
    }
}
