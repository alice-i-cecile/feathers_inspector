//! Handles a `world.inspect_cached` request coming from a client.
use bevy::{
    prelude::*,
    remote::{BrpError, BrpResult},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    component_inspection::ComponentMetadataMap,
    entity_inspection::{EntityInspectionError, EntityInspectionSettings},
    extension_methods::WorldInspectionExtensionTrait,
};

pub const METHOD: &str = "world.inspect_cached";

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
    pub metadata_map: ComponentMetadataMap,
}

pub fn process_remote_request(In(params): In<Option<Value>>, world: &World) -> BrpResult {
    let Params {
        entity,
        settings,
        metadata_map,
    } = super::parse_some(params)?;
    let entity_inspection = world.inspect_cached(entity, &settings, &metadata_map);
    match entity_inspection {
        Ok(entity_inspection) => {
            serde_json::to_value(entity_inspection).map_err(BrpError::internal)
        }
        Err(inspection_error) => Err(determine_error(entity, inspection_error)),
    }
}

pub(super) fn determine_error(entity: Entity, inspection_error: EntityInspectionError) -> BrpError {
    use EntityInspectionError::*;
    use bevy::ecs::query::QueryEntityError::*;
    match inspection_error {
        EntityNotFound(_) => BrpError::entity_not_found(entity),
        UnexpectedQueryError(query_entity_error) => match query_entity_error {
            QueryDoesNotMatch(_, _) => {
                BrpError::internal("Reached invalid state: `QueryDoesNotMatch` on `SpawnDetails`")
            }
            EntityDoesNotExist(_) => BrpError::entity_not_found(entity),
            AliasedMutability(_) => {
                BrpError::internal("Reached invalid state: `AliasedMutability`")
            }
        },
    }
}
