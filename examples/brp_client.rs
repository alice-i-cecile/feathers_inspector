//! Demonstrates how to inspect an out-of-process Bevy app
//! by sending BRP requests to it.
//!
//! Run this example with the `remote` feature enabled:
//! ```bash
//! cargo run --example brp_client --features="remote"
//! ```

use bevy::prelude::*;
use bevy::remote::BrpRequest;
use bevy::remote::builtin_methods::{BRP_QUERY_METHOD, BrpQuery, BrpQueryFilter, BrpQueryParams};
use bevy::remote::http::{DEFAULT_ADDR, DEFAULT_PORT};
use feathers_inspector::brp;
use feathers_inspector::inspection::component_inspection::{
    ComponentDetailLevel, ComponentInspection, ComponentInspectionSettings, ComponentMetadataMap,
    ComponentTypeInspection,
};
use feathers_inspector::inspection::entity_inspection::{
    EntityInspection, EntityInspectionError, EntityInspectionSettings,
    MultipleEntityInspectionSettings,
};
use feathers_inspector::inspection::resource_inspection::{
    ResourceInspection, ResourceInspectionSettings,
};
use feathers_inspector::summary::{SummarySettings, WorldSummary};

use crate::helper::{construct_request, post_request, query};

const SPRITE_COMPONENT_NAME: &str = "bevy_sprite::sprite::Sprite";
const TIME_RESOURCE_NAME: &str = "bevy_time::time::Time";

#[derive(Resource, Debug)]
struct BrpUrl(String);

impl Default for BrpUrl {
    fn default() -> Self {
        let host_part = format!("{DEFAULT_ADDR}:{DEFAULT_PORT}");
        Self(format!("http://{host_part}/"))
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<BrpUrl>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                inspect_all_entities_when_space_pressed,
                inspect_specific_component_when_c_pressed,
                inspect_resource_when_r_pressed,
                inspect_all_resources_when_a_pressed,
                inspect_sprite_component_type_when_m_pressed,
                summarize_when_s_pressed,
            ),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    let instructions = "\
This is your client process, that connects to the Bevy app via BRP.
You can use the keyboard buttons to send BRP requests.
Output will be shown in the console.

Press `Space` to inspect all entities
Press 'C' to inspect the Sprite component on all Sprite entities
Press 'R' to inspect the Time resource
Press 'A' to inspect all resources
Press 'M' to inspect the Sprite component type metadata
Press 'S' to obtain summary statistics"
        .to_string();

    commands.spawn((
        Text::new(instructions),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
    ));
}

fn inspect_all_entities_when_space_pressed(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    brp_url: Res<BrpUrl>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        let query_params = BrpQueryParams {
            data: BrpQuery::default(),
            filter: BrpQueryFilter::default(),
            strict: false,
        };
        let entities = query(query_params, &brp_url.0);
        let request = BrpRequest {
            jsonrpc: String::from("2.0"),
            method: brp::component_metadata_map_generate::METHOD.to_string(),
            id: None,
            params: None,
        };
        let metadata_map = post_request::<ComponentMetadataMap>(request, &brp_url.0);
        let settings = MultipleEntityInspectionSettings {
            entity_settings: EntityInspectionSettings {
                include_components: false,
                ..default()
            },
            ..default()
        };
        let params = brp::inspect_multiple::Params {
            entities: entities.into_iter().collect::<Vec<Entity>>(),
            settings,
            metadata_map,
        };
        let request = construct_request(brp::inspect_multiple::METHOD, params);
        let inspections = post_request::<Vec<Result<EntityInspection, EntityInspectionError>>>(
            request, &brp_url.0,
        );

        for result in inspections {
            if let Ok(inspection) = result {
                info!("{inspection}");
            } else {
                warn!("Could not inspect an entity")
            }
        }
    }
}

fn inspect_specific_component_when_c_pressed(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    brp_url: Res<BrpUrl>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyC) {
        let query_params = BrpQueryParams {
            data: BrpQuery::default(),
            filter: BrpQueryFilter {
                with: vec![SPRITE_COMPONENT_NAME.to_string()],
                ..default()
            },
            strict: false,
        };
        let entities = query(query_params, &brp_url.0);
        let settings = ComponentInspectionSettings {
            detail_level: ComponentDetailLevel::Values,
            full_type_names: true,
        };
        for entity in entities {
            let params = brp::inspect_component::Params {
                component_type: SPRITE_COMPONENT_NAME.to_string(),
                entity,
                settings,
                metadata_map: None,
            };
            let request = construct_request(brp::inspect_component::METHOD, params);
            let inspection = post_request::<ComponentInspection>(request, &brp_url.0);
            info!("{inspection}");
        }
    }
}

fn inspect_resource_when_r_pressed(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    brp_url: Res<BrpUrl>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        let settings = ResourceInspectionSettings {
            full_type_names: true,
        };
        let params = brp::inspect_resource::Params {
            component_type: TIME_RESOURCE_NAME.to_string(),
            settings,
            metadata_map: None,
        };
        let request = construct_request(brp::inspect_resource::METHOD, params);
        let inspection = post_request::<ResourceInspection>(request, &brp_url.0);
        info!("{inspection}");
    }
}

fn inspect_all_resources_when_a_pressed(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    brp_url: Res<BrpUrl>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyA) {
        let settings = ResourceInspectionSettings {
            full_type_names: false,
        };
        let params = brp::inspect_all_resources::Params { settings };
        let request = construct_request(brp::inspect_all_resources::METHOD, params);
        let inspections = post_request::<Vec<ResourceInspection>>(request, &brp_url.0);
        for inspection in inspections {
            info!("{inspection}");
        }
    }
}

fn inspect_sprite_component_type_when_m_pressed(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    brp_url: Res<BrpUrl>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyM) {
        let params = brp::inspect_component_type::Params {
            component_type: SPRITE_COMPONENT_NAME.to_string(),
            metadata_map: None,
        };
        let request = construct_request(brp::inspect_component_type::METHOD, params);
        let inspection = post_request::<ComponentTypeInspection>(request, &brp_url.0);
        info!("{inspection}");
    }
}

fn summarize_when_s_pressed(keyboard_input: Res<ButtonInput<KeyCode>>, brp_url: Res<BrpUrl>) {
    if keyboard_input.just_pressed(KeyCode::KeyS) {
        let settings = SummarySettings::default();
        let params = brp::summarize::Params { settings };
        let request = construct_request(brp::summarize::METHOD, params);
        let summary = post_request::<WorldSummary>(request, &brp_url.0);
        info!("{summary}");
    }
}

// Since BRP request and response handling are quite verbose,
// we define a helper module to contain the complexity.
mod helper {
    use serde::{Serialize, de::DeserializeOwned};

    use super::*;

    pub fn query(params: BrpQueryParams, url: &str) -> Vec<Entity> {
        let query_entities_request = BrpRequest {
            jsonrpc: String::from("2.0"),
            method: String::from(BRP_QUERY_METHOD),
            id: None,
            params: Some(
                serde_json::to_value(params)
                    .expect("Unable to convert query parameters to a valid JSON value"),
            ),
        };
        let response = ureq::post(url)
            .send_json(query_entities_request)
            .expect("Failed to send JSON to server")
            .body_mut()
            .read_json::<serde_json::Value>()
            .expect("Failed to read JSON response");
        response["result"]
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item["entity"].as_u64())
                    .map(Entity::from_bits)
                    .collect::<Vec<Entity>>()
            })
            .unwrap_or_default()
    }

    pub fn construct_request<T>(method: &str, params: T) -> BrpRequest
    where
        T: Serialize,
    {
        BrpRequest {
            jsonrpc: String::from("2.0"),
            method: method.to_string(),
            id: None,
            params: Some(
                serde_json::to_value(params)
                    .expect("Unable to convert query parameters to a valid JSON value"),
            ),
        }
    }

    pub fn post_request<T>(request: BrpRequest, url: &str) -> T
    where
        T: DeserializeOwned,
    {
        let response = ureq::post(url)
            .send_json(request)
            .expect("Failed to send JSON to server")
            .body_mut()
            .read_json::<serde_json::Value>()
            .expect("Failed to read JSON response");
        let result = response
            .get("result")
            .expect("Missing `result` field in JSON-RPC response");
        serde_json::from_value::<T>(result.clone()).expect("Failed to deserialize")
    }
}
