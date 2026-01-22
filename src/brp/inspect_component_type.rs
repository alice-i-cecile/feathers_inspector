//! Handles a `world.inspect_component_type` request coming from a client.
use bevy::{
    prelude::*,
    remote::{BrpError, BrpResult, builtin_methods::parse_some},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    extension_methods::WorldInspectionExtensionTrait,
    inspection::component_inspection::{ComponentInspectionError, ComponentMetadataMap},
};

pub const METHOD: &str = "world.inspect_component_type";

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
    pub metadata_map: Option<ComponentMetadataMap>,
}

pub fn process_remote_request(In(params): In<Option<Value>>, world: &World) -> BrpResult {
    let Params {
        component_type,
        metadata_map,
    } = parse_some(params)?;
    let metadata_map = metadata_map.unwrap_or(ComponentMetadataMap::generate(world));
    let Some((component_id, _)) = super::component_type_to_metadata(&component_type, &metadata_map)
    else {
        return Err(super::component_type_not_in_metadata_brp_error(
            &component_type,
        ));
    };
    match world.inspect_component_type_by_id(component_id) {
        Ok(inspection) => Ok(serde_json::to_value(inspection).map_err(BrpError::internal)?),
        Err(error) => Err(determine_error(error)),
    }
}

fn determine_error(error: ComponentInspectionError) -> BrpError {
    use ComponentInspectionError::*;
    match error {
        ComponentNotFound(component_id) => {
            let component_index = component_id.index().to_string();
            BrpError::component_error(format!("Component not found: {component_index}"))
        }
        ComponentNotRegistered(component_type_name) => {
            BrpError::component_error(format!("Component not registered: {component_type_name}"))
        }
        ComponentIdNotRegistered(component_id) => {
            let component_index = component_id.index().to_string();
            BrpError::component_error(format!("Component not registered: {component_index}"))
        }
    }
}
