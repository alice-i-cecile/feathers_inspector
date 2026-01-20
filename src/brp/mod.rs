//! Provides a plugin that adds custom BRP verbs
//! for methods defined in this library.
//!
//! To remotely use [`World`] methods
//! and some other items defined in this crate,
//! set up the BRP server in your Bevy app
//! according to [`bevy::remote`]'s documentation.
//! Then, register the custom methods by adding the [`InspectorBrpPlugin`].
//! Now you can send inspector requests via BRP to your app and get a response.
//!
//! Refer to the submodules to learn more about specific handlers.

use bevy::{
    ecs::component::ComponentId,
    prelude::*,
    remote::{BrpError, BrpResult, RemoteMethodSystemId, RemoteMethods},
};
use serde_json::Value;

use crate::component_inspection::{ComponentMetadataMap, ComponentTypeMetadata};

pub mod component_metadata_map_generate;
pub mod fuzzy_component_name_to_name;
pub mod fuzzy_resource_name_to_name;
pub mod inspect;
pub mod inspect_all_resources;
pub mod inspect_cached;
pub mod inspect_component;
pub mod inspect_component_type;
pub mod inspect_multiple;
pub mod inspect_resource;
pub mod summarize;

/// Provides BRP verbs for calling functions and methods defined in this crate.
///
/// ## Panics
///
/// This plugin assumes [`RemotePlugin`] is already added,
/// and will panic otherwise.
///
/// [`RemotePlugin`]: bevy::remote::RemotePlugin
pub struct InspectorBrpPlugin;

impl Plugin for InspectorBrpPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            component_metadata_map_generate::VerbPlugin,
            fuzzy_component_name_to_name::VerbPlugin,
            fuzzy_resource_name_to_name::VerbPlugin,
            inspect::VerbPlugin,
            inspect_all_resources::VerbPlugin,
            inspect_cached::VerbPlugin,
            inspect_component::VerbPlugin,
            inspect_component_type::VerbPlugin,
            inspect_multiple::VerbPlugin,
            inspect_resource::VerbPlugin,
            summarize::VerbPlugin,
        ));
    }
}

/// Registers an instant BRP method system under the given `method` name.
///
/// ## Panics
///
/// - If the [`RemotePlugin`] hasn't been added to the app
///   (i.e., [`RemoteMethods`] resource is missing).
///
/// [`RemotePlugin`]: bevy::remote::RemotePlugin
pub(crate) fn register_remote_method(
    world: &mut World,
    method: &str,
    system: fn(bevy::prelude::In<Option<Value>>, &World) -> BrpResult,
) {
    let system_id = world.register_system(system);

    let mut remote_methods = world
        .get_resource_mut::<RemoteMethods>()
        .expect("`RemotePlugin` must be present");
    remote_methods.insert(method, RemoteMethodSystemId::Instant(system_id));
}

/// Returns a [`ComponentMetadataMap`] entry
pub fn component_type_to_metadata<'metadata>(
    component_type: &str,
    metadata_map: &'metadata ComponentMetadataMap,
) -> Option<(ComponentId, &'metadata ComponentTypeMetadata)> {
    metadata_map.map.iter().find_map(|(id, meta)| {
        let full = meta.name.to_string();
        (full == component_type).then_some((*id, meta))
    })
}

/// Custom BRP error codes for this library.
pub mod error_codes {
    /// Fuzzy name mapping returned no candidates.
    pub const NO_FUZZY_NAME_CANDIDATES: i16 = 1;
    /// The [`ComponentMetadataMap`] does not contain data about the given component.
    ///
    /// [`ComponentMetadataMap`]: crate::component_inspection::ComponentMetadataMap
    pub const COMPONENT_TYPE_NOT_IN_METADATA: i16 = 2;
}

pub fn no_fuzzy_name_candidates_brp_error(fuzzy_name: &str) -> BrpError {
    let data = serde_json::to_value(fuzzy_name.to_string()).ok();
    BrpError {
        code: error_codes::NO_FUZZY_NAME_CANDIDATES,
        message: format!("No matches found for fuzzy name \"{fuzzy_name}\""),
        data,
    }
}

pub fn component_type_not_in_metadata_brp_error(component_type: &str) -> BrpError {
    let data = serde_json::to_value(component_type.to_string()).ok();
    BrpError {
        code: error_codes::COMPONENT_TYPE_NOT_IN_METADATA,
        message: format!("Component not found in metadata: `{component_type}`"),
        data,
    }
}
