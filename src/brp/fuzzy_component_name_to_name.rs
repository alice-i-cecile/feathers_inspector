//! Handles a `world.fuzzy_component_name_to_name` request coming from a client.
use bevy::{
    prelude::*,
    remote::{BrpResult, builtin_methods::parse_some},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    brp::fuzzy_id_to_name_result,
    entity_name_resolution::fuzzy_name_mapping::fuzzy_component_name_to_id,
    inspection::component_inspection::ComponentMetadataMap,
};

pub const METHOD: &str = "world.fuzzy_component_name_to_name";

pub(crate) struct VerbPlugin;

impl Plugin for VerbPlugin {
    fn build(&self, app: &mut App) {
        let world = app.world_mut();
        super::register_remote_method(world, METHOD, process_remote_request);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Params {
    pub fuzzy_name: String,
    pub metadata_map: Option<ComponentMetadataMap>,
}

pub fn process_remote_request(In(params): In<Option<Value>>, world: &World) -> BrpResult {
    let Params {
        fuzzy_name,
        metadata_map,
    } = parse_some(params)?;
    let matched_id = fuzzy_component_name_to_id(world, &fuzzy_name);
    fuzzy_id_to_name_result(world, &fuzzy_name, matched_id, metadata_map)
}
