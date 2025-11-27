//! Configuration constants for the inspector UI.

use bevy::prelude::*;
use bevy::ui::Val;

/// Configuration for inspector UI layout and styling.
#[derive(Resource, Clone)]
pub struct InspectorConfig {
    // Layout
    /// Width of the left panel (entity list).
    pub left_panel_width: Val,
    /// Height of the title bar.
    pub title_bar_height: Val,
    /// Height of the tab bar.
    pub tab_bar_height: Val,

    // Spacing
    /// Padding inside panels.
    pub panel_padding: UiRect,
    /// Gap between items in lists.
    pub item_gap: Val,
    /// Gap between columns.
    pub column_gap: Val,

    // Typography
    /// Font size for titles.
    pub title_font_size: f32,
    /// Font size for body text.
    pub body_font_size: f32,
    /// Font size for small/secondary text.
    pub small_font_size: f32,

    // Colors (for non-themed elements)
    /// Border color.
    pub border_color: Color,
    /// Muted text color.
    pub muted_text_color: Color,
    /// Error text color.
    pub error_text_color: Color,
}

impl Default for InspectorConfig {
    fn default() -> Self {
        Self {
            // Layout
            left_panel_width: Val::Percent(30.0),
            title_bar_height: Val::Px(40.0),
            tab_bar_height: Val::Px(36.0),

            // Spacing
            panel_padding: UiRect::all(Val::Px(8.0)),
            item_gap: Val::Px(4.0),
            column_gap: Val::Px(8.0),

            // Typography
            title_font_size: 16.0,
            body_font_size: 13.0,
            small_font_size: 11.0,

            // Colors
            border_color: Color::srgba(0.3, 0.3, 0.3, 1.0),
            muted_text_color: Color::srgba(0.6, 0.6, 0.6, 1.0),
            error_text_color: Color::srgba(0.8, 0.3, 0.3, 1.0),
        }
    }
}
