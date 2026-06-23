//! Handles a `world.inspect_component` request coming from a client.
use bevy::{
    prelude::*,
    remote::{BrpError, BrpResult, builtin_methods::parse_some},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    extension_methods::WorldInspectionExtensionTrait,
    inspection::component_inspection::{
        ComponentInspectionError, ComponentInspectionSettings, ComponentMetadataMap,
    },
};

pub const METHOD: &str = "world.inspect_component";

pub(crate) struct VerbPlugin;

impl Plugin for VerbPlugin {
    fn build(&self, app: &mut App) {
        let world = app.world_mut();
        super::register_remote_method(world, METHOD, process_remote_request);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Params {
    pub component_name: String,
    pub entity: Entity,
    pub settings: ComponentInspectionSettings,
    pub metadata_map: Option<ComponentMetadataMap>,
}

pub fn process_remote_request(In(params): In<Option<Value>>, world: &World) -> BrpResult {
    let Params {
        component_name,
        entity,
        settings,
        metadata_map,
    } = parse_some(params)?;
    let metadata_map = metadata_map.unwrap_or(ComponentMetadataMap::generate(world));
    let Some((component_id, metadata)) =
        metadata_map.get_component_metadata_by_name(&component_name)
    else {
        return Err(super::component_name_not_in_metadata_brp_error(
            &component_name,
        ));
    };
    match world.inspect_component_by_id(component_id, entity, metadata, settings) {
        Ok(inspection) => Ok(serde_json::to_value(inspection).map_err(BrpError::internal)?),
        Err(error) => Err(determine_error(component_name, entity, error)),
    }
}

fn determine_error(
    component_type: String,
    entity: Entity,
    error: ComponentInspectionError,
) -> BrpError {
    match error {
        ComponentInspectionError::ComponentNotFound(_) => {
            BrpError::component_not_present(&component_type, entity)
        }
        ComponentInspectionError::ComponentNotRegistered(component_name) => {
            BrpError::component_error(format!("Component not registered: `{component_name}`"))
        }
        ComponentInspectionError::ComponentIdNotRegistered(component_id) => {
            BrpError::component_error(format!("Component id not registered: `{component_id:?}`"))
        }
    }
}
