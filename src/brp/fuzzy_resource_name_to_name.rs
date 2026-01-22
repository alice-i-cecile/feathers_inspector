//! Handles a `world.fuzzy_resource_name_to_name` request coming from a client.
use bevy::{
    prelude::*,
    remote::{BrpError, BrpResult, builtin_methods::parse_some},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    fuzzy_name_mapping::fuzzy_resource_name_to_id,
    inspection::component_inspection::ComponentMetadataMap,
};

pub const METHOD: &str = "world.fuzzy_resource_name_to_name";

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
    match fuzzy_resource_name_to_id(world, &fuzzy_name) {
        Some(component_id) => {
            let metadata_map = metadata_map.unwrap_or(ComponentMetadataMap::generate(world));
            let component_metadata = metadata_map.get(&component_id);
            let Some(component_metadata) = component_metadata else {
                let index = component_id.index();
                return Err(BrpError::component_error(format!(
                    "Could not find metadata for component `{index}`"
                )));
            };
            Ok(serde_json::to_value(component_metadata.name.to_string())
                .map_err(BrpError::internal)?)
        }
        None => Err(super::no_fuzzy_name_candidates_brp_error(&fuzzy_name)),
    }
}
