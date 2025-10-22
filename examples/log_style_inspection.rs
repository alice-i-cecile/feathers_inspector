//! Demonstrates how to inspect the Bevy world by logging entity details
//! in an ad-hoc manner using Commands.
//!
//! Analogous to `println!`-debugging, this pattern is useful for
//! quick debugging and inspection without setting up a full inspector UI.

use bevy::prelude::*;
use feathers_inspector::{
    entity_inspection::InspectExtensionCommandsTrait,
    resource_inspection::ResourceInspectExtensionCommandsTrait,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                inspect_entities_when_e_pressed,
                inspect_resource_when_r_pressed,
            ),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(Sprite {
        image: asset_server.load("ducky.png"),
        ..Default::default()
    });
}

fn inspect_entities_when_e_pressed(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    entities: Query<Entity>,
    mut commands: Commands,
) {
    if keyboard_input.just_pressed(KeyCode::KeyE) {
        for entity in entities.iter() {
            commands.entity(entity).inspect();
        }
    }
}

fn inspect_resource_when_r_pressed(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        commands.inspect_resource::<AmbientLight>();
    }
}
