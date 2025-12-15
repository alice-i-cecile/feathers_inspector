//! Handles a `world.summarize` request coming from a client.
use bevy::{
    prelude::*,
    remote::{BrpError, BrpResult},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::summary::{SummarySettings, WorldSummaryExt};

pub const METHOD: &str = "world.summarize";

pub(crate) struct VerbPlugin;

impl Plugin for VerbPlugin {
    fn build(&self, app: &mut App) {
        let world = app.world_mut();
        super::register_remote_method(world, METHOD, process_remote_request);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Params {
    pub settings: SummarySettings,
}

pub fn process_remote_request(In(params): In<Option<Value>>, world: &World) -> BrpResult {
    let Params { settings } = super::parse_some(params)?;
    let summary = world.summarize(settings);
    serde_json::to_value(summary).map_err(BrpError::internal)
}
