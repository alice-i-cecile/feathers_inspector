use std::any::TypeId;

use bevy::{
    ecs::{component::ComponentId, resource::IsResource, system::SystemIdMarker},
    prelude::*,
};

use crate::{
    entity_grouping::{EntityGrouping, GroupingStrategy},
    extension_methods::WorldInspectionExtensionTrait,
    gui::{
        cache::{InspectorCache, snapshot::WorldSnapshot},
        state::{InspectorInternal, InspectorState, ObjectListEntry, ObjectListTab},
    },
    inspection::{
        component_inspection::{ComponentInspectionSettings, ComponentMetadataMap},
        entity_inspection::{
            EntityInspection, EntityInspectionError, EntityInspectionSettings,
            MultipleEntityInspectionSettings,
        },
    },
    memory_size::MemorySize,
};

/// Criteria used to filter objects in both paused and running modes.
pub(crate) struct ObjectListFilter {
    pub tab: ObjectListTab,
    pub text: String,
    pub mandatory_components: Vec<ComponentId>,
}

/// Allows abstracting over paused and running mode
/// for checking component presence.
pub(crate) trait ComponentChecker {
    fn has_component<T: Component>(&self) -> bool;
}

impl<'w> ComponentChecker for EntityRef<'w> {
    fn has_component<T: Component>(&self) -> bool {
        self.contains::<T>()
    }
}

impl ComponentChecker for (&EntityInspection, &ComponentMetadataMap) {
    fn has_component<T: Component>(&self) -> bool {
        let inspection: &EntityInspection = self.0;
        let type_id = TypeId::of::<T>();
        let metadata_map: &ComponentMetadataMap = self.1;
        inspection.components.iter().flatten().any(|ci| {
            metadata_map
                .map
                .get(&ci.component_id)
                .is_some_and(|metadata| metadata.type_id == Some(type_id))
        })
    }
}

/// Exclusive system that refreshes the [`InspectorCache`].
/// We can't sensibly cache results from one call to the next,
/// as the world may have changed substantially in between.
///
/// Uses exclusive world access to avoid resource conflicts.
pub fn update_inspector_cache(world: &mut World) {
    update_component_metadata_map(world);
    let (is_paused, selected_object, filter) = {
        let state = world.resource::<InspectorState>();
        (
            state.is_paused,
            state.selected_object,
            ObjectListFilter {
                tab: state.active_objects_tab,
                text: state.filter_text.clone(),
                mandatory_components: state.mandatory_components.clone(),
            },
        )
    };

    if is_paused {
        update_cache_paused(world, &filter);
    } else {
        update_cache_running(world, selected_object, &filter);
    }
}

fn update_component_metadata_map(world: &mut World) {
    world.resource_scope(|world, mut inspector_cache: Mut<InspectorCache>| {
        let metadata_map = match inspector_cache.metadata_map.take() {
            Some(mut metadata_map) => {
                metadata_map.update(world);
                metadata_map
            }
            None => ComponentMetadataMap::generate(world),
        };
        inspector_cache.metadata_map = Some(metadata_map);
    });
}

fn update_cache_running(
    world: &mut World,
    selected_object: Option<Entity>,
    filter: &ObjectListFilter,
) {
    let object_list = generate_live_object_list(world, filter);
    let updated_snapshot = if let Some(selected) = selected_object {
        WorldSnapshot::single(world, selected)
    } else {
        WorldSnapshot::empty()
    };

    let mut cache = world.resource_mut::<InspectorCache>();
    cache.snapshot = updated_snapshot;
    cache.filtered_objects = object_list;

    // Prevents sudden writing of `RefreshCache`
    // after a forceful refresh.
    if let Some(ref mut timer) = cache.timer {
        timer.reset();
    }
}

fn generate_live_object_list(world: &mut World, filter: &ObjectListFilter) -> Vec<ObjectListEntry> {
    let entities = query_entities_for_tab(world, filter.tab);
    let inspections = inspect_entities(world, &filter.mandatory_components, entities);

    filter_inspections_and_create_entries(
        inspections.iter().filter_map(|r| r.as_ref().ok()),
        filter,
        None,
    )
}

/// Queries all entities matching the active tab, excluding UI nodes, windows, and inspector internals.
fn query_entities_for_tab(world: &mut World, tab: ObjectListTab) -> Vec<Entity> {
    let mut query = world.query::<EntityRef>();
    query
        .iter(world)
        .filter(|e| {
            if e.contains::<InspectorInternal>() {
                return false;
            }
            matches_tab(*e, tab)
        })
        .map(|e| e.id())
        .collect()
}

fn inspect_entities(
    world: &mut World,
    mandatory_components: &[ComponentId],
    entities: Vec<Entity>,
) -> Vec<Result<EntityInspection, EntityInspectionError>> {
    world.resource_scope(|world, mut cache: Mut<InspectorCache>| {
        let Some(ref mut metadata_map) = cache.metadata_map else {
            return Vec::new();
        };
        let inspection_settings = MultipleEntityInspectionSettings {
            with_component_filter: mandatory_components.to_vec(),
            ..default()
        };
        world.inspect_multiple(entities.iter().copied(), inspection_settings, metadata_map)
    })
}

fn update_cache_paused(world: &mut World, filter: &ObjectListFilter) {
    let has_full_snapshot = {
        let cache = world.resource::<InspectorCache>();
        cache.snapshot.is_full()
    };
    if !has_full_snapshot {
        create_full_snapshot(world);
    }

    world.resource_scope(|_world, mut cache: Mut<InspectorCache>| {
        let object_list = filter_inspections_and_create_entries(
            cache.snapshot.iter(),
            filter,
            cache.metadata_map.as_ref(),
        );
        cache.filtered_objects = object_list;
    });
}

fn create_full_snapshot(world: &mut World) {
    let entities_to_snapshot = get_all_inspectable_entities(world);
    let sorted_entities =
        EntityGrouping::generate(world, entities_to_snapshot, GroupingStrategy::Hierarchy)
            .flatten();

    let metadata_map = world
        .resource_mut::<InspectorCache>()
        .metadata_map
        .take()
        .unwrap();
    let settings = EntityInspectionSettings {
        include_components: true,
        component_settings: ComponentInspectionSettings {
            store_reflected_value: true,
            ..default()
        },
    };

    let mut inspections = Vec::new();
    for entity in &sorted_entities {
        let inspection = world.inspect_cached(*entity, &settings, &metadata_map);
        if let Ok(inspection) = inspection {
            inspections.push(inspection);
        }
    }

    let mut cache = world.resource_mut::<InspectorCache>();
    cache.metadata_map = Some(metadata_map);
    cache.snapshot = WorldSnapshot::full(inspections, sorted_entities);
}

/// Gets all the entities without the component [`InspectorInternal`].
fn get_all_inspectable_entities(world: &mut World) -> Vec<Entity> {
    let mut query = world.query::<EntityRef>();
    query
        .iter(world)
        .filter(|e| !e.contains::<InspectorInternal>())
        .map(|e| e.id())
        .collect()
}

pub(crate) fn filter_inspections_and_create_entries<'a>(
    inspections: impl Iterator<Item = &'a EntityInspection>,
    filter: &ObjectListFilter,
    metadata_map: Option<&ComponentMetadataMap>,
) -> Vec<ObjectListEntry> {
    inspections
        .filter(|inspection| {
            if let Some(metadata_map) = metadata_map {
                if !matches_tab((*inspection, metadata_map), filter.tab) {
                    return false;
                }

                if !filter
                    .mandatory_components
                    .iter()
                    .all(|&comp_id| has_component_by_id(inspection, comp_id))
                {
                    return false;
                }
            }
            true
        })
        .filter_map(|inspection| try_create_object_list_entry(inspection, &filter.text))
        .collect()
}

fn try_create_object_list_entry(
    inspection: &EntityInspection,
    filter_text: &str,
) -> Option<ObjectListEntry> {
    let name = inspection
        .name
        .as_ref()
        .map(|n| n.to_string())
        .unwrap_or_else(|| format!("Entity {:?}", inspection.entity));

    if !filter_text.is_empty() && !name.to_lowercase().contains(&filter_text.to_lowercase()) {
        return None;
    }

    Some(ObjectListEntry {
        entity: inspection.entity,
        display_name: name,
        component_count: inspection.components.as_ref().map(|c| c.len()).unwrap_or(0),
        memory_size: inspection.total_memory_size.unwrap_or(MemorySize::new(0)),
    })
}

fn matches_tab(checker: impl ComponentChecker, tab: ObjectListTab) -> bool {
    match tab {
        ObjectListTab::Entities => {
            !checker.has_component::<Node>()
                && !checker.has_component::<IsResource>()
                && !checker.has_component::<Observer>()
                && !checker.has_component::<SystemIdMarker>()
        }
        ObjectListTab::Resources => checker.has_component::<IsResource>(),
        ObjectListTab::Observers => checker.has_component::<Observer>(),
        ObjectListTab::OneShotSystems => checker.has_component::<SystemIdMarker>(),
    }
}

fn has_component_by_id(inspection: &EntityInspection, component_id: ComponentId) -> bool {
    inspection
        .components
        .iter()
        .flatten()
        .any(|ci| ci.component_id == component_id)
}
