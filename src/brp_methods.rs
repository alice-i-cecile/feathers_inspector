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
pub const BRP_WORLD_INSPECT_RESOURCE_BY_ID_METHOD: &str = "world.inspect_resource_by_id";
pub const BRP_WORLD_INSPECT_ALL_RESOURCES_METHOD: &str = "world.inspect_all_resources";
pub const BRP_WORLD_INSPECT_COMPONENT_TYPE_BY_ID_METHOD: &str =
    "world.inspect_component_type_by_id";

/// Provides inspection methods defined in this crate
/// to be called via BRP.
///
/// ## Panics
///
/// This plugin assumes [`RemotePlugin`] is already added,
/// and will panic otherwise.
///
/// [`RemotePlugin`]: bevy::remote::RemotePlugin
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
        let world_inspect_resource_by_id_id =
            world.register_system(process_remote_world_inspect_resource_by_id_request);
        let world_inspect_all_resources_id =
            world.register_system(process_remote_world_inspect_all_resources_request);
        let world_inspect_component_type_by_id_id =
            world.register_system(process_remote_world_inspect_component_type_by_id_request);

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
            BRP_WORLD_INSPECT_RESOURCE_BY_ID_METHOD,
            RemoteMethodSystemId::Instant(world_inspect_resource_by_id_id),
        );
        remote_methods.insert(
            BRP_WORLD_INSPECT_ALL_RESOURCES_METHOD,
            RemoteMethodSystemId::Instant(world_inspect_all_resources_id),
        );
        remote_methods.insert(
            BRP_WORLD_INSPECT_COMPONENT_TYPE_BY_ID_METHOD,
            RemoteMethodSystemId::Instant(world_inspect_component_type_by_id_id),
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
pub struct BrpWorldInspectResourceByIdParams;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BrpWorldInspectResourceByIdResponse;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BrpWorldInspectAllResourcesParams;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BrpWorldInspectAllResourcesResponse;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BrpWorldInspectComponentTypeByIdParams;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BrpWorldInspectComponentTypeByIdResponse;

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

/// Handles a `world.inspect_component_type_by_id` request coming from a client.
pub fn process_remote_world_inspect_component_type_by_id_request(
    In(_params): In<Option<Value>>,
    _world: &World,
) -> BrpResult {
    let response = "called `world.inspect_component_type_by_id` handler successfully.";
    serde_json::to_value(response).map_err(BrpError::internal)
}
