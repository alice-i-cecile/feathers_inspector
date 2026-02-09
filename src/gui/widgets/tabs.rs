//! Tabs are a container widget that organizes child widgets into multiple
//! sections, each represented by a tab header. Users can switch between different
//! sections by clicking on the corresponding tab headers.
//!
//! Tabs are useful for grouping related content and improving the user interface's
//! organization. Each tab can contain various types of widgets, allowing for a
//! flexible and dynamic layout.
//!
//! ## Headless Tabs
//!
//! [`Tab`] entities are arranged into groups using a controlling [`TabGroup`] component.
//! The [`TabGroup`] manages which tab is currently active and ensures that only the content of the active tab is visible at any given time.
//! Each [`Tab`] should be a child (or grandchild or...) if a [`TabGroup`]: when an [`Activate`] event
//! is sent to a [`Tab`] entity, we search upward to find the controlling [`TabGroup`].
//!
//! Underneath each [`Tab`] is a content entity, linked via the [`HasContent`]/[`ContentOfTab`] relationship.
//! When a tab is activated, its content is shown, while its sibling tabs' content is hidden.
//!
//! [`TabGroup`]s handles user interactions (e.g., clicking on a tab header) (via [`TabPlugin`]) and updates the content nodes accordingly.
//!
//! These components define "headless" widgets, meaning they provide the underlying
//! functionality without any specific styling or appearance. This allows developers to
//! customize the look and feel of the tabs according to their application's design requirements.
//!
//! ## Feathers-Styled Tabs
//!
//! The [`feathers`] module provides styled versions of tabs, using TODO.
//! These styled components come with predefined appearances that align with the Feathers UI
//! design language. They offer a convenient way to implement tabs with a consistent look and feel
//! while still allowing for some customization through theming and styling options.

use bevy::{
    app::{App, Plugin},
    ecs::prelude::*,
    log::warn,
    ui::{Display, Node},
    ui_widgets::Activate,
};

/// Marks a group of tabs, of which only one is visible at a time.
#[derive(Component, Default)]
pub struct TabGroup {
    /// The active tab in this group.
    //
    // This is not a relationship to not rely on `Commands`,
    // which would cause synchronization problems.
    active_tab: Option<Entity>,
}

impl TabGroup {
    /// Creates a new [`TabGroup`] with the given initial active tab.
    pub fn new(active_tab: Option<Entity>) -> Self {
        Self { active_tab }
    }

    /// Returns the currently active tab, if any.
    pub fn active_tab(&self) -> Option<Entity> {
        self.active_tab
    }
}

/// Marker component for a tab.
#[derive(Component)]
pub struct Tab;

/// Points to the content shown by a [`Tab`].
#[derive(Component)]
#[relationship(relationship_target = ContentOfTab)]
pub struct HasContent(pub Entity);

/// The collection of [`Tab`]s pointing to this content entity.
#[derive(Component)]
#[relationship_target(relationship = HasContent)]
pub struct ContentOfTab(Vec<Entity>);

/// Defines the [`Display`] mode to use when this tab content is active.
/// Defaults to [`Display::Flex`] if not present.
#[derive(Component)]
pub struct TabContentDisplayMode(pub Display);

/// Event triggered when a tab needs to be activated.
#[derive(Event, Clone, Debug)]
pub struct ActivateTab {
    /// The [`TabGroup`] entity that this event targets.
    pub group: Entity,
    /// The [`Tab`] entity that needs to be made visible.
    ///
    /// It must be a child of the `group` entity.
    pub tab: Entity,
}

/// Event triggered after a tab has been successfully activated.
///
/// Use this to update application state in response to UI changes.
#[derive(Event, Clone, Debug)]
pub struct TabActivated {
    /// The [`TabGroup`] entity.
    pub group: Entity,
    /// The [`Tab`] entity that was activated.
    pub tab: Entity,
}

/// Observes for [`Activate`]d tab buttons to trigger the [`ActivateTab`] event.
fn trigger_activate_tab_on_activate(
    on_activate: On<Activate>,
    child_of: Query<&ChildOf>,
    tabs: Query<&Tab>,
    tab_groups: Query<&TabGroup>,
    mut commands: Commands,
) {
    let clicked_entity = on_activate.entity;

    // Ensure the clicked entity is a Tab
    if !tabs.contains(clicked_entity) {
        return;
    }

    let Ok(mut current_child_of) = child_of.get(clicked_entity) else {
        warn!(
            "Clicked tab entity {clicked_entity} has no parent, and so cannot be inside a TabGroup"
        );
        return;
    };

    let mut tab_group_found = false;

    while !tab_group_found {
        let parent_entity = current_child_of.0;
        if let Ok(_tab_group) = tab_groups.get(parent_entity) {
            tab_group_found = true;
            commands.trigger(ActivateTab {
                group: parent_entity,
                tab: clicked_entity,
            });
        } else if let Ok(parent_child_of) = child_of.get(parent_entity) {
            // Continue traversing up the hierarchy
            current_child_of = parent_child_of
        } else {
            // Reached an entity that is not a tab and not a tab group, warn
            warn!(
                "Clicked tab entity {clicked_entity} is not inside a TabGroup (traversal stopped at {parent_entity})"
            );
            return;
        }
    }
}

/// Observes [`ActivateTab`] to switch which tab is currently active.
fn show_content_on_activate_tab(
    on_activate_tab: On<ActivateTab>,
    mut tab_groups: Query<&mut TabGroup>,
    contents: Query<(&HasContent, Option<&TabContentDisplayMode>), With<Tab>>,
    mut nodes: Query<&mut Node>,
    mut commands: Commands,
    parents: Query<&ChildOf>,
) {
    let event = on_activate_tab.event();
    let tab_group_entity = event.group;
    let tab_to_activate = event.tab;

    // Verify that the tab belongs to the group
    let mut is_descendant = false;
    let mut current_entity = tab_to_activate;
    while let Ok(child_of) = parents.get(current_entity) {
        if child_of.0 == tab_group_entity {
            is_descendant = true;
            break;
        }
        current_entity = child_of.0;
    }

    if !is_descendant {
        warn!(
            "Attempted to activate tab {:?} which is not a descendant of group {:?}",
            tab_to_activate, tab_group_entity
        );
        return;
    }

    if let Ok(mut tab_group) = tab_groups.get_mut(tab_group_entity) {
        if let Some(tab_to_deactivate) = tab_group.active_tab {
            // Do nothing if the active tab is clicked again, to avoid unnecessary UI updates.
            if tab_to_deactivate == tab_to_activate {
                return;
            }

            set_tab_content_display(&contents, &mut nodes, tab_to_deactivate, Display::None);
        }
        tab_group.active_tab = Some(tab_to_activate);
    } else {
        warn!("TabGroup entity {:?} not found", tab_group_entity);
    }

    let new_display_mode = if let Ok((_, Some(mode))) = contents.get(tab_to_activate) {
        mode.0
    } else {
        Display::Flex
    };

    set_tab_content_display(&contents, &mut nodes, tab_to_activate, new_display_mode);

    commands.trigger(TabActivated {
        group: tab_group_entity,
        tab: tab_to_activate,
    });
}

/// Helper to set the [`Display`] of a content [`Node`].
fn set_tab_content_display(
    contents: &Query<(&HasContent, Option<&TabContentDisplayMode>), With<Tab>>,
    nodes: &mut Query<&mut Node>,
    tab: Entity,
    display: Display,
) {
    if let Ok((content_entity, _)) = contents.get(tab) {
        if let Ok(mut node) = nodes.get_mut(content_entity.0) {
            node.display = display;
        } else {
            warn!(
                "Tab {:?} content entity {:?} missing Node component",
                tab, content_entity.0
            );
        }
    }
}

/// Plugin that adds logic and behavior for headless
/// [`TabGroup`] and [`Tab`] widgets.
pub struct TabPlugin;

impl Plugin for TabPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(trigger_activate_tab_on_activate)
            .add_observer(show_content_on_activate_tab);
    }
}

pub mod feathers {}

#[cfg(test)]
mod tests {
    use bevy::utils::default;

    use super::*;

    #[test]
    fn activate_tab_logic() {
        let mut app = App::new();
        app.add_plugins(TabPlugin);

        // Tab 1
        let content1_node = app
            .world_mut()
            .spawn(Node {
                display: Display::Flex,
                ..default()
            })
            .id();
        let tab1 = app.world_mut().spawn(Tab).id();
        app.world_mut()
            .entity_mut(tab1)
            .insert(HasContent(content1_node));

        // Tab 2
        let content2_node = app
            .world_mut()
            .spawn(Node {
                display: Display::None,
                ..default()
            })
            .id();
        let tab2 = app.world_mut().spawn(Tab).id();
        app.world_mut()
            .entity_mut(tab2)
            .insert(HasContent(content2_node));

        // Initial `TabGroup` state
        let tab_group_entity = app.world_mut().spawn(TabGroup::new(Some(tab1))).id();
        app.world_mut()
            .entity_mut(tab1)
            .insert(ChildOf(tab_group_entity));
        app.world_mut()
            .entity_mut(tab2)
            .insert(ChildOf(tab_group_entity));

        // Flush archetype updates so queries in observers work correctly
        app.update();

        // Verify tab activation works
        app.world_mut().trigger(ActivateTab {
            group: tab_group_entity,
            tab: tab2,
        });

        let node1 = app.world().get::<Node>(content1_node).unwrap();
        assert_eq!(node1.display, Display::None);
        let node2 = app.world().get::<Node>(content2_node).unwrap();
        assert_eq!(node2.display, Display::Flex);

        let tab_group = app.world().get::<TabGroup>(tab_group_entity).unwrap();
        assert_eq!(tab_group.active_tab(), Some(tab2));

        // Verify idempotency
        app.world_mut().trigger(ActivateTab {
            group: tab_group_entity,
            tab: tab2,
        });

        let tab_group = app.world().get::<TabGroup>(tab_group_entity).unwrap();
        assert_eq!(tab_group.active_tab(), Some(tab2));
        let node1 = app.world().get::<Node>(content1_node).unwrap();
        assert_eq!(node1.display, Display::None);
        let node2 = app.world().get::<Node>(content2_node).unwrap();
        assert_eq!(node2.display, Display::Flex);

        // Switch back to first tab
        app.world_mut().trigger(ActivateTab {
            group: tab_group_entity,
            tab: tab1,
        });

        let tab_group = app.world().get::<TabGroup>(tab_group_entity).unwrap();
        assert_eq!(tab_group.active_tab(), Some(tab1));
        let node1 = app.world().get::<Node>(content1_node).unwrap();
        assert_eq!(node1.display, Display::Flex);
        let node2 = app.world().get::<Node>(content2_node).unwrap();
        assert_eq!(node2.display, Display::None);
    }

    // Demonstrates that the bug which occurred when multiple `ActivateTab` events
    // were triggered in the same frame is now fixed.
    #[test]
    fn rapid_switching() {
        let mut app = App::new();
        app.add_plugins(TabPlugin);

        // Tab 1
        let content1_node = app
            .world_mut()
            .spawn(Node {
                display: Display::Flex,
                ..default()
            })
            .id();
        let tab1 = app.world_mut().spawn(Tab).id();
        app.world_mut()
            .entity_mut(tab1)
            .insert(HasContent(content1_node));

        // Tab 2
        let content2_node = app
            .world_mut()
            .spawn(Node {
                display: Display::None,
                ..default()
            })
            .id();
        let tab2 = app.world_mut().spawn(Tab).id();
        app.world_mut()
            .entity_mut(tab2)
            .insert(HasContent(content2_node));

        // Tab 3
        let content3_node = app
            .world_mut()
            .spawn(Node {
                display: Display::None,
                ..default()
            })
            .id();
        let tab3 = app.world_mut().spawn(Tab).id();
        app.world_mut()
            .entity_mut(tab3)
            .insert(HasContent(content3_node));

        // Initial `TabGroup` state
        let tab_group_entity = app.world_mut().spawn(TabGroup::new(Some(tab1))).id();
        app.world_mut()
            .entity_mut(tab1)
            .insert(ChildOf(tab_group_entity));
        app.world_mut()
            .entity_mut(tab2)
            .insert(ChildOf(tab_group_entity));
        app.world_mut()
            .entity_mut(tab3)
            .insert(ChildOf(tab_group_entity));

        // Flush archetype updates so queries in observers work correctly
        app.update();

        // Two consecutive `ActivateTab` trigger
        // to test resilience against rapid switching.
        app.world_mut().trigger(ActivateTab {
            group: tab_group_entity,
            tab: tab2,
        });
        app.world_mut().trigger(ActivateTab {
            group: tab_group_entity,
            tab: tab3,
        });
        app.update();

        let tab_group = app.world().get::<TabGroup>(tab_group_entity).unwrap();
        assert_eq!(
            tab_group.active_tab(),
            Some(tab3),
            "Should end up at content3 after rapid switch"
        );

        // Multiple contents shown simultaneously.
        let node3 = app.world().get::<Node>(content3_node).unwrap();
        assert_eq!(node3.display, Display::Flex, "Content 3 should be visible");
        let node2 = app.world().get::<Node>(content2_node).unwrap();
        assert_eq!(node2.display, Display::None, "Content 2 should be hidden");
        let node1 = app.world().get::<Node>(content1_node).unwrap();
        assert_eq!(node1.display, Display::None, "Content 1 should be hidden");
    }

    #[test]
    fn custom_display_mode() {
        let mut app = App::new();
        app.add_plugins(TabPlugin);

        // Tab with custom `Display::Grid` mode (Grid)
        let content_node = app
            .world_mut()
            .spawn(Node {
                display: Display::None,
                ..default()
            })
            .id();
        let tab = app.world_mut().spawn(Tab).id();
        app.world_mut().entity_mut(tab).insert((
            HasContent(content_node),
            TabContentDisplayMode(Display::Grid),
        ));

        // Create Group
        let tab_group_entity = app.world_mut().spawn(TabGroup::default()).id();
        app.world_mut()
            .entity_mut(tab)
            .insert(ChildOf(tab_group_entity));

        app.update();

        // Activate Tab
        app.world_mut().trigger(ActivateTab {
            group: tab_group_entity,
            tab,
        });

        // Verify content uses Grid instead of Flex
        let node = app.world().get::<Node>(content_node).unwrap();
        assert_eq!(
            node.display,
            Display::Grid,
            "Should use custom display mode (`Grid`)"
        );
    }

    #[test]
    fn activate_tab_wrong_group() {
        let mut app = App::new();
        app.add_plugins(TabPlugin);

        // Define `TabGroups`
        let group_a = app.world_mut().spawn(TabGroup::default()).id();
        let group_b = app.world_mut().spawn(TabGroup::default()).id();

        // Tab in Group A with content
        let content_node = app
            .world_mut()
            .spawn(Node {
                display: Display::None,
                ..default()
            })
            .id();
        let tab_in_a = app.world_mut().spawn(Tab).id();
        app.world_mut()
            .entity_mut(tab_in_a)
            .insert((HasContent(content_node), ChildOf(group_a)));
        app.update();

        // Attempt to activate `tab_in_a` using `group_b`
        app.world_mut().trigger(ActivateTab {
            group: group_b,
            tab: tab_in_a,
        });

        // Verify state remained unchanged
        let group_b_data = app.world().get::<TabGroup>(group_b).unwrap();
        assert_eq!(group_b_data.active_tab(), None);
        let node = app.world().get::<Node>(content_node).unwrap();
        assert_eq!(node.display, Display::None);
    }
}
