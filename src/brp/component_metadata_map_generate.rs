//! Handles a `component_metadata_map.generate` request coming from a client.
use bevy::{
    prelude::*,
    remote::{BrpError, BrpResult},
};
use serde_json::Value;

use crate::inspection::component_inspection::ComponentMetadataMap;

pub const METHOD: &str = "component_metadata_map.generate";

pub(crate) struct VerbPlugin;

impl Plugin for VerbPlugin {
    fn build(&self, app: &mut App) {
        let world = app.world_mut();
        super::register_remote_method(world, METHOD, process_remote_request);
    }
}

pub fn process_remote_request(In(_params): In<Option<Value>>, world: &World) -> BrpResult {
    let metadata_map = ComponentMetadataMap::generate(world);
    serde_json::to_value(metadata_map).map_err(BrpError::internal)
}
