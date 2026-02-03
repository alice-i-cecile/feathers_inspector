//! Organizes [`Node`]s into separate views, where only one is visible at a time.

// TODO: Create a `TabGroupPlugin`, with observers.

use bevy::ecs::event::EntityEvent;
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

/// Determines to which [`TabGroup`] the panel belongs to.
#[derive(Component)]
#[relationship(relationship_target = HasTab)]
pub struct BelongsToTabGroup(pub Entity);

/// Target component for [`BelongsToTabGroup`] relationship.
#[derive(Component)]
#[relationship_target(relationship = BelongsToTabGroup)]
pub struct HasTab(Vec<Entity>);

/// Determines the visible tab in the [`TabGroup`].
#[derive(Component)]
#[relationship(relationship_target = ActiveInTabGroup)]
pub struct ActiveTab(pub Entity);

/// Target component for [`ActiveTab`] relationship.
#[derive(Component)]
#[relationship_target(relationship = ActiveTab)]
pub struct ActiveInTabGroup(Vec<Entity>);

/// Event triggered when a tab needs to be switched.
// TODO: Make it a plain `Event`.
#[derive(EntityEvent, Clone, Debug)]
pub struct SwitchTab {
    /// The [`TabGroup`] that this event targets.
    #[event_target]
    pub tab_group: Entity,
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

    // TODO: Instead of blindly triggering, check if tab is already active.
    commands.trigger(SwitchTab {
        tab_group: to_root.0,
        panel: tab_trigger.target,
    });
}

/// Observes [`SwitchTab`] to update panel visibility.
fn switch_active_panel_on_switch_tab(
    on_switch_tab: On<SwitchTab>,
    mut commands: Commands,
    mut active_tab: Query<&mut Node, With<ActiveTab>>,
    mut inactive_tabs: Query<&mut Node, Without<ActiveTab>>,
) {
    let event = on_switch_tab.event();
    let tab_group = event.tab_group;
    let panel_to_activate = event.panel;

    if let Err(e) = active_tab.single_mut().and_then(|mut node| {
        node.display = Display::None;
        Ok(())
    }) {
        warn!("Unable to find tab do deactivate: {e}");
    }
    if let Err(e) = inactive_tabs.get_mut(event.panel).and_then(|mut node| {
        node.display = Display::Flex;
        Ok(())
    }) {
        warn!("Unable to find tab to activate: {e}");
    }
    commands
        .entity(tab_group)
        .insert(ActiveTab(panel_to_activate));
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
