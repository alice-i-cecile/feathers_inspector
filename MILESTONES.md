# Milestones

## EntityCommands::inspect

- [x] `EntityInspection` type that holds output
- [x] `impl Display for EntityInspection`
- [x] EntityRef::inspect
- [x] EntityCommands::inspect
- [x] Test in example
- [x] Displays entity name
- [x] Log-style inspection example

## Resource inspection

- [x] Create `ResourceInspection` type
- [x] Add a command + World method to log dynamically typed resource
- [x] Add a command + World method to log strongly typed resource
- [x] Add a command + World method to log all resources

## Component inspection

- [x] List components in `Inspection`
- [x] Inspect a single component (typed and untyped)

## Reflection-wiring

- [x] Extract `TypeRegistration` for inspected components and resources
- [x] Display values of components and resource using reflection
  - [x] Get [reflected component info](https://github.com/jakobhellermann/bevy-inspector-egui/blob/93fe429ba0570405094370f31d0530c1a8f0e28d/crates/bevy-inspector-egui/src/restricted_world_view.rs#L375)
  - [x] [Match on the `ReflectedRef`](https://github.com/jakobhellermann/bevy-inspector-egui/blob/93fe429ba0570405094370f31d0530c1a8f0e28d/crates/bevy-inspector-egui/src/reflect_inspector/mod.rs#L318-L319)
  - [x] Extract the [represented type info](https://github.com/jakobhellermann/bevy-inspector-egui/blob/93fe429ba0570405094370f31d0530c1a8f0e28d/crates/bevy-inspector-egui/src/reflect_inspector/mod.rs#L544)
  - [x] Read the values of `Struct` etc to format the output

## User-friendly names

- [x] Add a name resolution solution
  - [x] Can be implemented for user-defined types
  - [x] Implement on foreign types from Bevy
  - [x] Supports prioritization

## Spawn location

- [x] add spawn location to `EntityInspection`
- [x] gather and store `SpawnDetails` with more information

## Basic settings

- [x] add settings struct for each inspection method, allowing users to pass it in
- [x] allow users to toggle short vs long type names

## Inspect all entities

- [x] add an API to let users inspect multiple entities at once
- [x] add dedicated settings struct for entity inspection
- [x] add dedicated settings struct for multiple entity inspection
- [x] add on-screen instructions to log_style_inspection example

## Entity sorting

- [x] display entities of the same archetype together
- [x] sort archetypes by similarity

## Tweak inspector output for your type

- [x] define a stub `Inspectable` trait for reflected type information
- [ ] customize the precision, increment and range of numbers
- [x] hide fields from the inspector
- [ ] customize display output based on `Inspectable`
- [ ] add a derive macro

## Filtering and search

- [x] allow users to pass in a search parameter to settings for relevant inspection methods
- [x] search by component presence
- [x] search by component absence
- [x] search by name

## Text-only editing

- [x] Add a nice way to fuzzily map component / resource names to a `ComponentId`
- [x] Expose a convenient mutable reflection API: `get_reflected_component_mut` / `get_reflected_resource_mut`
- [x] Add an example for how to dynamically modify values using reflection

## Component metadata

- [x] Don't recompute component metadata for every single entity
- [x] Extract all interesting information from `ComponentInfo` (can't be stored: not Send and Sync)
- [x] Add an API to get more information about component types in general
  - [x] Full path
  - [x] Reflected docs
  - [x] How many entities have this component
- [x] Report the size in bytes of:
  - [x] Individual components
  - [x] Resources
  - [x] Entities
- [x] Report interesting metadata for resources in `ResourceInspection`

## BRP inspection

- [x] Investigate and record how BRP works
- [x] Investigate and record how to integrate a reflection-backed workflow
- [x] Implement BRP methods for feathers-inspector operations

## Basic GUI

- [x] Exclude entities that belong to the inspector GUI itself
- [x] Display a list of entities
- [x] List components under each entity
- [x] Crudely display component values
- [ ] Create a new window that inspects resources

## Tabs

- [ ] Create a simple feathers-based tab abstraction
- [ ] Split entities into tabs based on categories
- [ ] Move resources inspection into a tab
- [ ] Close and open tabs
- [ ] Reorder tabs

## Pop-out

- [x] Render a pop-up UI window
- [x] Give the UI window a title
- [x] Make it draggable
- [x] Make it resizable

## Hierarchy

- [ ] Show entities in parent-child hierarchy structure
- [ ] Add entity folding

## Asset inspection

- [ ] Add inspection capabilities for assets
- [ ] Display values for components that contain Handle

## Important entities

- [ ] Pin entities to the top of the list
- [ ] Hide entities manually

## Fancy GUI

- [ ] Add pagination

## Summary statistics

- [x] Add `World::summarize`
- [x] List total number of entities
- [x] List total number of archetypes
  - [x] Show number of entities by archetype
  - [x] Show sorted list
- [x] List total number of resources

## Categories

- [ ] Define user-extensible categories for entities to be filtered by
- [ ] Return and log category as part of `EntityInspection`

## Out-of scope

- GUI value editing
  - Needs text input
  - Lots of work
- Spawn, despawn, insert, remove components, reparenting...
  - Really wants value editing to be useful
- Advanced entity sorting
  - Exclude sparse-set components and ensure relatively stable sorting
  - Persist entity clusters and incrementally recompute
- GUI-based search
  - Requires text input to work well
