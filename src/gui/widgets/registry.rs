use bevy::prelude::*;
use bevy::reflect::enums::Enum;
use bevy::ui::Val::*;
use bevy::{
    feathers::{
        theme::{ThemeFontColor, ThemedText},
        tokens,
    },
    platform::collections::HashMap,
};
use core::any::TypeId;

use crate::gui::{config::InspectorConfig, widgets::FieldPath};

use super::reflected::PartialReflectWidget;

/// Type-erasing function that places a widget [`Bundle`] into an empty entity
pub struct WidgetBuilder(Box<dyn FnOnce(&mut EntityWorldMut<'_>) + 'static>);

impl WidgetBuilder {
    pub fn new<B: Bundle>(widget: B) -> Self {
        Self(Box::new(move |entity: &'_ mut EntityWorldMut<'_>| {
            entity.insert(widget);
        }))
    }

    pub fn apply_widget(self, entity: &mut EntityWorldMut<'_>) {
        self.0(entity)
    }
}

/// Type-erasing function that initializes the [`Bundle`] for a given widget
type WidgetCreator = Box<
    dyn Fn(&dyn PartialReflect, &FieldPath, &InspectorConfig) -> Option<WidgetBuilder>
        + Sync
        + Send
        + 'static,
>;

/// Registry storing the widget implementations for types
#[derive(Resource, Default)]
pub struct WidgetRegistry {
    builders: HashMap<TypeId, WidgetCreator>,
}

impl WidgetRegistry {
    pub fn with<T: PartialReflectWidget>(mut self) -> Self {
        self.add::<T>();
        self
    }

    /// Register a type that implements a widget
    pub fn add<T: PartialReflectWidget>(&mut self) {
        self.builders
            .insert(TypeId::of::<T>(), Box::new(Self::builder_for::<T>));
    }

    /// Register or override a type with a widget and bypass the [`PartialReflectWidget`] trait
    pub fn add_custom<T: Reflect>(&mut self, builder: WidgetCreator) {
        self.builders
            .insert(TypeId::of::<T>(), builder);
    }

    /// get a widget builder for this type if is registered
    pub fn get_widget(
        &self,
        t: &dyn PartialReflect,
        field_path: &FieldPath,
        config: &InspectorConfig,
    ) -> Option<WidgetBuilder> {
        let type_id = t.get_represented_type_info().map(|info| info.type_id());
        type_id
            .and_then(|type_id| self.builders.get(&type_id))
            .and_then(|b| b(t, field_path, config))
    }

    /// get a label pseudo-widget
    pub fn label_widget(&self, label: String, config: &InspectorConfig) -> WidgetBuilder {
        WidgetBuilder::new((
            Text::new(label),
            TextFont {
                font_size: FontSize::Px(config.small_font_size),
                ..default()
            },
            ThemeFontColor(tokens::TEXT_DIM),
            ThemedText,
            TextColor(config.muted_text_color),
        ))
    }

    /// get an enum variant widget for this type
    /// todo: mutation of the variant with this widget
    pub fn enum_widget(
        &self,
        type_name: &str,
        t: &dyn Enum,
        config: &InspectorConfig,
    ) -> WidgetBuilder {
        WidgetBuilder::new((
            Node {
                min_width: Px(60.0),
                padding: UiRect::horizontal(Px(4.0)),
                border: UiRect::all(Px(1.0)),
                ..default()
            },
            BorderColor::all(Color::srgba(0.3, 0.3, 0.3, 1.0)),
            BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 1.0)),
            Children::spawn_one((
                Text::new(format!("{type_name}::{}", t.variant_name())),
                TextFont {
                    font_size: FontSize::Px(config.small_font_size),
                    ..default()
                },
            )),
        ))
    }

    /// function enclosing the creation of a builder 
    fn builder_for<T: PartialReflectWidget>(
        t: &dyn PartialReflect,
        field_path: &FieldPath,
        config: &InspectorConfig,
    ) -> Option<WidgetBuilder> {
        let widget = <T as PartialReflectWidget>::try_widget(t, field_path, config)?;
        Some(WidgetBuilder::new(widget))
    }
}
