//! Demonstrates the Feathers Inspector window UI.
//!
//! This example shows how to use the inspector window to browse entities
//! and resources in a separate window with a graphical interface.

use bevy::prelude::*;
use feathers_inspector::{InspectorWindowPlugin, entity_name_resolution::NameResolutionPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // NOTE: will not be required once this crate is upstreamed
        .add_plugins(NameResolutionPlugin)
        // Add the inspector window plugin
        .add_plugins(InspectorWindowPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn a camera
    commands.spawn(Camera2d);

    // Spawn a parent entity with children to demonstrate relationships
    commands
        .spawn((
            Sprite {
                image: asset_server.load("ducky.png"),
                ..Default::default()
            },
            Name::new("Parent Ducky"),
        ))
        .with_children(|parent| {
            parent.spawn((
                Sprite {
                    color: Color::srgb(1.0, 0.0, 0.0),
                    custom_size: Some(Vec2::new(30.0, 30.0)),
                    ..Default::default()
                },
                Transform::from_xyz(50.0, 0.0, 0.0),
                Name::new("Child Red"),
            ));

            parent.spawn((
                Sprite {
                    color: Color::srgb(0.0, 0.0, 1.0),
                    custom_size: Some(Vec2::new(30.0, 30.0)),
                    ..Default::default()
                },
                Transform::from_xyz(-50.0, 0.0, 0.0),
                Name::new("Child Blue"),
            ));
        });

    // Spawn another standalone entity
    commands.spawn((
        Sprite {
            color: Color::srgb(0.0, 1.0, 0.0),
            custom_size: Some(Vec2::new(50.0, 50.0)),
            ..Default::default()
        },
        Transform::from_xyz(-150.0, 0.0, 0.0),
        Name::new("Standalone Green"),
    ));

    // Spawn an entity without a name
    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 1.0, 0.0),
            custom_size: Some(Vec2::new(40.0, 40.0)),
            ..Default::default()
        },
        Transform::from_xyz(150.0, 0.0, 0.0),
    ));

    // Add instructions on the main window
    let instructions = "\
Check the Inspector Window!

The inspector window shows:
- Entity list with component counts and memory usage
- Components tab with reflected values
- Relationships tab showing parent/child hierarchy
- Click entities in the Relationships tab to navigate"
        .to_string();

    commands.spawn((
        Text::new(instructions),
        Node {
            position_type: PositionType::Absolute,
            top: px(12.0),
            left: px(12.0),
            ..default()
        },
        TextFont {
            font_size: 16.0,
            ..default()
        },
    ));
}

fn px(value: f32) -> Val {
    Val::Px(value)
}
