//! Organizes [`Node`]s into separate views, where only one is visible at a time.
// TODO: Write usage example (after integrating into `object_list.rs`).

// TODO: Create a `TabGroupPlugin`, with observers.

// TODO: Reorganize relationships.

use bevy::prelude::*;

/// Root marker for the TabGroup widget.
#[derive(Component)]
pub struct TabGroup;

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

/// Determines the visible tab in the [`TabGroup`].
#[derive(Component)]
#[relationship(relationship_target = ActiveTabOfGroup)]
pub struct CurrentTab(pub Entity);

/// Target component for [`ActiveTab`] relationship.
#[derive(Component)]
#[relationship_target(relationship = CurrentTab)]
pub struct ActiveTabOfGroup(Vec<Entity>);

/// Event triggered when a tab needs to be switched.
#[derive(Event, Clone, Debug)]
pub struct SwitchTab {
    /// The [`TabGroup`] that this event targets.
    pub tab_group: Entity,
    // TODO: Consider using the tab button as target instead of the panel.
    /// The [`TabPanel`] that needs to be made visible.
    pub target_panel: Entity,
}

/// Observes for [`Click`]ed tab buttons to trigger the [`SwitchTab`] event.
fn trigger_switch_tab_on_click(
    on_click: On<Pointer<Click>>,
    tab_trigger_query: Query<(&TabTrigger, &TabTriggerRoot)>,
    current_tabs: Query<&CurrentTab>,
    mut commands: Commands,
) {
    let clicked_button = on_click.entity;
    let Ok((tab_trigger, to_root)) = tab_trigger_query.get(clicked_button) else {
        return;
    };

    if current_tabs
        .get(to_root.0)
        .is_ok_and(|current_tab| current_tab.0 == tab_trigger.target)
    {
        return;
    }
    commands.trigger(SwitchTab {
        tab_group: to_root.0,
        target_panel: tab_trigger.target,
    });
}

/// Observes [`SwitchTab`] to switch which panel is currently active.
fn switch_active_panel_on_switch_tab(
    on_switch_tab: On<SwitchTab>,
    mut commands: Commands,
    current_tabs: Query<&CurrentTab>,
    mut nodes: Query<&mut Node>,
) {
    let event = on_switch_tab.event();
    let tab_group = event.tab_group;
    let panel_to_activate = event.target_panel;

    if let Ok(panel_to_deactivate) = current_tabs.get(tab_group) {
        let panel_to_deactivate = panel_to_deactivate.0;
        if panel_to_deactivate == panel_to_activate {
            return;
        }
        if let Ok(mut node) = nodes.get_mut(panel_to_deactivate) {
            node.display = Display::None;
        }
    }
    if let Ok(mut node) = nodes.get_mut(panel_to_activate) {
        node.display = Display::Flex;
    }
    commands
        .entity(tab_group)
        .insert(CurrentTab(panel_to_activate));
}
