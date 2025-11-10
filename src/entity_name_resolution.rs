//! Rules and strategies for determining the inspection-displayed name of an entity.

use bevy::core_pipeline::Skybox;
use bevy::ecs::component::ComponentId;
use bevy::ecs::system::SystemIdMarker;
use bevy::light::{FogVolume, IrradianceVolume, SunDisk};
use bevy::pbr::wireframe::Wireframe;
use bevy::pbr::{Atmosphere, Lightmap};
use bevy::picking::pointer::PointerId;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::window::Monitor;
use core::any::TypeId;

use crate::component_inspection::{ComponentInspection, ComponentTypeMetadata};

/// The name of an inspected entity.
///
/// This data is produced by [`resolve_name`].
#[derive(Clone, Debug, PartialEq, Eq, Deref, DerefMut)]
pub struct EntityName {
    #[deref]
    pub name: String,
    pub origin: NameOrigin,
}

impl EntityName {
    /// Constructs a [`Custom`] entity name.
    ///
    /// [`Custom`]: NameOrigin::Custom
    pub(crate) fn custom(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            origin: NameOrigin::Custom,
        }
    }

    /// Constructs a [`Resolved`] entity name.
    ///
    /// [`Resolved`]: NameOrigin::Resolved
    pub(crate) fn resolved(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            origin: NameOrigin::Resolved,
        }
    }
}

/// Identifies whether the inspected entity's name
/// is manually assigned or automatically resolved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NameOrigin {
    /// The entity name comes from the [`Name`] component.
    Custom,
    /// The name was generated using [`resolve_name`].
    Resolved,
}

/// Determines the name to display for this entity.
///
/// If the [`Name`] component is present, its value will be used.
///
/// If any component marked as "name-defining" is present, its name will be used.
/// If multiple name-defining components are present, they will be joined in alphabetical order,
/// separated by a "|" character.
///
/// Otherwise, [`None`] is returned.
/// The caller can then fall back to a default name such as "Entity".
pub fn resolve_name(
    world: &World,
    entity: Entity,
    components: &Option<Vec<ComponentInspection>>,
    metadata_map: &HashMap<ComponentId, ComponentTypeMetadata>,
) -> Option<EntityName> {
    if let Some(custom_name) = world.get::<Name>(entity).cloned() {
        Some(EntityName::custom(custom_name.as_str()))
    } else {
        let Some(component_data) = components else {
            return None;
        };

        let mut name_resolution_priorities = component_data
            .iter()
            .filter_map(|comp_inspection| {
                let name_definition_priority = metadata_map
                    .get(&comp_inspection.component_id)?
                    .name_definition_priority;
                name_definition_priority
                    .map(|priority| (comp_inspection.name.shortname().to_string(), priority))
            })
            .collect::<Vec<(String, i8)>>();

        if name_resolution_priorities.is_empty() {
            return None;
        }

        // Sort by priority (higher priority first), then by name alphabetically
        name_resolution_priorities.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        // Only include names with the highest priority
        // PERF: we could do this more efficiently by combining the sort and filter steps
        let highest_priority = name_resolution_priorities[0].1;
        name_resolution_priorities.retain(|&(_, priority)| priority == highest_priority);

        let resolved_name = name_resolution_priorities
            .into_iter()
            .map(|(name, _)| name)
            .collect::<Vec<String>>()
            .join(" | ");

        Some(EntityName::resolved(&resolved_name))
    }
}

/// Stores the registered name-defining component types and their priorities.
///
/// When determining an entity's name via [`resolve_name`], components with higher priority values
/// will take precedence over those with lower priority values.
///
/// Explicitly named entities (via the [`Name`] component) will always take precedence over name-defining components.
///
/// # Priority Conventions
///
/// - User-defined name-defining components should have a priority of `0`.
/// - Library-defined components (in Bevy, or in third-party Bevy crates) that are name-defining should have a priority of `-10`.
/// - Fallback components (e.g. [`Camera`]) should have a priority of `-20`.
///
/// Leaving space between these priority levels allows for future expansion and customization in tricky edge cases.
///
/// # Usage
///
/// Components that should be "name-defining" should be registered in this registry
/// using [`NameResolutionRegistry::register_name_defining_type`],
/// typically in the plugin that defines the component.
#[derive(Debug, Resource, Default)]
pub struct NameResolutionRegistry {
    /// A mapping of name-defining component TypeIds to their priority levels.
    name_defining_types: HashMap<TypeId, i8>,
}

impl NameResolutionRegistry {
    /// Creates a new, empty [`NameResolutionRegistry`].
    pub const fn new() -> Self {
        Self {
            name_defining_types: HashMap::new(),
        }
    }

    /// Registers a name-defining component type with the given priority.
    ///
    /// Higher priority components will take precedence when determining an entity's name.
    pub fn register_name_defining_type<T: 'static>(&mut self, priority: i8) {
        let type_id = TypeId::of::<T>();
        self.name_defining_types.insert(type_id, priority);
    }

    /// Gets the priority of a name-defining component type, if registered.
    pub fn get_priority<T: 'static>(&self) -> Option<i8> {
        let type_id = TypeId::of::<T>();
        self.get_priority_by_type_id(type_id)
    }

    /// Gets the priority of a name-defining component type by its [`TypeId`], if registered.
    pub fn get_priority_by_type_id(&self, type_id: TypeId) -> Option<i8> {
        self.name_defining_types.get(&type_id).cloned()
    }

    /// Removes a name-defining component type from the registry.
    pub fn unregister_name_defining_type<T: 'static>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.unregister_name_defining_type_by_type_id(type_id);
    }

    /// Removes a name-defining component type from the registry by its [`TypeId`].
    pub fn unregister_name_defining_type_by_type_id(&mut self, type_id: TypeId) {
        self.name_defining_types.remove(&type_id);
    }
}

/// A plugin which registers name-defining components for Bevy's first-party types
/// in the [`NameResolutionRegistry`] resource.
///
/// When upstreamed, this plugin should not be necessary,
/// as each name-defining component can register itself in its own plugin.
pub struct NameResolutionPlugin;
impl Plugin for NameResolutionPlugin {
    fn build(&self, app: &mut App) {
        let mut name_resolution_registry = NameResolutionRegistry::new();

        // Windowing and input
        name_resolution_registry.register_name_defining_type::<Window>(-10);
        name_resolution_registry.register_name_defining_type::<Monitor>(-10);
        name_resolution_registry.register_name_defining_type::<Gamepad>(-10);
        name_resolution_registry.register_name_defining_type::<PointerId>(-10);

        // UI
        name_resolution_registry.register_name_defining_type::<Node>(-20);
        name_resolution_registry.register_name_defining_type::<Button>(-10);
        name_resolution_registry.register_name_defining_type::<Text>(-10);
        name_resolution_registry.register_name_defining_type::<TextSpan>(-10);
        name_resolution_registry.register_name_defining_type::<Text2d>(-10);
        name_resolution_registry.register_name_defining_type::<ImageNode>(-10);
        name_resolution_registry.register_name_defining_type::<ViewportNode>(-10);

        // Cameras
        name_resolution_registry.register_name_defining_type::<Camera>(-20);
        name_resolution_registry.register_name_defining_type::<Camera2d>(-10);
        name_resolution_registry.register_name_defining_type::<Camera3d>(-10);

        // Lights
        name_resolution_registry.register_name_defining_type::<DirectionalLight>(-10);
        name_resolution_registry.register_name_defining_type::<PointLight>(-10);
        name_resolution_registry.register_name_defining_type::<SpotLight>(-10);
        name_resolution_registry.register_name_defining_type::<AmbientLight>(-10);
        name_resolution_registry.register_name_defining_type::<LightProbe>(-10);
        name_resolution_registry.register_name_defining_type::<IrradianceVolume>(-10);
        name_resolution_registry.register_name_defining_type::<SunDisk>(-10);
        name_resolution_registry.register_name_defining_type::<Lightmap>(-10);

        // Core rendering components
        name_resolution_registry.register_name_defining_type::<Sprite>(-10);
        name_resolution_registry.register_name_defining_type::<Mesh2d>(-10);
        name_resolution_registry.register_name_defining_type::<Mesh3d>(-10);
        name_resolution_registry.register_name_defining_type::<Wireframe>(-10);

        // Atmospherics
        name_resolution_registry.register_name_defining_type::<Skybox>(-10);
        name_resolution_registry.register_name_defining_type::<FogVolume>(-10);
        name_resolution_registry.register_name_defining_type::<Atmosphere>(-10);
        name_resolution_registry.register_name_defining_type::<DistanceFog>(-10);

        // Animation
        name_resolution_registry.register_name_defining_type::<AnimationPlayer>(-10);

        // Audio
        name_resolution_registry.register_name_defining_type::<AudioPlayer>(-10);
        name_resolution_registry.register_name_defining_type::<AudioSink>(-10);

        // System-likes
        name_resolution_registry.register_name_defining_type::<Observer>(-10);
        name_resolution_registry.register_name_defining_type::<SystemIdMarker>(-10);

        app.insert_resource(name_resolution_registry);
    }
}
