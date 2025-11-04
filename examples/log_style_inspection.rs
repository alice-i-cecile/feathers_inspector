//! Demonstrates how to inspect the Bevy world by logging entity details
//! in an ad-hoc manner using Commands.
//!
//! Analogous to `println!`-debugging, this pattern is useful for
//! quick debugging and inspection without setting up a full inspector UI.

use bevy::prelude::*;
use feathers_inspector::{
    component_inspection::{ComponentInspectionSettings, ComponentMetadataMap},
    entity_inspection::{EntityInspectionSettings, MultipleEntityInspectionSettings},
    entity_name_resolution::NameResolutionPlugin,
    extension_methods::{
        CommandsExtensionTrait, EntityCommandsInspectionTrait, WorldInspectionExtensionTrait,
    },
    resource_inspection::ResourceInspectionSettings,
    summary::{CommandsSummaryExt, SummarySettings},
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
                summarize_when_s_pressed,
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

    let instructions = "\
Output will be logged to the console: check your terminal window!

Press 'E' to inspect all Sprite entities
Press 'R' to inspect the AmbientLight resource
Press 'A' to inspect all resources
Press 'C' to inspect the Sprite component on all Sprite entities
Press `Space` to inspect all entities
Press 'S' to obtain summary statistics"
        .to_string();

    commands.spawn((
        Text::new(instructions),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
    ));
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
                .inspect(EntityInspectionSettings::default());
        }
    }
}

fn inspect_all_entities_when_space_pressed(
    world: &World,
    // Computing and storing the metadata for each component type can be expensive,
    // so we cache it across frames using a Local system parameter.
    mut metadata_map: Local<ComponentMetadataMap>,
) {
    if world
        .resource::<ButtonInput<KeyCode>>()
        .just_pressed(KeyCode::Space)
    {
        let mut entity_query = world.try_query::<Entity>().unwrap();
        let entities = entity_query.iter(world);

        let inspection_results = world.inspect_multiple(
            entities,
            MultipleEntityInspectionSettings::default(),
            &mut metadata_map,
        );

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

fn summarize_when_s_pressed(keyboard_input: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if keyboard_input.just_pressed(KeyCode::KeyS) {
        commands.summarize(SummarySettings::default());
    }
}
