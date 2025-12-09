//! Handles a `world.inspect` request coming from a client.
use bevy::{
    prelude::*,
    remote::{BrpError, BrpResult},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    brp::inspect_cached::determine_error, entity_inspection::EntityInspectionSettings,
    extension_methods::WorldInspectionExtensionTrait,
};

pub const METHOD: &str = "world.inspect";

pub(crate) struct VerbPlugin;

impl Plugin for VerbPlugin {
    fn build(&self, app: &mut App) {
        let world = app.world_mut();
        super::register_remote_method(world, METHOD, process_remote_request);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Params {
    pub entity: Entity,
    pub settings: EntityInspectionSettings,
}

pub fn process_remote_request(In(params): In<Option<Value>>, world: &World) -> BrpResult {
    let Params { entity, settings } = super::parse_some(params)?;
    let entity_inspection = world.inspect(entity, settings);
    match entity_inspection {
        Ok(entity_inspection) => {
            serde_json::to_value(entity_inspection).map_err(BrpError::internal)
        }
        Err(inspection_error) => Err(determine_error(entity, inspection_error)),
    }
}
