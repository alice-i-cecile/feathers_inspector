//! Handles a `world.inspect_all_resources` request coming from a client.
use bevy::{
    prelude::*,
    remote::{BrpError, BrpResult, builtin_methods::parse_some},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    extension_methods::WorldInspectionExtensionTrait,
    inspection::resource_inspection::ResourceInspectionSettings,
};

pub const METHOD: &str = "world.inspect_all_resources";

pub(crate) struct VerbPlugin;

impl Plugin for VerbPlugin {
    fn build(&self, app: &mut App) {
        let world = app.world_mut();
        super::register_remote_method(world, METHOD, process_remote_request);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Params {
    pub settings: ResourceInspectionSettings,
}

pub fn process_remote_request(In(params): In<Option<Value>>, world: &World) -> BrpResult {
    let Params { settings } = parse_some(params)?;
    let inspection = world.inspect_all_resources(settings);
    serde_json::to_value(inspection).map_err(BrpError::internal)
}
