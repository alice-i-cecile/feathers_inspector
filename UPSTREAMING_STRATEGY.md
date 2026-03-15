# Upstreaming strategy

When upstreaming this work, there are several distinct phases:

1. Reflection enhancements: enriching Bevy's core reflection layer with access helpers and display utilities.
2. Reusable UI widgets: generic widget primitives with no inspector coupling.
3. Assorted utilities: misc work required by the inspection backend that can be done independently
4. Inspection backend + log-style front-end: code which defines the API in `bevy_ecs` which various inspection tools should call.
5. BRP inspection frontend.
6. Feathers-based GUI frontend.

Phases 1 and 2 are independent and can land in parallel. Several Phase 3 items (memory-size utilities, world summary, entity grouping, fuzzy name mapping) are also fully self-contained and can be pursued in parallel with Phases 1–2.

For each item, follow the format below:

```md
1. **STATUS:** Sample work item, [#12345](https://github.com/bevyengine/bevy/pulls)
  - details on scope
  - Code: [lib.rs](src/lib.rs)
  - Target: [bevy_ecs/inspection](https://github.com/bevyengine/bevy/tree/main/crates/bevy_ecs/src/inspection.rs)
```

Possible STATUS replacements include: "Needs work", "PR please", "Blocked", "Needs review", "Merged", and "Integrated".

As PRs are created, link them in the appropriate subsection.
Once they are merged, update this crate to latest upstream `main` and replace local internals with merged equivalents.
Once that's done, make sure everything still works then check off the associated  elements.

Track the current feature status in [MILESTONES.md](MILESTONES.md); this file tracks the process of upstreaming only.

## Phase 1: Reflection enhancements

1. **Needs work:** reflection access helpers for components and resources
  - error type, component-level and resource-level reflected accessors, extending the existing `get_reflect()` / `get_reflect_mut()` pattern
  - resource accessors use dedicated resource entities internally but share the same error type and target file
  - **Naming problems:** the existing file already defines `get_reflect()` / `get_reflect_mut()` and its own error types. Audit for naming conflicts with our `get_reflected_component_ref()` / `get_reflected_component_mut()` / `ReflectionFetchError` and align with the established naming conventions before opening a PR.
  - Code: [ReflectionFetchError](src/reflection_tools.rs), [get_reflected_component_ref()](src/reflection_tools.rs), [get_reflected_component_mut()](src/reflection_tools.rs), [get_reflected_resource_ref()](src/reflection_tools.rs), [get_reflected_resource_mut()](src/reflection_tools.rs)
  - Target: [bevy_ecs/src/world/reflect.rs](https://github.com/bevyengine/bevy/tree/main/crates/bevy_ecs/src/world/reflect.rs) (extend existing)
2. **PR please:** reflection safety and cloning utilities
  - pure reflection utilities with zero ECS coupling: recursive safety check for dynamic conversion and a safe cloning helper
  - Code: [is_dynamic_safe()](src/reflection_tools.rs), [clone_partial_reflect()](src/reflection_tools.rs)
  - Target: [bevy_reflect/src/](https://github.com/bevyengine/bevy/tree/main/crates/bevy_reflect/src) (these operate on `PartialReflect` only, no ECS types needed)
3. **PR please:** semantic field name registry for tuple structs
  - maps tuple indices to human-readable names (e.g. "x", "y", "z") for common math types
  - standalone utility; useful for any reflection-based UI, not just the inspector
  - Code: [SemanticFieldNames](src/gui/semantic_names.rs)
  - Target: [bevy_reflect/src/semantic_names.rs](https://github.com/bevyengine/bevy/tree/main/crates/bevy_reflect/src) (new file; operates purely on reflected type info)
4. **PR please:** changing values with reflection example
  - this is useful to demonstrate, but is fundamentally independent of the rest of the inspection work
  - Code: [changing_values_with_reflection example](examples/changing_values_with_reflection.rs)
  - Target: [examples/reflection/](https://github.com/bevyengine/bevy/tree/main/examples/reflection) (new example)

## Phase 2: Reusable UI widgets

1. **PR please:** headless tab container widget
  - generic TabGroup/Tab system with activation events and content visibility management; zero inspector dependencies
  - already uses `bevy::ui_widgets::Activate`, confirming alignment with the headless widget layer
  - crate placement note: this is a headless behavior-only widget (activation state, visibility toggling) with no styling. Reviewers may suggest `bevy_feathers` instead, but `bevy_feathers` wraps `bevy_ui_widgets` with styled controls — and this widget has no style opinions. If reviewers push back, the fallback is `bevy_feathers/src/containers/tabs.rs`.
  - Code: [TabPlugin](src/gui/widgets/tabs.rs), [TabGroup](src/gui/widgets/tabs.rs), [Tab](src/gui/widgets/tabs.rs), [HasContent](src/gui/widgets/tabs.rs), [ContentOfTab](src/gui/widgets/tabs.rs), [TabContentDisplayMode](src/gui/widgets/tabs.rs), [ActivateTab](src/gui/widgets/tabs.rs), [TabActivated](src/gui/widgets/tabs.rs)
  - Target: [bevy_ui_widgets/src/tabs.rs](https://github.com/bevyengine/bevy/tree/main/crates/bevy_ui_widgets/src) (new file; headless widget alongside existing Activate event)
2. **Needs work:** drag-value numeric input widget
  - draggable number input (like ImGui's DragFloat) with double-click-to-edit; reusable as a styled control
  - `apply_pending_value_changes()` and `FieldPath` currently use `get_reflected_component_mut()` for ECS write-back. Split the core widget (drag/edit behavior + `DragValueChanged` event) from the reflection-based write-back, which should stay with the inspector.
  - Splitting strategy:
    - Upstream to `bevy_feathers`: `DragValuePlugin`, `DragValue`, `DragValueProps`, `DragValueDragState`, `DragValueChanged`, `DragValueEditModeChanged` — the pure input control with no ECS reflection coupling. `DragValueChanged` events carry the new `f64` value and consumers decide what to do with it.
    - Keep in inspector (Phase 6): `FieldPath`, `PendingValueChanges`, `apply_pending_value_changes()` — these consume `DragValueChanged` events and use `get_reflected_component_mut()` to write values back into ECS components. They stay in `bevy_dev_tools/src/inspector/` as part of the GUI frontend.
  - Code: [DragValuePlugin](src/gui/widgets/drag_value.rs), [DragValue](src/gui/widgets/drag_value.rs), [DragValueProps](src/gui/widgets/drag_value.rs), [DragValueDragState](src/gui/widgets/drag_value.rs), [DragValueChanged](src/gui/widgets/drag_value.rs), [DragValueEditModeChanged](src/gui/widgets/drag_value.rs), [FieldPath](src/gui/widgets/drag_value.rs), [PendingValueChanges](src/gui/widgets/drag_value.rs), [apply_pending_value_changes()](src/gui/widgets/drag_value.rs)
  - Target: [bevy_feathers/src/controls/drag_value.rs](https://github.com/bevyengine/bevy/tree/main/crates/bevy_feathers/src/controls) (new file; bevy_feathers already has slider, checkbox, and other input controls)

## Phase 3: Assorted utilities

1. **PR please:** memory-size utility types and formatting helpers
  - isolate reusable byte-size types/helpers with docs/tests
  - Code: [MemorySize](src/memory_size.rs), [MemoryUnit](src/memory_size.rs), [MemorySize::appropriate_unit()](src/memory_size.rs), [impl Display for MemorySize](src/memory_size.rs)
  - Target: [bevy_diagnostic/src/memory_size.rs](https://github.com/bevyengine/bevy/tree/main/crates/bevy_diagnostic/src) (new file; `bevy_diagnostic` already deals with system measurements. Avoid `bevy_utils`, which is being actively shrunk and discourages new additions.)
2. **PR please:** serde conversion helpers (inspection-agnostic subset)
  - only the modules that don't depend on inspection output types
  - `serialize_spawn_details()` has been verified to depend only on `bevy::ecs` types (no inspection imports), so it belongs here
  - the remaining inspection-dependent helper (`hash_map_component_id_component_type_metadata`) stays in Phase 4 alongside the types it serializes
  - Code: [component_id](src/serde_conversions.rs), [archetype_id](src/serde_conversions.rs), [slice_component_id](src/serde_conversions.rs), [debug_name](src/serde_conversions.rs), [storage_type](src/serde_conversions.rs), [option_vec_debug_name](src/serde_conversions.rs), [serialize_spawn_details()](src/serde_conversions.rs)
  - Target: [bevy_remote/src/serde_conversions.rs](https://github.com/bevyengine/bevy/tree/main/crates/bevy_remote/src) (new file, alongside existing builtin_methods.rs)
3. **Needs work:** name resolution
  - entity name resolution system and fuzzy name mapping utilities
  - circular dependency must be fixed before upstreaming: `resolve_name()` currently takes `&Option<Vec<ComponentInspection>>` and `&HashMap<ComponentId, ComponentTypeMetadata>` — both Phase 4 types. Meanwhile, `ComponentInspection::new()` reads `NameResolutionRegistry` from the world. This bidirectional dependency prevents either module from landing first.
    - required refactoring: change `resolve_name()` to accept `&[(ComponentId, &str, Option<NameDefinitionPriority>)]` — a simple slice of component IDs, short names, and priorities that callers assemble from whatever data they have. Move `NameDefinitionPriority` into this module (it's a simple enum/integer, not an inspection type). This breaks the cycle: name resolution becomes a leaf, and `ComponentInspection` calls it with data it already has.
  - **Fuzzy name mapping** (`fuzzy_component_name_to_id()`, `fuzzy_resource_name_to_id()`) has zero internal dependencies and can be split into a separate PR that lands independently.
  - Code: [EntityName](src/entity_name_resolution/mod.rs), [NameOrigin](src/entity_name_resolution/mod.rs), [resolve_name()](src/entity_name_resolution/mod.rs), [NameResolutionRegistry](src/entity_name_resolution/mod.rs), [NameResolutionPlugin](src/entity_name_resolution/mod.rs), [fuzzy_component_name_to_id()](src/entity_name_resolution/fuzzy_name_mapping.rs), [fuzzy_resource_name_to_id()](src/entity_name_resolution/fuzzy_name_mapping.rs)
  - Target: [bevy_ecs/src/entity/name_resolution.rs](https://github.com/bevyengine/bevy/tree/main/crates/bevy_ecs/src/entity) (new module; `Name` component already lives in bevy_ecs)
4. **PR please:** world summary types and entry points
  - self-contained world summary with zero internal dependencies; can land in parallel with Phases 1–2
  - originally grouped with multi-inspection in Phase 4, but code analysis confirms this module has no `use crate::` imports — it depends only on `bevy::ecs` types
  - Code: [SummarySettings](src/inspection/world_summary.rs), [ArchetypeSummary](src/inspection/world_summary.rs), [WorldSummary](src/inspection/world_summary.rs), [WorldSummaryExt::summarize()](src/inspection/world_summary.rs), [CommandsSummaryExt::summarize()](src/inspection/world_summary.rs)
  - Target: [bevy_ecs/src/inspection/world_summary.rs](https://github.com/bevyengine/bevy/tree/main/crates/bevy_ecs/src) (new file; can land before the rest of the inspection module)
5. **PR please:** entity grouping primitives
  - self-contained grouping strategies with zero internal dependencies; can land in parallel with Phases 1–2
  - Code: [GroupingStrategy](src/entity_grouping/mod.rs), [EntityGrouping](src/entity_grouping/mod.rs), [EntityGrouping::generate()](src/entity_grouping/mod.rs), [archetype_similarity_grouping](src/entity_grouping/archetype_similarity_grouping.rs), [hierarchy_grouping](src/entity_grouping/hierarchy_grouping.rs)
  - Target: [bevy_ecs/src/inspection/grouping.rs](https://github.com/bevyengine/bevy/tree/main/crates/bevy_ecs/src) (new file)

## Phase 4: Inspection backend + log-style front-end

Settings types (`EntityInspectionSettings`, `ComponentInspectionSettings`, etc.) will be upstreamed incrementally: each PR adds the fields it needs, rather than landing every field up front.

1. **Blocked:** core inspection output types + `Display` formatting
  - `EntityInspection`/`ResourceInspection`/component output shape and text formatting behavior
  - Code: [EntityInspection](src/inspection/entity_inspection.rs), [ResourceInspection](src/inspection/resource_inspection.rs), [ComponentInspection](src/inspection/component_inspection.rs), [impl Display for EntityInspection](src/inspection/entity_inspection.rs), [impl Display for ResourceInspection](src/inspection/resource_inspection.rs), [impl Display for ComponentInspection](src/inspection/component_inspection.rs)
  - Target: [bevy_ecs/src/inspection/](https://github.com/bevyengine/bevy/tree/main/crates/bevy_ecs/src) (new module)
2. **Blocked:** reflection-backed value rendering
  - convert reflected values to human-readable strings for display
  - Code: [reflected_value_to_string()](src/reflection_tools.rs), [pretty_print_reflected_struct()](src/reflection_tools.rs), [pretty_print_reflected_enum()](src/reflection_tools.rs), [pretty_print_reflected_opaque()](src/reflection_tools.rs)
  - Target: [bevy_reflect/src/display.rs](https://github.com/bevyengine/bevy/tree/main/crates/bevy_reflect/src) (new file; complements existing `debug()` on `PartialReflect`)
3. **Blocked:** metadata types
  - component/resource metadata surfaces: type-level information about components that the inspection system needs
  - split from entry-point methods (P4.4) to keep the PR focused and reviewable
  - Code: [ComponentTypeMetadata](src/inspection/component_inspection.rs), [ComponentTypeMetadata::new()](src/inspection/component_inspection.rs), [ComponentTypeInspection](src/inspection/component_inspection.rs), [ComponentMetadataMap::generate()](src/inspection/component_inspection.rs), [hash_map_component_id_component_type_metadata](src/serde_conversions.rs)
  - Target: [bevy_ecs/src/inspection/](https://github.com/bevyengine/bevy/tree/main/crates/bevy_ecs/src) (extend new module from P4.1)
4. **Blocked:** world/command inspection entry points
  - the `World` and `Commands` extension APIs that consume metadata and output types to perform inspection
  - depends on P4.1 (output types), P4.3 (metadata types), and Phase 3 items (name resolution, memory size)
  - Code: [WorldInspectionExtensionTrait](src/extension_methods.rs), [inspect()](src/extension_methods.rs), [inspect_cached()](src/extension_methods.rs), [inspect_component_by_id()](src/extension_methods.rs), [inspect_resource_by_id()](src/extension_methods.rs), [inspect_all_resources()](src/extension_methods.rs)
  - Target: [bevy_ecs/src/inspection/](https://github.com/bevyengine/bevy/tree/main/crates/bevy_ecs/src) (extend new module from P4.1)
5. **Blocked:** search/filter primitives
  - search predicates and entity list filtering; extends settings types with filter fields
  - Code: [NameFilter](src/inspection/entity_inspection.rs), [filter_entity_list_for_inspection()](src/inspection/entity_inspection.rs)
  - Target: [bevy_ecs/src/inspection/](https://github.com/bevyengine/bevy/tree/main/crates/bevy_ecs/src) (extend new module from P4.1)
6. **Blocked:** multi-inspection + sorting/grouping integration
  - inspect-all APIs and grouping/sorting hooks; consumes the grouping primitives landed in Phase 3
  - introduces `MultipleEntityInspectionSettings`; `GroupingStrategy`, `EntityGrouping`, and `WorldSummary` already landed in Phase 3
  - Code: [MultipleEntityInspectionSettings](src/inspection/entity_inspection.rs), [inspect_multiple()](src/extension_methods.rs)
  - Target: [bevy_ecs/src/inspection/](https://github.com/bevyengine/bevy/tree/main/crates/bevy_ecs/src) (extend new module from P4.1)
7. **Blocked:** log-style example integration over upstream APIs
  - ensure example demonstrates only stabilized backend contracts
  - Code: [log_style_inspection example](examples/log_style_inspection.rs)
  - Target: [examples/ecs/](https://github.com/bevyengine/bevy/tree/main/examples/ecs) (new example)

## Phase 5: BRP inspection frontend

1. **Blocked:** BRP core registration + primary verbs
  - core plugin structure, error codes, and the primary inspection verbs that cover the most common use cases
  - these verbs directly mirror the Phase 4 entry points and should be reviewed together for API consistency
  - Code: [InspectorBrpPlugin](src/brp/mod.rs), [register_remote_method()](src/brp/mod.rs), [component_type_to_metadata()](src/brp/mod.rs), [inspect::VerbPlugin](src/brp/inspect.rs), [inspect_cached::VerbPlugin](src/brp/inspect_cached.rs), [inspect_resource::VerbPlugin](src/brp/inspect_resource.rs), [inspect_component::VerbPlugin](src/brp/inspect_component.rs), [inspect_component_type::VerbPlugin](src/brp/inspect_component_type.rs)
  - Target: [bevy_remote/src/inspection_methods.rs](https://github.com/bevyengine/bevy/tree/main/crates/bevy_remote/src) (new file; register verbs via `RemotePlugin::with_method()`, following the pattern in [builtin_methods.rs](https://github.com/bevyengine/bevy/tree/main/crates/bevy_remote/src/builtin_methods.rs))
2. **Blocked:** BRP remaining verbs
  - batch/query verbs, metadata generation, fuzzy name resolution, and world summary
  - depends on P5.1 for the core registration infrastructure
  - Code: [inspect_multiple::VerbPlugin](src/brp/inspect_multiple.rs), [inspect_all_resources::VerbPlugin](src/brp/inspect_all_resources.rs), [component_metadata_map_generate::VerbPlugin](src/brp/component_metadata_map_generate.rs), [fuzzy_component_name_to_name::VerbPlugin](src/brp/fuzzy_component_name_to_name.rs), [fuzzy_resource_name_to_name::VerbPlugin](src/brp/fuzzy_resource_name_to_name.rs), [summarize_world::VerbPlugin](src/brp/summarize_world.rs)
  - Target: [bevy_remote/src/inspection_methods.rs](https://github.com/bevyengine/bevy/tree/main/crates/bevy_remote/src) (extend file from P5.1)
3. **Blocked:** BRP inspection examples
  - client/server examples demonstrating the BRP inspection API end-to-end
  - Code: [brp_client example](examples/brp_client.rs), [brp_server example](examples/brp_server.rs)
  - Target: [examples/remote/](https://github.com/bevyengine/bevy/tree/main/examples/remote) (new examples)

## Phase 6: Feathers-based GUI frontend

1. **Blocked:** inspector cache layer
  - data caching, snapshot management, and refresh systems that bridge the inspection backend (Phase 4) with the GUI
  - this is the data layer and can be reviewed independently of the visual panels
  - Code: [InspectorCache](src/gui/cache/mod.rs), [cache snapshot](src/gui/cache/snapshot.rs), [cache systems](src/gui/cache/systems.rs), [InspectorConfig](src/gui/config.rs), [InspectorState](src/gui/state.rs), [RefreshCache](src/gui/plugin.rs)
  - Target: [bevy_dev_tools/src/inspector/cache.rs](https://github.com/bevyengine/bevy/tree/main/crates/bevy_dev_tools/src) (new module; bevy_dev_tools already hosts FPS overlay and other developer tools)
2. **Blocked:** inspector panels, plugin, and example
  - the visual inspector panels (object list + detail view), top-level plugin, and example
  - depends on P6.1 for the cache layer; uses [bevy_feathers](https://github.com/bevyengine/bevy/tree/main/crates/bevy_feathers) for styled controls
  - target is MVP: something broadly useful but incomplete; review and polish should be done incrementally in this repo
  - Code: [InspectorWindowPlugin](src/gui/plugin.rs), [SetInspectorWindow](src/gui/plugin.rs), [spawn_object_list_panel()](src/gui/panels/object_list.rs), [spawn_detail_panel()](src/gui/panels/detail_panel.rs), [render_object_list()](src/gui/panels/object_list.rs), [render_detail_panel()](src/gui/panels/detail_panel.rs), [inspector_window example](examples/inspector_window.rs)
  - Target: [bevy_dev_tools/src/inspector/](https://github.com/bevyengine/bevy/tree/main/crates/bevy_dev_tools/src) (extend module from P6.1)
