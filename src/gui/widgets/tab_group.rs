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

    if let Ok(content_to_deactivate) = active_tabs.get(tab_group) {
        let content_to_deactivate = content_to_deactivate.0;
        if content_to_deactivate == content_to_activate {
            return;
        }
        if let Ok(mut node) = nodes.get_mut(content_to_deactivate) {
            node.display = Display::None;
        }
    }
    if let Ok(mut node) = nodes.get_mut(content_to_activate) {
        node.display = Display::Flex;
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
