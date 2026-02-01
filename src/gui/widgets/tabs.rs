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
//! The [`TabGroup`] entity manages a collection of [`Tab`] entities. Each tab entity should have two children:
//! first for a header and then for content area. The [`TabGroup`] handles user interactions, such as
//! switching between tabs and rendering the appropriate content based on the selected tab.
//!
//! These components represent "headless" widgets, meaning they provide the underlying
//! functionality without any specific styling or appearance. This allows developers to
//! customize the look and feel of the tabs according to their application's design requirements.
//!
//! This behavior is managed by the [`TabPlugin`], which registers the necessary systems and observers
//! to handle tab interactions and state management.
//!
//! ## Feathers-Styled Tabs
//!
//! The [`feathers`] module provides styled versions of tabs, using the .
//! These styled components come with predefined appearances that align with the Feathers UI
//! design language. They offer a convenient way to implement tabs with a consistent look and feel
//! while still allowing for some customization through theming and styling options.

use bevy::{
    app::{App, Plugin},
    ecs::prelude::*,
};

/// A component that marks an entity as a headless tab group.
///
/// Each [`TabGroup`] entity should have multiple [`Tab`] children,
/// each representing a different tab within the group.
///
/// The [`TabGroup`] component uses observers registered in the [`TabPlugin`] manages the state and behavior of the tabs,
/// including switching between tabs and rendering the appropriate content
/// based on the selected tab.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TabGroup {
    /// The [`Entity`] identifier of the currently selected tab.
    pub selected_tab: Entity,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tab;

/// A plugin that registers systems and observers for managing headless tabs.
pub struct TabPlugin;

impl Plugin for TabPlugin {
    fn build(&self, _app: &mut App) {}
}

pub mod feathers {}
