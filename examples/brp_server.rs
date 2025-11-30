//! Demonstrates how to set up a BRP server
//! with registered custom `feathers_inspector` methods on a Bevy app
//! to allow out-of-process inspectors to inspect the Bevy app via BRP requests.
//!
//! Run this example with the `remote` feature enabled:
//! ```bash
//! cargo run --example brp_server --features="remote"
//! ```

use bevy::{
    prelude::*,
    remote::{RemotePlugin, http::RemoteHttpPlugin},
};
use feathers_inspector::{
    brp_methods::InspectorBrpPlugin, entity_name_resolution::NameResolutionPlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(NameResolutionPlugin)
        .add_plugins((
            RemotePlugin::default(),
            RemoteHttpPlugin::default(),
            InspectorBrpPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(Sprite {
        image: asset_server.load("ducky.png"),
        ..Default::default()
    });

    let instructions = "\
This is your Bevy app, where the BRP server runs.
Run the `brp_client` example on this machine to send BRP requests here."
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
