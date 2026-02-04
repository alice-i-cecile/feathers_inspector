//! Organizes [`Node`]s into separate views, where only one is visible at a time.
// TODO: Write usage example (after integrating into `object_list.rs`).

// TODO: Reorganize relationships.

use bevy::prelude::*;

/// Root marker for the TabGroup widget.
#[derive(Component)]
pub struct TabGroup;

/// Content entity associated to a tab, made visible when the tab is activated.
#[derive(Component)]
#[relationship(relationship_target = AssociatedToTab)]
pub struct TabContent(pub Entity);

/// The tab entity that is associated to the content.
#[derive(Component, Default)]
#[relationship_target(relationship = TabContent)]
pub struct AssociatedToTab(Vec<Entity>);

/// The [`TabGroup`] entity that owns the tab.
#[derive(Component)]
#[relationship(relationship_target = HasTabs)]
pub struct BelongsToTabGroup(pub Entity);

/// The tab entities owned by the [`TabGroup`].
#[derive(Component, Default)]
#[relationship_target(relationship = BelongsToTabGroup)]
pub struct HasTabs(Vec<Entity>);

// TODO: Make it actually point to the tab, not the content.
/// Points to the active content in the [`TabGroup`].
#[derive(Component)]
#[relationship(relationship_target = ActiveTabOfGroup)]
pub struct ActiveTab(pub Entity);

/// Target component for [`ActiveTab`] relationship.
#[derive(Component)]
#[relationship_target(relationship = ActiveTab)]
pub struct ActiveTabOfGroup(Vec<Entity>);

/// Event triggered when a tab needs to be switched.
#[derive(Event, Clone, Debug)]
pub struct SwitchTab {
    /// The [`TabGroup`] that this event targets.
    pub tab_group: Entity,
    // TODO: Use the tab as target instead of the content.
    /// The content that needs to be made visible.
    pub target_content: Entity,
}

/// Observes for [`Click`]ed tab buttons to trigger the [`SwitchTab`] event.
fn trigger_switch_tab_on_click(
    on_click: On<Pointer<Click>>,
    relations: Query<(&TabContent, &BelongsToTabGroup)>,
    active_tabs: Query<&ActiveTab>,
    mut commands: Commands,
) {
    let clicked_entity = on_click.entity;
    let Ok((tab_content_entity, tab_group_entity)) = relations.get(clicked_entity) else {
        return;
    };

    if active_tabs
        .get(tab_group_entity.0)
        .is_ok_and(|active_tab| active_tab.0 == tab_content_entity.0)
    {
        return;
    }
    commands.trigger(SwitchTab {
        tab_group: tab_group_entity.0,
        target_content: tab_content_entity.0,
    });
}

/// Observes [`SwitchTab`] to switch which content is currently active.
fn switch_active_content_on_switch_tab(
    on_switch_tab: On<SwitchTab>,
    mut commands: Commands,
    active_tabs: Query<&ActiveTab>,
    mut nodes: Query<&mut Node>,
) {
    let event = on_switch_tab.event();
    let tab_group = event.tab_group;
    let content_to_activate = event.target_content;

    // FIXME: Iterate all tabs to avoid the command desync.
    if let Ok(active_tab) = active_tabs.get(tab_group) {
        let content_to_deactivate = active_tab.0;
        if content_to_deactivate == content_to_activate {
            return;
        }
        if let Ok(mut node) = nodes.get_mut(content_to_deactivate) {
            node.display = Display::None;
        }
    }
    if let Ok(mut node) = nodes.get_mut(content_to_activate) {
        node.display = Display::Grid;
    }
    commands
        .entity(tab_group)
        .insert(ActiveTab(content_to_activate));
}

/// Plugin that adds the [`TabGroup`] widget observers.
pub struct TabGroupPlugin;

impl Plugin for TabGroupPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(trigger_switch_tab_on_click)
            .add_observer(switch_active_content_on_switch_tab);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_switch_tab_logic() {
        let mut app = App::new();
        app.add_plugins(TabGroupPlugin);
        let content1 = app
            .world_mut()
            .spawn(Node {
                display: Display::Grid,
                ..default()
            })
            .id();
        let content2 = app
            .world_mut()
            .spawn(Node {
                display: Display::None,
                ..default()
            })
            .id();

        // Initial `TabGroup` state
        let tab_group = app.world_mut().spawn((TabGroup, ActiveTab(content1))).id();

        // Verify switching works
        app.world_mut().trigger(SwitchTab {
            tab_group,
            target_content: content2,
        });
        app.update();

        let active_tab = app.world().get::<ActiveTab>(tab_group).unwrap();
        assert_eq!(active_tab.0, content2);
        let node1 = app.world().get::<Node>(content1).unwrap();
        assert_eq!(node1.display, Display::None);
        let node2 = app.world().get::<Node>(content2).unwrap();
        assert_eq!(node2.display, Display::Grid);

        // Verify idempotency
        app.world_mut().trigger(SwitchTab {
            tab_group,
            target_content: content2,
        });
        app.update();

        let active_tab = app.world().get::<ActiveTab>(tab_group).unwrap();
        assert_eq!(active_tab.0, content2);
        let node1 = app.world().get::<Node>(content1).unwrap();
        assert_eq!(node1.display, Display::None);
        let node2 = app.world().get::<Node>(content2).unwrap();
        assert_eq!(node2.display, Display::Grid);

        // Switch back to first tab
        app.world_mut().trigger(SwitchTab {
            tab_group,
            target_content: content1,
        });
        app.update();

        let active_tab = app.world().get::<ActiveTab>(tab_group).unwrap();
        assert_eq!(active_tab.0, content1);
        let node1 = app.world().get::<Node>(content1).unwrap();
        assert_eq!(node1.display, Display::Grid);
        let node2 = app.world().get::<Node>(content2).unwrap();
        assert_eq!(node2.display, Display::None);
    }

    // Demonstrates a bug that occurs when multiple `SwitchTab` events
    // are triggered in the same frame.
    //
    // Because the `ActiveTab` relationship is updated via deferred `Commands`,
    // the second event reads the stale `ActiveTab` state
    // (it still sees the original tab as active, not the intermediate one).
    // As a result, the second event fails to hide the content activated by the first event,
    // resulting in multiple tabs being visible simultaneously.
    #[test]
    #[ignore = "Fails due to race condition in `ActiveTab` update"]
    fn rapid_switching() {
        let mut app = App::new();
        app.add_plugins(TabGroupPlugin);
        let content1 = app
            .world_mut()
            .spawn(Node {
                display: Display::Grid,
                ..default()
            })
            .id();
        let content2 = app
            .world_mut()
            .spawn(Node {
                display: Display::None,
                ..default()
            })
            .id();
        let content3 = app
            .world_mut()
            .spawn(Node {
                display: Display::None,
                ..default()
            })
            .id();

        // Initial `TabGroup` state
        let tab_group = app.world_mut().spawn((TabGroup, ActiveTab(content1))).id();

        // Two consecutive `SwitchTab` trigger
        // to test resilience against rapid switching.
        app.world_mut().trigger(SwitchTab {
            tab_group,
            target_content: content2,
        });
        app.world_mut().trigger(SwitchTab {
            tab_group,
            target_content: content3,
        });
        app.update();

        let active_tab = app.world().get::<ActiveTab>(tab_group).unwrap();
        assert_eq!(
            active_tab.0, content3,
            "Should end up at content3 after rapid switch"
        );

        // Multiple contents shown simultaneously.
        let node3 = app.world().get::<Node>(content3).unwrap();
        assert_eq!(node3.display, Display::Grid, "Content 3 should be visible");
        let node2 = app.world().get::<Node>(content2).unwrap();
        assert_eq!(node2.display, Display::None, "Content 2 should be hidden");
    }
}
