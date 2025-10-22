use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use feathers_inspector::InspectExtensionCommandsTrait;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            // Wasm builds will check for meta files (that don't exist) if this isn't set.
            // This causes errors and even panics in web builds on itch.
            // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, inspect_entity_when_space_pressed)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(Sprite {
        image: asset_server.load("ducky.png"),
        ..Default::default()
    });
}

fn inspect_entity_when_space_pressed(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    entities: Query<Entity>,
    mut commands: Commands,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for entity in entities.iter() {
            commands.entity(entity).inspect();
        }
    }
}
