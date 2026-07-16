# Upstreaming strategy

When upstreaming this work, there are several distinct phases:

1. Reflection enhancements: improving `bevy_reflect` with assorted access helpers and display utilities.
2. Reusable UI widgets: generic widget primitives with no inspector coupling.
3. Assorted utilities: misc work required by the inspection backend that can be done independently
4. Inspection backend + log-style front-end: code which defines the API in `bevy_dev_tools/inspector` which various inspection tools should call.
5. BRP inspection frontend.
6. Feathers-based GUI frontend.

Phases 1 and 2 are independent and can land in parallel. Several Phase 3 items (memory-size utilities, world summary, entity grouping, fuzzy name mapping) are also fully self-contained and can be pursued in parallel with Phases 1–2.

For each item, follow the format below:

```md
1. **STATUS:** Sample work item, [#12345](https://github.com/bevyengine/bevy/pulls)
  - details on scope
  - Code: [lib.rs](src/lib.rs)
  - Target: [bevy_dev_tools/inspector/inspection](https://github.com/bevyengine/bevy/tree/main/crates/bevy_dev_tools/inspector/src/inspection.rs)
```

Possible STATUS replacements include: "Needs work", "PR please", "Blocked", "Needs review", "Merged", and "Integrated".

As PRs are created, record them in the appropriate subsection, and link back to the root Bevy issue: [#23013](https://github.com/bevyengine/bevy/issues/23013).

Once PRs are merged, update this crate to latest upstream `main` and replace local internals with merged equivalents.
Once that's done, make sure everything still works then change the work status to **Integrated**.

Track the current feature status in [MILESTONES.md](MILESTONES.md); this file tracks the process of upstreaming only.

## Phase 1: Reflection enhancements

1. **Merged:** Make `to_dynamic` fallible. [#24748](https://github.com/bevyengine/bevy/pull/24748)
  - Code: Noted in [clone_incomplete](src/reflection_tools.rs)
  - Target: [bevy_reflect/src/](https://github.com/bevyengine/bevy/tree/main/crates/bevy_reflect/src)
2. **Merged:** changing values with reflection example. [#24747](https://github.com/bevyengine/bevy/pull/24747)
  - this is a useful pattern to demonstrate, but is fundamentally independent from the rest of the inspection work
  - Code: [changing_values_with_reflection example](examples/changing_values_with_reflection.rs)
  - Target: [examples/reflection/](https://github.com/bevyengine/bevy/tree/main/examples/reflection)
3. **Merged** reflection pretty printing. [#24995]
  - these should all just be `Display` impls where possible; could not do here because orphan rules 
  - Code: [reflected_value_to_string()](src/reflection_tools.rs), [pretty_print_reflected_struct()](src/reflection_tools.rs), [pretty_print_reflected_enum()](src/reflection_tools.rs), [pretty_print_reflected_opaque()](src/reflection_tools.rs)
  - Target: [bevy_reflect/src/display.rs](https://github.com/bevyengine/bevy/tree/main/crates/bevy_reflect/src)

## Phase 2: Reusable UI widgets

1. **Work needed:** headless tab container widget
  - existing implementation of this in this crate is so-so; see what Jackdaw has?
  - Code: [TabPlugin](src/gui/widgets/tabs.rs), [TabGroup](src/gui/widgets/tabs.rs), [Tab](src/gui/widgets/tabs.rs), [HasContent](src/gui/widgets/tabs.rs), [ContentOfTab](src/gui/widgets/tabs.rs), [TabContentDisplayMode](src/gui/widgets/tabs.rs), [ActivateTab](src/gui/widgets/tabs.rs), [TabActivated](src/gui/widgets/tabs.rs)
  - Target: [bevy_ui_widgets/src/tabs.rs](https://github.com/bevyengine/bevy/tree/main/crates/bevy_ui_widgets/src)
2. **Merged:** drag-value numeric input widget. Completed in [#24636](https://github.com/bevyengine/bevy/pull/24636).
  - draggable number input (like ImGui's DragFloat) with double-click-to-edit
  - Code: [DragValuePlugin](src/gui/widgets/drag_value.rs), [DragValue](src/gui/widgets/drag_value.rs), [DragValueProps](src/gui/widgets/drag_value.rs), [DragValueDragState](src/gui/widgets/drag_value.rs), [DragValueChanged](src/gui/widgets/drag_value.rs), [DragValueEditModeChanged](src/gui/widgets/drag_value.rs), [FieldPath](src/gui/widgets/drag_value.rs), [PendingValueChanges](src/gui/widgets/drag_value.rs), [apply_pending_value_changes()](src/gui/widgets/drag_value.rs)
  - Target: [bevy_feathers/src/controls/drag_value.rs](https://github.com/bevyengine/bevy/tree/main/crates/bevy_feathers/src/controls)

## Phase 3: Assorted utilities

1. **Review please:** memory-size utility types and formatting helpers [#25006](https://github.com/bevyengine/bevy/pull/25006)
  - Code: [MemorySize](src/memory_size.rs) and everything else in that file
  - Target: [bevy_diagnostic/src/memory_size.rs](https://github.com/bevyengine/bevy/tree/main/crates/bevy_diagnostic/src)
2. **Review please:** name resolution [#25007](https://github.com/bevyengine/bevy/pull/25007)
  - Code: [EntityName](src/entity_name_resolution/mod.rs), [NameOrigin](src/entity_name_resolution/mod.rs), [NameDefinitionPriority](src/entity_name_resolution/mod.rs), [ComponentNameData](src/entity_name_resolution/mod.rs), [resolve_name()](src/entity_name_resolution/mod.rs), [NameResolutionRegistry](src/entity_name_resolution/mod.rs), [NameResolutionPlugin](src/entity_name_resolution/mod.rs)
  - Target: [bevy_dev_tools/inspector/src/entity/name_resolution.rs](https://github.com/bevyengine/bevy/tree/main/crates/bevy_dev_tools/inspector/src/entity) + [bevy_dev_tools/inspector/src/entity/fuzzy_name_mapping.rs](https://github.com/bevyengine/bevy/tree/main/crates/bevy_dev_tools/inspector/src/entity)
3. **PR please:** entity grouping primitives
  - Code: [GroupingStrategy](src/entity_grouping/mod.rs), [EntityGrouping](src/entity_grouping/mod.rs), [EntityGrouping::generate()](src/entity_grouping/mod.rs), [archetype_similarity_grouping](src/entity_grouping/archetype_similarity_grouping.rs), [hierarchy_grouping](src/entity_grouping/hierarchy_grouping.rs)
  - Target: [bevy_dev_tools/inspector/src/inspection/grouping.rs](https://github.com/bevyengine/bevy/tree/main/crates/bevy_dev_tools/inspector/src)
4. **PR please:** world summary types and entry points
  - Code: [SummarySettings](src/inspection/world_summary.rs), [ArchetypeSummary](src/inspection/world_summary.rs), [WorldSummary](src/inspection/world_summary.rs), [WorldSummaryExt::summarize()](src/inspection/world_summary.rs), [CommandsSummaryExt::summarize()](src/inspection/world_summary.rs)
  - Target: [bevy_dev_tools/inspector/src/inspection/world_summary.rs](https://github.com/bevyengine/bevy/tree/main/crates/bevy_dev_tools/inspector/src)
5. **PR please:** fuzzy name mapping
  - Code: [fuzzy_name_mapping.rs](fuzzy_name_mapping.rs)
  - Target: [`bevy_dev_tools/inspector/fuzzy_name_mapping.rs`](https://github.com/bevyengine/bevy/tree/main/crates/bevy_dev_tools/src/inspector/)

## Phase 4: Inspection backend + log-style front-end

Settings types (`EntityInspectionSettings`, `ComponentInspectionSettings`, etc.) will be upstreamed incrementally: each PR adds the fields it needs, rather than landing every field up front.

1. **PR please:** core inspection output types + `Display` formatting + world/command inspection entry points + clone_incomplete
  - Code: [EntityInspection](src/inspection/entity_inspection.rs), [ResourceInspection](src/inspection/resource_inspection.rs), [ComponentInspection](src/inspection/component_inspection.rs), [impl Display for EntityInspection](src/inspection/entity_inspection.rs), [impl Display for ResourceInspection](src/inspection/resource_inspection.rs), [impl Display for ComponentInspection](src/inspection/component_inspection.rs), [WorldInspectionExtensionTrait](src/extension_methods.rs), [inspect()](src/extension_methods.rs), [inspect_cached()](src/extension_methods.rs), [inspect_component_by_id()](src/extension_methods.rs), [inspect_resource_by_id()](src/extension_methods.rs), [inspect_all_resources()](src/extension_methods.rs), [clone_incomplete](src/reflection_tools.rs)
  - Target: [bevy_dev_tools/inspector/src/inspection/](https://github.com/bevyengine/bevy/tree/main/crates/bevy_dev_tools/inspector/src)
2. **Blocked:** component metadata inspection
  - Code: [ComponentTypeMetadata](src/inspection/component_inspection.rs), [ComponentTypeMetadata::new()](src/inspection/component_inspection.rs), [ComponentTypeInspection](src/inspection/component_inspection.rs), [ComponentMetadataMap::generate()](src/inspection/component_inspection.rs), [hash_map_component_id_component_type_metadata](src/serde_conversions.rs)
  - Target: [bevy_dev_tools/inspector/src/inspection/](https://github.com/bevyengine/bevy/tree/main/crates/bevy_dev_tools/inspector/src)
3. **Blocked:** search/filter primitives
  - Code: [NameFilter](src/inspection/entity_inspection.rs), [filter_entity_list_for_inspection()](src/inspection/entity_inspection.rs)
  - Target: [bevy_dev_tools/inspector/src/inspection/](https://github.com/bevyengine/bevy/tree/main/crates/bevy_dev_tools/inspector/src)
4. **Blocked:** multi-inspection + sorting/grouping integration
  - introduces `MultipleEntityInspectionSettings`; `GroupingStrategy`, `EntityGrouping`, and `WorldSummary` already landed in Phase 3
  - Code: [MultipleEntityInspectionSettings](src/inspection/entity_inspection.rs), [inspect_multiple()](src/extension_methods.rs)
  - Target: [bevy_dev_tools/inspector/src/inspection/](https://github.com/bevyengine/bevy/tree/main/crates/bevy_dev_tools/inspector/src)
5. **Blocked:** log-style inspection example
  - Code: [log_style_inspection example](examples/log_style_inspection.rs)
  - Target: [examples/ecs/](https://github.com/bevyengine/bevy/tree/main/examples/ecs)

## Phase 5: BRP inspection frontend

1. **Blocked:** BRP core registration + primary verbs
  - register verbs via `RemotePlugin::with_method()`, following the pattern in [builtin_methods.rs](https://github.com/bevyengine/bevy/tree/main/crates/bevy_remote/src/builtin_methods.rs)
  - Target: [bevy_remote/src/inspection_methods.rs](https://github.com/bevyengine/bevy/tree/main/crates/bevy_remote/src) 
  - Code: [InspectorBrpPlugin](src/brp/mod.rs), [register_remote_method()](src/brp/mod.rs), [component_type_to_metadata()](src/brp/mod.rs), [inspect::VerbPlugin](src/brp/inspect.rs), [inspect_cached::VerbPlugin](src/brp/inspect_cached.rs), [inspect_resource::VerbPlugin](src/brp/inspect_resource.rs), [inspect_component::VerbPlugin](src/brp/inspect_component.rs), [inspect_component_type::VerbPlugin](src/brp/inspect_component_type.rs), [serde_conversion](src/serde_conversion.rs)
2. **Blocked:** BRP remaining verbs
  - batch/query verbs, metadata generation, fuzzy name resolution, and world summary
  - Code: [inspect_multiple::VerbPlugin](src/brp/inspect_multiple.rs), [inspect_all_resources::VerbPlugin](src/brp/inspect_all_resources.rs), [component_metadata_map_generate::VerbPlugin](src/brp/component_metadata_map_generate.rs), [fuzzy_component_name_to_name::VerbPlugin](src/brp/fuzzy_component_name_to_name.rs), [fuzzy_resource_name_to_name::VerbPlugin](src/brp/fuzzy_resource_name_to_name.rs), [summarize_world::VerbPlugin](src/brp/summarize_world.rs)
  - Target: [bevy_remote/src/inspection_methods.rs](https://github.com/bevyengine/bevy/tree/main/crates/bevy_remote/src)
3. **Blocked:** BRP inspection examples
  - Code: [brp_client example](examples/brp_client.rs), [brp_server example](examples/brp_server.rs)
  - Target: [examples/remote/](https://github.com/bevyengine/bevy/tree/main/examples/remote)

## Phase 6: Feathers-based GUI frontend

1. **Blocked:** inspector cache layer
  - not super happy about the design here; think critically before upstreaming
  - Code: [InspectorCache](src/gui/cache/mod.rs), [cache snapshot](src/gui/cache/snapshot.rs), [cache systems](src/gui/cache/systems.rs), [InspectorConfig](src/gui/config.rs), [InspectorState](src/gui/state.rs), [RefreshCache](src/gui/plugin.rs)
  - Target: `bevy_dev_tools/inspector/cache`
2. **Blocked:** inspector panels, plugin, and example
  - depends on P6.1 for the crate and P6.2 for the cache layer
  - uses `feathers` widgets for styled controls, but do not block on adding more. Imperfect functionality is okay!
  - targeting MVP quality: something broadly useful but incomplete
  - probably easier to rewrite from scratch, but you can look at this crate for inspiration
  - Code: [InspectorWindowPlugin](src/gui/plugin.rs), [SetInspectorWindow](src/gui/plugin.rs), [spawn_object_list_panel()](src/gui/panels/object_list.rs), [spawn_detail_panel()](src/gui/panels/detail_panel.rs), [render_object_list()](src/gui/panels/object_list.rs), [render_detail_panel()](src/gui/panels/detail_panel.rs), [inspector_window example](examples/inspector_window.rs)
  - Target: `bevy_dev_tools/inspector/gui`
