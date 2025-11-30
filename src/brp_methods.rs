//! Provides a plugin that adds custom BRP methods for this library.
//!
//! To remotely use [`World`] and [`Commands`] methods defined in this crate,
//! set up the BRP server in your Bevy app
//! according to [`bevy::remote`]'s documentation.
//! Then, register the custom methods by adding the [`InspectorBrpPlugin`].
//! Now you can send inspector requests via BRP to your app and get a response.
//!
//! Refer to the constants defined in this module
//! to understand the names of the registered methods.

use bevy::{
    prelude::*,
    remote::{BrpError, BrpResult, RemoteMethodSystemId, RemoteMethods},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const BRP_WORLD_INSPECT_METHOD: &str = "world.inspect";
pub const BRP_WORLD_INSPECT_CACHED_METHOD: &str = "world.inspect_cached";
pub const BRP_WORLD_INSPECT_MULTIPLE_METHOD: &str = "world.inspect_multiple";
pub const BRP_WORLD_INSPECT_COMPONENT_BY_ID_METHOD: &str = "world.inspect_component_by_id";
pub const BRP_WORLD_INSPECT_COMPONENT_METHOD: &str = "world.inspect_component";
pub const BRP_WORLD_INSPECT_RESOURCE_METHOD: &str = "world.inspect_resource";
pub const BRP_WORLD_INSPECT_RESOURCE_BY_ID_METHOD: &str = "world.inspect_resource_by_id";
pub const BRP_WORLD_INSPECT_ALL_RESOURCES_METHOD: &str = "world.inspect_all_resources";
pub const BRP_WORLD_INSPECT_COMPONENT_TYPE_METHOD: &str = "world.inspect_component_type";
pub const BRP_WORLD_INSPECT_COMPONENT_TYPE_BY_ID_METHOD: &str =
    "world.inspect_component_type_by_id";
pub const BRP_COMMANDS_INSPECT_RESOURCE_METHOD: &str = "commands.inspect_resource";
pub const BRP_COMMANDS_INSPECT_ALL_RESOURCES_METHOD: &str = "commands.inspect_all_resources";

/// Provides inspection methods defined in this crate
/// to be called via BRP.
///
/// ## Panics
///
/// This plugin assumes [`RemotePlugin`] is already added,
/// and will panic otherwise.
pub struct InspectorBrpPlugin;

impl Plugin for InspectorBrpPlugin {
    fn build(&self, app: &mut App) {
        let world = app.world_mut();

        let world_inspect_id = world.register_system(process_remote_world_inspect_request);
        let world_inspect_cached_id =
            world.register_system(process_remote_world_inspect_cached_request);
        let world_inspect_multiple_id =
            world.register_system(process_remote_world_inspect_multiple_request);
        let world_inspect_component_by_id_id =
            world.register_system(process_remote_world_inspect_component_by_id_request);
        let world_inspect_component_id =
            world.register_system(process_remote_world_inspect_component_request);
        let world_inspect_resource_id =
            world.register_system(process_remote_world_inspect_resource_request);
        let world_inspect_resource_by_id_id =
            world.register_system(process_remote_world_inspect_resource_by_id_request);
        let world_inspect_all_resources_id =
            world.register_system(process_remote_world_inspect_all_resources_request);
        let world_inspect_component_type_id =
            world.register_system(process_remote_world_inspect_component_type_request);
        let world_inspect_component_type_by_id_id =
            world.register_system(process_remote_world_inspect_component_type_by_id_request);
        let commands_inspect_resource_id =
            world.register_system(process_remote_commands_inspect_resource_request);
        let commands_inspect_all_resources_id =
            world.register_system(process_remote_commands_inspect_all_resources_request);

        // Avoids adding `RemotePlugin` by design,
        // since users might also want to add it themselves for other purposes.
        let mut remote_methods = world
            .get_resource_mut::<RemoteMethods>()
            .expect("`RemotePlugin` must be present");

        remote_methods.insert(
            BRP_WORLD_INSPECT_METHOD,
            RemoteMethodSystemId::Instant(world_inspect_id),
        );
        remote_methods.insert(
            BRP_WORLD_INSPECT_CACHED_METHOD,
            RemoteMethodSystemId::Instant(world_inspect_cached_id),
        );
        remote_methods.insert(
            BRP_WORLD_INSPECT_MULTIPLE_METHOD,
            RemoteMethodSystemId::Instant(world_inspect_multiple_id),
        );
        remote_methods.insert(
            BRP_WORLD_INSPECT_COMPONENT_BY_ID_METHOD,
            RemoteMethodSystemId::Instant(world_inspect_component_by_id_id),
        );
        remote_methods.insert(
            BRP_WORLD_INSPECT_COMPONENT_METHOD,
            RemoteMethodSystemId::Instant(world_inspect_component_id),
        );
        remote_methods.insert(
            BRP_WORLD_INSPECT_RESOURCE_METHOD,
            RemoteMethodSystemId::Instant(world_inspect_resource_id),
        );
        remote_methods.insert(
            BRP_WORLD_INSPECT_RESOURCE_BY_ID_METHOD,
            RemoteMethodSystemId::Instant(world_inspect_resource_by_id_id),
        );
        remote_methods.insert(
            BRP_WORLD_INSPECT_ALL_RESOURCES_METHOD,
            RemoteMethodSystemId::Instant(world_inspect_all_resources_id),
        );
        remote_methods.insert(
            BRP_WORLD_INSPECT_COMPONENT_TYPE_METHOD,
            RemoteMethodSystemId::Instant(world_inspect_component_type_id),
        );
        remote_methods.insert(
            BRP_WORLD_INSPECT_COMPONENT_TYPE_BY_ID_METHOD,
            RemoteMethodSystemId::Instant(world_inspect_component_type_by_id_id),
        );
        remote_methods.insert(
            BRP_COMMANDS_INSPECT_RESOURCE_METHOD,
            RemoteMethodSystemId::Instant(commands_inspect_resource_id),
        );
        remote_methods.insert(
            BRP_COMMANDS_INSPECT_ALL_RESOURCES_METHOD,
            RemoteMethodSystemId::Instant(commands_inspect_all_resources_id),
        );
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BrpWorldInspectParams;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BrpWorldInspectResponse;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BrpWorldInspectCachedParams;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BrpWorldInspectCachedResponse;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BrpWorldInspectMultipleParams;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BrpWorldInspectMultipleResponse;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BrpWorldInspectComponentByIdParams;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BrpWorldInspectComponentByIdResponse;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BrpWorldInspectComponentParams;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BrpWorldInspectComponentResponse;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BrpWorldInspectResourceParams;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BrpWorldInspectResourceResponse;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BrpWorldInspectResourceByIdParams;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BrpWorldInspectResourceByIdResponse;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BrpWorldInspectAllResourcesParams;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BrpWorldInspectAllResourcesResponse;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BrpWorldInspectComponentTypeParams;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BrpWorldInspectComponentTypeResponse;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BrpWorldInspectComponentTypeByIdParams;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BrpWorldInspectComponentTypeByIdResponse;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BrpCommandsInspectResourceParams;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BrpCommandsInspectResourceResponse;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BrpCommandsInspectAllResourcesParams;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BrpCommandsInspectAllResourcesResponse;

/// Handles a `world.inspect` request coming from a client.
pub fn process_remote_world_inspect_request(
    In(_params): In<Option<Value>>,
    _world: &World,
) -> BrpResult {
    let response = "called `world.inspect` handler successfully.";
    serde_json::to_value(response).map_err(BrpError::internal)
}

/// Handles a `world.inspect_cached` request coming from a client.
pub fn process_remote_world_inspect_cached_request(
    In(_params): In<Option<Value>>,
    _world: &World,
) -> BrpResult {
    let response = "called `world.inspect_cached` handler successfully.";
    serde_json::to_value(response).map_err(BrpError::internal)
}

/// Handles a `world.inspect_multiple` request coming from a client.
pub fn process_remote_world_inspect_multiple_request(
    In(_params): In<Option<Value>>,
    _world: &World,
) -> BrpResult {
    let response = "called `world.inspect_multiple` handler successfully.";
    serde_json::to_value(response).map_err(BrpError::internal)
}

/// Handles a `world.inspect_component_by_id` request coming from a client.
pub fn process_remote_world_inspect_component_by_id_request(
    In(_params): In<Option<Value>>,
    _world: &World,
) -> BrpResult {
    let response = "called `world.inspect_component_by_id` handler successfully.";
    serde_json::to_value(response).map_err(BrpError::internal)
}

/// Handles a `world.inspect_component` request coming from a client.
pub fn process_remote_world_inspect_component_request(
    In(_params): In<Option<Value>>,
    _world: &World,
) -> BrpResult {
    let response = "called `world.inspect_component` handler successfully.";
    serde_json::to_value(response).map_err(BrpError::internal)
}

/// Handles a `world.inspect_resource` request coming from a client.
pub fn process_remote_world_inspect_resource_request(
    In(_params): In<Option<Value>>,
    _world: &World,
) -> BrpResult {
    let response = "called `world.inspect_resource` handler successfully.";
    serde_json::to_value(response).map_err(BrpError::internal)
}

/// Handles a `world.inspect_resource_by_id` request coming from a client.
pub fn process_remote_world_inspect_resource_by_id_request(
    In(_params): In<Option<Value>>,
    _world: &World,
) -> BrpResult {
    let response = "called `world.inspect_resource_by_id` handler successfully.";
    serde_json::to_value(response).map_err(BrpError::internal)
}

/// Handles a `world.inspect_all_resources` request coming from a client.
pub fn process_remote_world_inspect_all_resources_request(
    In(_params): In<Option<Value>>,
    _world: &World,
) -> BrpResult {
    let response = "called `world.inspect_all_resources` handler successfully.";
    serde_json::to_value(response).map_err(BrpError::internal)
}

/// Handles a `world.inspect_component_type` request coming from a client.
pub fn process_remote_world_inspect_component_type_request(
    In(_params): In<Option<Value>>,
    _world: &World,
) -> BrpResult {
    let response = "called `world.inspect_component_type` handler successfully.";
    serde_json::to_value(response).map_err(BrpError::internal)
}

/// Handles a `world.inspect_component_type_by_id` request coming from a client.
pub fn process_remote_world_inspect_component_type_by_id_request(
    In(_params): In<Option<Value>>,
    _world: &World,
) -> BrpResult {
    let response = "called `world.inspect_component_type_by_id` handler successfully.";
    serde_json::to_value(response).map_err(BrpError::internal)
}

/// Handles a `commands.inspect_resource` request coming from a client.
pub fn process_remote_commands_inspect_resource_request(
    In(_params): In<Option<Value>>,
    mut _commands: Commands,
) -> BrpResult {
    let response = "called `commands.inspect_resource` handler successfully.";
    serde_json::to_value(response).map_err(BrpError::internal)
}

/// Handles a `commands.inspect_all_resources` request coming from a client.
pub fn process_remote_commands_inspect_all_resources_request(
    In(_params): In<Option<Value>>,
    mut _commands: Commands,
) -> BrpResult {
    let response = "called `commands.inspect_all_resources` handler successfully.";
    serde_json::to_value(response).map_err(BrpError::internal)
}
