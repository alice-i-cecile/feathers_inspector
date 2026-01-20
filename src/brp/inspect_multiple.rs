//! Handles a `world.inspect_multiple` request coming from a client.
use bevy::{
    prelude::*,
    remote::{BrpError, BrpResult, builtin_methods::parse_some},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    component_inspection::ComponentMetadataMap,
    entity_inspection::MultipleEntityInspectionSettings,
    extension_methods::WorldInspectionExtensionTrait,
};

pub const METHOD: &str = "world.inspect_multiple";

pub(crate) struct VerbPlugin;

impl Plugin for VerbPlugin {
    fn build(&self, app: &mut App) {
        let world = app.world_mut();
        super::register_remote_method(world, METHOD, process_remote_request);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Params {
    pub entities: Vec<Entity>,
    pub settings: MultipleEntityInspectionSettings,
    pub metadata_map: ComponentMetadataMap,
}

pub fn process_remote_request(In(params): In<Option<Value>>, world: &World) -> BrpResult {
    let Params {
        entities,
        settings,
        mut metadata_map,
    } = parse_some(params)?;
    let inspection = world.inspect_multiple(entities, settings, &mut metadata_map);
    serde_json::to_value(inspection).map_err(BrpError::internal)
}
