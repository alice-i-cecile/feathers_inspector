//! Rules and strategies for determining the inspection-displayed name of an entity.

use bevy::prelude::*;

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
            let name_defining_components: Vec<(i32, String)> = self
                .components
                .iter()
                .filter(|comp| comp.is_name_defining.is_some())
                .map(|comp| {
                    (
                        comp.is_name_defining.unwrap(),
                        comp.name.shortname().to_string(),
                    )
                })
                .collect();

            if !name_defining_components.is_empty() {
                // Filter for only the highest-priority name-defining components
                let mut highest_priority = i32::MIN;
                let mut selected_names = vec![];

                // PERF: this can definitely be done more efficiently
                for (priority, name) in &name_defining_components {
                    if *priority > highest_priority {
                        highest_priority = *priority;
                        selected_names = vec![name.clone()];
                    } else if *priority == highest_priority {
                        selected_names.push(name.clone());
                    }
                }

                // Sort alphabetically for consistent ordering
                selected_names.sort();

                let combined_name = selected_names.join(" | ");
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
pub trait NameDefining: Component {
    /// The priority of this name-defining component,
    /// controlling which name takes precedence when multiple are present.
    ///
    /// Higher values indicate higher priority.
    /// If multiple components have the same priority, their names will be joined alphabetically.
    ///
    /// # Conventions
    ///
    /// - Engine-defined or library-defined components use negative priorities
    ///   - Components that will typically define an entity's name (e.g. [`Sprite`], [`Camera2d`]) have a priority of -10
    ///   - Fallback components (e.g. [`Camera`]) have a priority of -20
    /// - Application-defined components use zero or positive priorities
    /// 	- 0 is the default value, and can be used for typical application-defined name-defining components (e.g. `Unit`)
    /// 	- Higher values can be used for more specific components that should take precedence (e.g. `Player`, `Enemy`)
    /// - Each entity, when spawned in its "default" configuration, should have one unambiguous highest-priority name-defining component
    /// - Values should be spaced out when possible to allow for future additions without requiring renumbering
    const PRIORITY: i8 = 0;
}

/// Implementations of [`NameDefining`] for common first-party `bevy` components.
///
/// When upstreamed, these should be added to the definitions of those components directly.
///
/// # Rendering components on cameras
///
/// Many important scene properties are defined on the camera entity,
/// but adding them as name-defining components would cause
/// every camera to have a long, unwieldy name.
///
/// Therefore, we do not implement NameDefining for these components.
/// In the future, it may be worth moving these to separate entities,
/// joined by relationships, to allow for more intuitive inspection and naming.
///
// A partial list follows for reference:
/// - [`AmbientLight`]
/// - [`Skybox`](bevy::core_pipeline::Skybox)
/// - [`Atmosphere`](bevy::pbr::Atmosphere)
/// - [`DistanceFog`](bevy::pbr::DistanceFog)
mod bevy_name_defining_components {
    use super::NameDefining;
    use bevy::{
        ecs::system::SystemIdMarker,
        light::{FogVolume, IrradianceVolume, SunDisk},
        pbr::Lightmap,
        prelude::*,
        window::Monitor,
    };

    // Windowing and input
    impl NameDefining for Window {
        const PRIORITY: i8 = -10;
    }
    impl NameDefining for Monitor {
        const PRIORITY: i8 = -10;
    }
    impl NameDefining for Gamepad {
        const PRIORITY: i8 = -10;
    }

    // UI
    impl NameDefining for Node {
        const PRIORITY: i8 = -20;
    }
    impl NameDefining for Button {
        const PRIORITY: i8 = -10;
    }
    impl NameDefining for Text {
        const PRIORITY: i8 = -10;
    }
    impl NameDefining for TextSpan {
        const PRIORITY: i8 = -10;
    }
    impl NameDefining for Text2d {
        const PRIORITY: i8 = -10;
    }
    impl NameDefining for ImageNode {
        const PRIORITY: i8 = -10;
    }
    impl NameDefining for ViewportNode {
        const PRIORITY: i8 = -10;
    }

    // Cameras
    impl NameDefining for Camera {
        const PRIORITY: i8 = -20;
    }
    impl NameDefining for Camera2d {
        const PRIORITY: i8 = -10;
    }
    impl NameDefining for Camera3d {
        const PRIORITY: i8 = -10;
    }

    // Lights
    impl NameDefining for DirectionalLight {
        const PRIORITY: i8 = -10;
    }
    impl NameDefining for SunDisk {
        const PRIORITY: i8 = -5;
    }
    impl NameDefining for PointLight {
        const PRIORITY: i8 = -10;
    }
    impl NameDefining for SpotLight {
        const PRIORITY: i8 = -10;
    }
    impl NameDefining for LightProbe {
        const PRIORITY: i8 = -20;
    }
    impl NameDefining for IrradianceVolume {
        const PRIORITY: i8 = -10;
    }
    impl NameDefining for Lightmap {
        const PRIORITY: i8 = -10;
    }

    // Core rendering components
    impl NameDefining for Sprite {
        const PRIORITY: i8 = -10;
    }
    impl NameDefining for Mesh2d {
        const PRIORITY: i8 = -10;
    }
    impl NameDefining for Mesh3d {
        const PRIORITY: i8 = -10;
    }

    // Atmospherics
    impl NameDefining for FogVolume {}

    // Animation
    impl NameDefining for AnimationPlayer {
        /// This should be at the same level as Mesh3d to ensure that animated meshes are obviously identified.
        const PRIORITY: i8 = -10;
    }

    // Audio
    impl NameDefining for AudioPlayer {
        const PRIORITY: i8 = -20;
    }
    impl NameDefining for AudioSink {
        const PRIORITY: i8 = -20;
    }

    // System-likes
    impl NameDefining for Observer {
        /// Observers are sometimes attached to the entity being observed,
        /// so we give them a low priority to avoid interfering with more important name-defining components.
        const PRIORITY: i8 = -20;
    }
    impl NameDefining for SystemIdMarker {
        const PRIORITY: i8 = -10;
    }
}
