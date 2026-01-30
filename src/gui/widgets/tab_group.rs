//! Organizes [`Node`]s into separate views, where only one is visible at a time.
//!
//! # Example
//!
//! Use the helper function [`spawn_tab_group`] to insert the widget.
//!
//! ```
//! use bevy::prelude::*;
//! use feathers_inspector::gui::widgets::{spawn_tab_group, SwitchTab};
//!
//! fn setup(mut commands: Commands) {
//!     commands.spawn(Node::default()).with_children(|parent| {
//!         let (tab_group, panels) = spawn_tab_group(parent, vec![
//!             ("Tab A", Box::new(|parent| {
//!                 parent.spawn((
//!                     Node {
//!                         width: Val::Percent(100.0),
//!                         height: Val::Percent(100.0),
//!                         ..default()
//!                     },
//!                     BackgroundColor(Color::srgb(1.0, 0.0, 0.0)),
//!                 ));
//!             })),
//!             ("Tab B", Box::new(|parent| {
//!                 parent.spawn(Text::new("Hello World"));
//!             })),
//!         ]);
//!
//!         // Trigger initial tab
//!         if let Some(&first_panel) = panels.first() {
//!             parent.commands().trigger(SwitchTab {
//!                 target: tab_group,
//!                 panel: first_panel,
//!             });
//!         }
//!     });
//! }
//! ```

use bevy::ecs::event::EntityEvent;
use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::feathers::theme::ThemeBackgroundColor;
use bevy::feathers::tokens;
use bevy::prelude::*;

/// Root marker for the TabGroup widget.
#[derive(Component)]
pub struct TabGroup;

/// Marker for the header UI node that holds tab buttons (see [`TabTrigger`]).
#[derive(Component)]
pub struct TabGroupHeader;

/// Marker for the body UI node that holds tab panels with content (see [`TabPanel`]).
#[derive(Component)]
pub struct TabGroupBody;

/// Marker for tab panels containing content.
#[derive(Component)]
pub struct TabPanel;

/// Points to the [`TabPanel`] associated with a button.
#[derive(Component)]
#[relationship(relationship_target = TabTriggerTarget)]
pub struct TabTrigger {
    /// The [`TabPanel`] associated to this button.
    #[relationship]
    pub target: Entity,
}

/// Target component for [`TabTrigger`] relationship.
#[derive(Component, Default)]
#[relationship_target(relationship = TabTrigger)]
pub struct TabTriggerTarget(Vec<Entity>);

/// Points to the root [`TabGroup`] entity.
///
/// This is used to easily find the root [`TabGroup`] entity
/// from a [`TabTrigger`] without having to traverse the hierarchy.
#[derive(Component)]
#[relationship(relationship_target = TabGroupRootTarget)]
pub struct TabTriggerRoot(pub Entity);

/// Target component for [`TabTriggerRoot`] relationship.
#[derive(Component, Default)]
#[relationship_target(relationship = TabTriggerRoot)]
pub struct TabGroupRootTarget(Vec<Entity>);

/// Event triggered when a tab needs to be switched.
#[derive(EntityEvent, Clone, Debug)]
pub struct SwitchTab {
    /// The [`TabGroup`] that this event targets.
    #[event_target]
    pub target: Entity,
    /// The [`TabPanel`] that needs to be made visible.
    pub panel: Entity,
}

/// Observes for [`Click`]ed tab buttons to trigger the [`SwitchTab`] event.
fn trigger_switch_tab_on_click(
    on_click: On<Pointer<Click>>,
    tab_trigger_query: Query<(&TabTrigger, &TabTriggerRoot)>,
    mut commands: Commands,
) {
    let clicked_button = on_click.entity;
    let Ok((tab_trigger, to_root)) = tab_trigger_query.get(clicked_button) else {
        return;
    };

    commands.trigger(SwitchTab {
        target: to_root.0,
        panel: tab_trigger.target,
    });
}

/// Observes [`SwitchTab`] to update panel visibility.
fn update_panel_visibility_on_switch_tab(
    on_switch_tab: On<SwitchTab>,
    root_query: Query<Option<&Children>, With<TabGroup>>,
    body_query: Query<(Entity, Option<&Children>), With<TabGroupBody>>,
    mut tab_panel_query: Query<&mut Node, With<TabPanel>>,
) {
    let event = on_switch_tab.event();
    let root_entity = on_switch_tab.observer();
    let Ok(root_children) = root_query.get(root_entity) else {
        warn!("`TabGroup` does not have children.");
        return;
    };
    let Some(root_children) = root_children else {
        warn!("`TabGroup` has no children.");
        return;
    };
    let Some((_, body_children)) = root_children
        .iter()
        .find_map(|child| body_query.get(child).ok())
    else {
        warn!("Could not find body in `TabGroup`.");
        return;
    };
    let Some(panels) = body_children else {
        return;
    };

    // While cycling through all tabs is slower than tracking the last active tab,
    // it is worth it in this case because it lets us avoid setting up sync logic.
    for &panel in panels {
        let Ok(mut node) = tab_panel_query.get_mut(panel) else {
            continue;
        };
        let new_display = if panel == event.panel {
            Display::Flex
        } else {
            Display::None
        };

        // Checking avoids needless triggering of change detection.
        if node.display != new_display {
            node.display = new_display;
        }
    }
}

/// Observes [`SwitchTab`] to update button styling.
fn update_button_appearance_on_switch_tab(
    on_switch_tab: On<SwitchTab>,
    root_query: Query<Option<&Children>, With<TabGroup>>,
    header_query: Query<Option<&Children>, With<TabGroupHeader>>,
    tab_trigger_query: Query<(&TabTrigger, Option<&ThemeBackgroundColor>)>,
    mut commands: Commands,
) {
    let event = on_switch_tab.event();
    let root_entity = on_switch_tab.observer();
    let Ok(root_children) = root_query.get(root_entity) else {
        warn!("`TabGroup` does not have children.");
        return;
    };
    let Some(root_children) = root_children else {
        return;
    };
    let Some(header_children) = root_children
        .iter()
        .find_map(|child| header_query.get(child).ok())
    else {
        warn!("Could not find header in `TabGroup`.");
        return;
    };

    let Some(buttons) = header_children else {
        return;
    };

    // While cycling through all tabs is slower than tracking the last active tab,
    // it is worth it in this case because it lets us avoid setting up sync logic.
    for &button in buttons {
        let Ok((trigger, theme_bg)) = tab_trigger_query.get(button) else {
            continue;
        };
        let target_token = if trigger.target == event.panel {
            tokens::BUTTON_PRIMARY_BG
        } else {
            tokens::WINDOW_BG
        };
        if let Some(bg) = theme_bg {
            // Checking avoids needless triggering of change detection.
            if bg.0 != target_token {
                commands
                    .entity(button)
                    .insert(ThemeBackgroundColor(target_token));
            }
        } else {
            commands
                .entity(button)
                .insert(ThemeBackgroundColor(target_token));
        }
    }
}

/// Spawns a [`TabGroup`], returning its own `Entity` and the panels' ones.
pub fn spawn_tab_group<'a>(
    parent: &mut ChildSpawnerCommands,
    tabs: Vec<(&str, Box<dyn FnOnce(&mut ChildSpawnerCommands) + 'a>)>,
) -> (Entity, Vec<Entity>) {
    let mut panel_entities: Vec<Entity> = Vec::with_capacity(tabs.len());
    let mut button_configs: Vec<String> = Vec::with_capacity(tabs.len());

    let mut root_commands = parent.spawn((
        TabGroup,
        Node {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            ..default()
        },
    ));
    let root = root_commands.id();

    root_commands
        .observe(update_panel_visibility_on_switch_tab)
        .observe(update_button_appearance_on_switch_tab)
        .with_children(|root_builder| {
            let header = spawn_header_container(root_builder);
            spawn_body(root_builder, tabs, &mut panel_entities, &mut button_configs);

            // Spawn buttons in header
            spawn_header_buttons(
                &mut root_builder.commands(),
                header,
                panel_entities.clone(),
                button_configs,
                root,
            );
        });

    (root, panel_entities)
}

fn spawn_header_container(parent: &mut ChildSpawnerCommands) -> Entity {
    parent
        .spawn((
            TabGroupHeader,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(30.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                ..default()
            },
            BackgroundColor(Color::BLACK),
        ))
        .id()
}

fn spawn_body<'a>(
    parent: &mut ChildSpawnerCommands,
    tabs: Vec<(&str, Box<dyn FnOnce(&mut ChildSpawnerCommands) + 'a>)>,
    panel_entities: &mut Vec<Entity>,
    button_configs: &mut Vec<String>,
) -> Option<Entity> {
    let body = parent
        .spawn((
            TabGroupBody,
            Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                ..default()
            },
        ))
        .id();

    parent
        .commands()
        .entity(body)
        .with_children(|parent: &mut ChildSpawnerCommands| {
            for (label, builder) in tabs {
                let panel = parent
                    .spawn((
                        TabPanel,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            display: Display::None,
                            ..default()
                        },
                    ))
                    .with_children(builder)
                    .id();
                panel_entities.push(panel);
                let label_str = label.to_string();
                button_configs.push(label_str);
            }
        });

    panel_entities.first().copied()
}

fn spawn_header_buttons(
    commands: &mut Commands,
    header: Entity,
    panel_entities: Vec<Entity>,
    button_configs: Vec<String>,
    root: Entity,
) {
    commands.entity(header).with_children(|parent| {
        for (i, label) in button_configs.into_iter().enumerate() {
            let target = panel_entities[i];
            parent
                .spawn((
                    Button,
                    TabTrigger { target },
                    TabTriggerRoot(root),
                    Node {
                        padding: UiRect::all(Val::Px(5.0)),
                        margin: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    ThemeBackgroundColor(tokens::WINDOW_BG),
                ))
                .observe(trigger_switch_tab_on_click)
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(label),
                        TextFont::default(),
                        TextColor(Color::WHITE),
                    ));
                });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::world::CommandQueue;

    #[test]
    fn spawn_tab_group_structure() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, app.world());

        commands.spawn(Node::default()).with_children(|parent| {
            spawn_tab_group(
                parent,
                vec![
                    (
                        "Tab 1",
                        Box::new(|parent| {
                            parent.spawn(Node::default());
                        }),
                    ),
                    (
                        "Tab 2",
                        Box::new(|parent| {
                            parent.spawn(Node::default());
                        }),
                    ),
                ],
            );
        });

        queue.apply(app.world_mut());
        let world = app.world_mut();

        // Verify Root exists
        let root_entity = world
            .query_filtered::<Entity, With<TabGroup>>()
            .single(world)
            .expect("`TabGroup` root entity should be spawned");

        // Verify Header and Body exist as children
        let root_children: Vec<Entity> = world
            .get::<Children>(root_entity)
            .expect("`TabGroup` root should have children")
            .to_vec();
        assert_eq!(
            root_children.len(),
            2,
            "`TabGroup` should have exactly two children (`TabGroupHeader` and `TabGroupBody`)"
        );
        let header = world
            .query_filtered::<Entity, With<TabGroupHeader>>()
            .single(world)
            .expect("`TabGroupHeader` entity should be spawned");
        let body = world
            .query_filtered::<Entity, With<TabGroupBody>>()
            .single(world)
            .expect("`TabGroupBody` entity should be spawned");
        assert!(
            root_children.contains(&header),
            "`TabGroupHeader` should be a child of the `TabGroup` root"
        );
        assert!(
            root_children.contains(&body),
            "`TabGroupBody` should be a child of the `TabGroup` root"
        );

        // Verify Buttons in Header
        let header_children = world
            .get::<Children>(header)
            .expect("`TabGroupHeader` should have children (buttons)");

        // Verify Text in Button (New Structure)
        for &button in header_children {
            let button_children = world
                .get::<Children>(button)
                .expect("Tab button should have children (text label)");
            assert_eq!(
                button_children.len(),
                1,
                "Button should have exactly one child (`Text`)"
            );
            let text_entity = button_children[0];
            assert!(
                world.get::<Text>(text_entity).is_some(),
                "Button's child should have `Text` component"
            );
        }
    }
}
