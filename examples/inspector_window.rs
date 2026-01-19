//! Demonstrates the Feathers Inspector window UI.
//!
//! This example shows how to use the inspector window to browse entities
//! and resources in a separate window with a graphical interface.

use bevy::prelude::*;
use feathers_inspector::{
    InspectorWindowPlugin,
    entity_name_resolution::{NameResolutionPlugin, NameResolutionRegistry},
};

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        // NOTE: will not be required once this crate is upstreamed
        .add_plugins(NameResolutionPlugin)
        // Add the inspector window plugin
        .add_plugins(InspectorWindowPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, fluctuating_entity_counts);

    // We can register our own component types to be used for name resolution
    // A priority of zero means this takes precedence over most engine-provided types,
    // and is generally a good fit for user-defined naming components.
    let mut name_registry = app.world_mut().resource_mut::<NameResolutionRegistry>();
    name_registry.register_name_defining_type::<Chaff>(0);

    app.run();
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

/// Marker component for entities that should be
/// spawned and despawned dynamically in [`fluctuating_entity_counts`].
#[derive(Component)]
struct Chaff;

/// Spawns and despawns entities to demonstrate dynamic updates in the inspector
fn fluctuating_entity_counts(
    mut commands: Commands,
    chaff_query: Query<Entity, With<Chaff>>,
    time: Res<Time>,
) {
    const MAX_CHAFF_COUNT: usize = 25;
    let chaff_count = chaff_query.iter().count();
    let sinusoid = (time.elapsed_secs().sin() + 1.0) / 2.0; // Normalize to [0, 1]
    let chaff_desired = (sinusoid * MAX_CHAFF_COUNT as f32) as usize;

    if chaff_count < chaff_desired {
        commands.spawn(Chaff);
    } else if chaff_count > chaff_desired {
        if let Some(marked_for_death) = chaff_query.iter().next() {
            commands.entity(marked_for_death).despawn();
        }
    }
}
