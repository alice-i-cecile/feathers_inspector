//! Demonstrates how to inspect the Bevy world by logging entity details
//! in an ad-hoc manner using Commands.
//!
//! Analogous to `println!`-debugging, this pattern is useful for
//! quick debugging and inspection without setting up a full inspector UI.

use bevy::prelude::*;
use feathers_inspector::{
    component_inspection::ComponentInspectionSettings,
    entity_inspection::{EntityInspectExtensionTrait, InspectExtensionCommandsTrait},
    name_resolution::NameResolutionPlugin,
    resource_inspection::{ResourceInspectExtensionCommandsTrait, ResourceInspectionSettings},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // NOTE: will not be required once this crate is upstreamed
        .add_plugins(NameResolutionPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                inspect_sprite_entities_when_e_pressed,
                inspect_resource_when_r_pressed,
                inspect_all_resources_when_a_pressed,
                inspect_specific_component_when_c_pressed,
                inspect_all_entities_when_space_pressed,
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

fn inspect_sprite_entities_when_e_pressed(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    entities: Query<Entity, With<Sprite>>,
    mut commands: Commands,
) {
    if keyboard_input.just_pressed(KeyCode::KeyE) {
        for entity in entities.iter() {
            commands
                .entity(entity)
                .inspect(ComponentInspectionSettings::default());
        }
    }
}

fn inspect_all_entities_when_space_pressed(world: &World) {
    if world
        .resource::<ButtonInput<KeyCode>>()
        .just_pressed(KeyCode::Space)
    {
        let mut entity_query = world.try_query::<Entity>().unwrap();
        let entities = entity_query.iter(world);

        let inspection_results =
            world.inspect_multiple(entities, ComponentInspectionSettings::default());

        for inspection in inspection_results {
            match inspection {
                Ok(inspection) => info!("{inspection}"),
                Err(err) => info!("Failed to inspect an entity: {err}"),
            }
        }
    }
}

fn inspect_resource_when_r_pressed(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        commands.inspect_resource::<AmbientLight>(ResourceInspectionSettings {
            full_type_names: true,
        });
    }
}

fn inspect_all_resources_when_a_pressed(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    if keyboard_input.just_pressed(KeyCode::KeyA) {
        commands.inspect_all_resources(ResourceInspectionSettings::default());
    }
}

fn inspect_specific_component_when_c_pressed(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query: Query<Entity, With<Sprite>>,
    mut commands: Commands,
) {
    if keyboard_input.just_pressed(KeyCode::KeyC) {
        for entity in query.iter() {
            commands
                .entity(entity)
                .inspect_component::<Sprite>(ComponentInspectionSettings::default());
        }
    }
}
