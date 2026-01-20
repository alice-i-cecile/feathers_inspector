//! Handles a `world.inspect_resource` request coming from a client.
use bevy::{
    prelude::*,
    remote::{BrpError, BrpResult, builtin_methods::parse_some},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    component_inspection::ComponentMetadataMap,
    extension_methods::WorldInspectionExtensionTrait,
    resource_inspection::{ResourceInspectionError, ResourceInspectionSettings},
};

pub const METHOD: &str = "world.inspect_resource";

pub(crate) struct VerbPlugin;

impl Plugin for VerbPlugin {
    fn build(&self, app: &mut App) {
        let world = app.world_mut();
        super::register_remote_method(world, METHOD, process_remote_request);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Params {
    pub component_type: String,
    pub settings: ResourceInspectionSettings,
    pub metadata_map: Option<ComponentMetadataMap>,
}

pub fn process_remote_request(In(params): In<Option<Value>>, world: &World) -> BrpResult {
    let Params {
        component_type,
        settings,
        metadata_map,
    } = parse_some(params)?;
    let metadata_map = metadata_map.unwrap_or(ComponentMetadataMap::generate(world));
    let Some((component_id, _)) = super::component_type_to_metadata(&component_type, &metadata_map)
    else {
        return Err(super::component_type_not_in_metadata_brp_error(
            &component_type,
        ));
    };
    match world.inspect_resource_by_id(component_id, settings) {
        Ok(inspection) => Ok(serde_json::to_value(inspection).map_err(BrpError::internal)?),
        Err(error) => Err(determine_error(error)),
    }
}

fn determine_error(error: ResourceInspectionError) -> BrpError {
    use ResourceInspectionError::*;
    match error {
        ResourceNotRegistered(type_name) => {
            BrpError::resource_error(format!("Resource not registered: {type_name}"))
        }
        ResourceNotFound(component_id) => {
            BrpError::resource_not_present(&format!("Resource not found: {component_id:?}"))
        }
    }
}
