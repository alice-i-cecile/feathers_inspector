//! Rules and strategies for determining the inspection-displayed name of an entity.

use bevy::core_pipeline::Skybox;
use bevy::ecs::component::ComponentId;
use bevy::ecs::system::SystemIdMarker;
use bevy::light::{Atmosphere, FogVolume, IrradianceVolume, SunDisk};
use bevy::pbr::Lightmap;
use bevy::pbr::wireframe::Wireframe;
use bevy::picking::pointer::PointerId;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::window::Monitor;
use core::any::TypeId;

pub mod fuzzy_name_mapping;

/// The priority level for name-defining components.
///
/// Higher values indicate higher priority when determining an entity's name.
///
/// # Priority Conventions
///
/// - User-defined name-defining components should use [`USER`](Self::USER) priority (`0`).
/// - Library-defined components (in Bevy, or in third-party Bevy crates)
///   that are name-defining should use [`LIBRARY`](Self::LIBRARY) priority (`-10`).
/// - Fallback components (e.g. [`Camera`]) should use [`FALLBACK`](Self::FALLBACK) priority (`-20`).
///
/// Leaving space between these priority levels allows for future expansion
/// and customization in tricky edge cases.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NameDefinitionPriority(pub i8);

impl NameDefinitionPriority {
    /// The recommended priority for user-defined name-defining components.
    pub const USER: Self = Self(0);

    /// The recommended priority for library-defined name-defining components
    /// (e.g. Bevy built-ins or third-party crate types).
    pub const LIBRARY: Self = Self(-10);

    /// The recommended priority for fallback name-defining components
    /// that should only be used when no better name is available
    /// (e.g. [`Camera`], [`Node`]).
    pub const FALLBACK: Self = Self(-20);
}

/// Summary of a component on an entity, used for name resolution in [`resolve_name`].
#[derive(Clone, Copy, Debug)]
pub struct ComponentNameData<'a> {
    /// The [`ComponentId`] of the component.
    pub component_id: ComponentId,
    /// The short (unqualified) name of the component type.
    pub short_name: &'a str,
    /// The name-defining priority, if this component is registered as name-defining.
    pub name_definition_priority: Option<NameDefinitionPriority>,
}

/// The name of an inspected entity.
///
/// This data is produced by [`resolve_name`].
#[derive(Clone, Debug, PartialEq, Eq, Deref, DerefMut)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EntityName {
    /// The resolved name to display for the entity.
    #[deref]
    pub name: Name,
    /// How the name was determined, which can be used to inform display decisions.
    pub origin: NameOrigin,
}

impl EntityName {
    /// Constructs a [`Custom`] entity name.
    ///
    /// [`Custom`]: NameOrigin::Custom
    pub(crate) fn custom(name: &str) -> Self {
        Self {
            name: Name::new(name.to_owned()),
            origin: NameOrigin::Custom,
        }
    }

    /// Constructs a [`Resolved`] entity name.
    ///
    /// [`Resolved`]: NameOrigin::Resolved
    pub(crate) fn resolved(name: &str) -> Self {
        Self {
            name: Name::new(name.to_owned()),
            origin: NameOrigin::Resolved,
        }
    }

    /// Constructs a [`Fallback`] entity name.
    ///
    /// [`Fallback`]: NameOrigin::Fallback
    pub(crate) fn fallback(name: &str) -> Self {
        Self {
            name: Name::new(name.to_owned()),
            origin: NameOrigin::Fallback,
        }
    }
}

/// Identifies how the inspected entity's name was determined.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NameOrigin {
    /// The entity name comes from the [`Name`] component.
    Custom,
    /// The name was resolved from name-defining components via [`resolve_name`].
    Resolved,
    /// No [`Name`] component or name-defining component was found;
    /// the caller provided a fallback name.
    Fallback,
}

/// Determines the name to display for the given `entity`.
///
/// If the [`Name`] component is present, its value will be used.
///
/// If any component marked as "name-defining" is present
/// (i.e., has a [`NameDefinitionPriority`]), its name will be used.
/// If multiple name-defining components with the same highest priority are present,
/// they will be joined in alphabetical order,
/// separated by a "|" character.
///
/// Otherwise, [`None`] is returned.
/// The caller can then fall back to a default name such as "Entity".
///
/// # Arguments
///
/// * `world` - The world to query for the entity's [`Name`] component.
/// * `entity` - The entity to resolve the name for.
/// * `components` - A slice of [`ComponentNameData`] describing each component on the entity.
///   Callers should assemble these from whatever component data they have.
pub fn resolve_name(
    world: &World,
    entity: Entity,
    components: &[ComponentNameData],
) -> Option<EntityName> {
    if let Some(custom_name) = world.get::<Name>(entity).cloned() {
        return Some(EntityName::custom(custom_name.as_str()));
    }

    let mut name_resolution_priorities: Vec<(&str, NameDefinitionPriority)> = components
        .iter()
        .filter_map(|c| c.name_definition_priority.map(|p| (c.short_name, p)))
        .collect();

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
        .collect::<Vec<&str>>()
        .join(" | ");

    Some(EntityName::resolved(&resolved_name))
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
/// See the associated constants on [`NameDefinitionPriority`] for recommended priority levels.
///
/// # Usage
///
/// Components that should be "name-defining" should be registered in this registry
/// using [`NameResolutionRegistry::register_name_defining_type`],
/// typically in the plugin that defines the component.
#[derive(Debug, Resource, Default)]
pub struct NameResolutionRegistry {
    /// A mapping of name-defining component TypeIds to their priority levels.
    name_defining_types: HashMap<TypeId, NameDefinitionPriority>,
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
    pub fn register_name_defining_type<T: 'static>(&mut self, priority: NameDefinitionPriority) {
        let type_id = TypeId::of::<T>();
        self.name_defining_types.insert(type_id, priority);
    }

    /// Gets the priority of a name-defining component type, if registered.
    pub fn get_priority<T: 'static>(&self) -> Option<NameDefinitionPriority> {
        let type_id = TypeId::of::<T>();
        self.get_priority_by_type_id(type_id)
    }

    /// Gets the priority of a name-defining component type by its [`TypeId`], if registered.
    pub fn get_priority_by_type_id(&self, type_id: TypeId) -> Option<NameDefinitionPriority> {
        self.name_defining_types.get(&type_id).copied()
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
        name_resolution_registry
            .register_name_defining_type::<Window>(NameDefinitionPriority::LIBRARY);
        name_resolution_registry
            .register_name_defining_type::<Monitor>(NameDefinitionPriority::LIBRARY);
        name_resolution_registry
            .register_name_defining_type::<Gamepad>(NameDefinitionPriority::LIBRARY);
        name_resolution_registry
            .register_name_defining_type::<PointerId>(NameDefinitionPriority::LIBRARY);

        // UI
        name_resolution_registry
            .register_name_defining_type::<Node>(NameDefinitionPriority::FALLBACK);
        name_resolution_registry
            .register_name_defining_type::<Button>(NameDefinitionPriority::LIBRARY);
        name_resolution_registry
            .register_name_defining_type::<Text>(NameDefinitionPriority::LIBRARY);
        name_resolution_registry
            .register_name_defining_type::<TextSpan>(NameDefinitionPriority::LIBRARY);
        name_resolution_registry
            .register_name_defining_type::<Text2d>(NameDefinitionPriority::LIBRARY);
        name_resolution_registry
            .register_name_defining_type::<ImageNode>(NameDefinitionPriority::LIBRARY);
        name_resolution_registry
            .register_name_defining_type::<ViewportNode>(NameDefinitionPriority::LIBRARY);

        // Cameras
        name_resolution_registry
            .register_name_defining_type::<Camera>(NameDefinitionPriority::FALLBACK);
        name_resolution_registry
            .register_name_defining_type::<Camera2d>(NameDefinitionPriority::LIBRARY);
        name_resolution_registry
            .register_name_defining_type::<Camera3d>(NameDefinitionPriority::LIBRARY);

        // Lights
        name_resolution_registry
            .register_name_defining_type::<DirectionalLight>(NameDefinitionPriority::LIBRARY);
        name_resolution_registry
            .register_name_defining_type::<PointLight>(NameDefinitionPriority::LIBRARY);
        name_resolution_registry
            .register_name_defining_type::<SpotLight>(NameDefinitionPriority::LIBRARY);
        name_resolution_registry
            .register_name_defining_type::<AmbientLight>(NameDefinitionPriority::LIBRARY);
        name_resolution_registry
            .register_name_defining_type::<LightProbe>(NameDefinitionPriority::LIBRARY);
        name_resolution_registry
            .register_name_defining_type::<IrradianceVolume>(NameDefinitionPriority::LIBRARY);
        name_resolution_registry
            .register_name_defining_type::<SunDisk>(NameDefinitionPriority::LIBRARY);
        name_resolution_registry
            .register_name_defining_type::<Lightmap>(NameDefinitionPriority::LIBRARY);

        // Core rendering components
        name_resolution_registry
            .register_name_defining_type::<Sprite>(NameDefinitionPriority::LIBRARY);
        name_resolution_registry
            .register_name_defining_type::<Mesh2d>(NameDefinitionPriority::LIBRARY);
        name_resolution_registry
            .register_name_defining_type::<Mesh3d>(NameDefinitionPriority::LIBRARY);
        name_resolution_registry
            .register_name_defining_type::<Wireframe>(NameDefinitionPriority::LIBRARY);

        // Atmospherics
        name_resolution_registry
            .register_name_defining_type::<Skybox>(NameDefinitionPriority::LIBRARY);
        name_resolution_registry
            .register_name_defining_type::<FogVolume>(NameDefinitionPriority::LIBRARY);
        name_resolution_registry
            .register_name_defining_type::<Atmosphere>(NameDefinitionPriority::LIBRARY);
        name_resolution_registry
            .register_name_defining_type::<DistanceFog>(NameDefinitionPriority::LIBRARY);

        // Animation
        name_resolution_registry
            .register_name_defining_type::<AnimationPlayer>(NameDefinitionPriority::LIBRARY);

        // Audio
        name_resolution_registry
            .register_name_defining_type::<AudioPlayer>(NameDefinitionPriority::LIBRARY);
        name_resolution_registry
            .register_name_defining_type::<AudioSink>(NameDefinitionPriority::LIBRARY);

        // System-likes
        name_resolution_registry
            .register_name_defining_type::<Observer>(NameDefinitionPriority::LIBRARY);
        name_resolution_registry
            .register_name_defining_type::<SystemIdMarker>(NameDefinitionPriority::LIBRARY);

        app.insert_resource(name_resolution_registry);
    }
}
