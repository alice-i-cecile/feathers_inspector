//! Rules and strategies for determining the inspection-displayed name of an entity.

use bevy::ecs::component::Component;

use crate::entity_inspection::EntityInspection;

impl EntityInspection {
    /// Determines the name to display for this entity.
    ///
    /// If the [`Name`](bevy::prelude::Name) component is present, its value will be used.
    ///
    /// If any component marked as "name-defining" is present, its name will be used.
    /// This is done by implementing the [`NameDefining`] trait for the component type,
    /// and then registering it for reflection by using the `#[reflect(NameDefining)]` attribute.
    /// If multiple name-defining components are present, they will be joined in alphabetical order,
    /// separated by a "|" character.
    ///
    /// Otherwise, [`None`] is returned.
    /// The caller can then fall back to a default name such as "Entity".
    pub fn resolve_name(&self) -> Option<String> {
        if let Some(custom_name) = &self.name {
            return Some(custom_name.as_str().to_string());
        } else {
            let mut name_defining_components: Vec<String> = self
                .components
                .iter()
                .filter(|comp| comp.is_name_defining)
                .map(|comp| comp.name.shortname().to_string())
                .collect();

            if !name_defining_components.is_empty() {
                name_defining_components.sort();

                let combined_name = name_defining_components.join(" | ");
                return Some(combined_name);
            } else {
                None
            }
        }
    }
}

/// A marker trait for components that should define an entity's name when inspected.
///
/// See [`EntityInspection::resolve_name`] for details on the name resolution rules.
///
/// Note: this should probably be replaced with a method on [`Component`] itself
/// once this crate is upstreamed into Bevy.
///
/// # Usage
///
/// ```
/// use bevy::prelude::*;
///
/// #[derive(Component, Reflect)]
/// #[reflect(NameDefining)]
/// struct Player;
/// ```
pub trait NameDefining: Component {}

/// Implementations of [`NameDefining`] for common first-party `bevy` components.
///
/// When upstreamed, these should be added to the definitions of those components directly.
mod bevy_name_defining_components {
    use super::NameDefining;
    use bevy::{
        core_pipeline::Skybox,
        ecs::system::SystemIdMarker,
        light::{FogVolume, IrradianceVolume, SunDisk},
        pbr::{Atmosphere, Lightmap, wireframe::Wireframe},
        prelude::*,
        window::Monitor,
    };

    // Windowing and input
    impl NameDefining for Window {}
    impl NameDefining for Monitor {}
    impl NameDefining for Gamepad {}

    // UI
    impl NameDefining for Node {}
    impl NameDefining for Button {}
    impl NameDefining for Text {}
    impl NameDefining for TextSpan {}
    impl NameDefining for Text2d {}
    impl NameDefining for ImageNode {}
    impl NameDefining for ViewportNode {}

    // Cameras
    impl NameDefining for Camera {}
    impl NameDefining for Camera2d {}
    impl NameDefining for Camera3d {}

    // Lights
    impl NameDefining for DirectionalLight {}
    impl NameDefining for PointLight {}
    impl NameDefining for SpotLight {}
    impl NameDefining for AmbientLight {}
    impl NameDefining for LightProbe {}
    impl NameDefining for IrradianceVolume {}
    impl NameDefining for SunDisk {}
    impl NameDefining for Lightmap {}

    // Core rendering components
    impl NameDefining for Sprite {}
    impl NameDefining for Mesh2d {}
    impl NameDefining for Mesh3d {}
    impl NameDefining for Wireframe {}

    // Atmospherics
    impl NameDefining for Skybox {}
    impl NameDefining for FogVolume {}
    impl NameDefining for Atmosphere {}
    impl NameDefining for DistanceFog {}

    // Animation
    impl NameDefining for AnimationPlayer {}

    // Audio
    impl NameDefining for AudioPlayer {}
    impl NameDefining for AudioSink {}

    // System-likes
    impl NameDefining for Observer {}
    impl NameDefining for SystemIdMarker {}
}
