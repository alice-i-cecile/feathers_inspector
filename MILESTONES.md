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
- [] Display values of components and resource using reflection
  - [x] Get [reflected component info](https://github.com/jakobhellermann/bevy-inspector-egui/blob/93fe429ba0570405094370f31d0530c1a8f0e28d/crates/bevy-inspector-egui/src/restricted_world_view.rs#L375)
  - [x] [Match on the `ReflectedRef`](https://github.com/jakobhellermann/bevy-inspector-egui/blob/93fe429ba0570405094370f31d0530c1a8f0e28d/crates/bevy-inspector-egui/src/reflect_inspector/mod.rs#L318-L319)
  - [x] Extract the [represented type info](https://github.com/jakobhellermann/bevy-inspector-egui/blob/93fe429ba0570405094370f31d0530c1a8f0e28d/crates/bevy-inspector-egui/src/reflect_inspector/mod.rs#L544)
  - [x] Read the values of `Struct` etc to format the output
- [ ] Improve formatting for the reflected information

## User-friendly names

- [x] Add a name resolution solution
  - [x] Can be implemented for user-defined types
  - [x] Implement on foreign types from Bevy
  - [x] Supports prioritization

## Spawn location

- [x] add spawn location to `EntityInspection`
- [x] gather and store `SpawnDetails` with more information

## Inspect all entities

- [] add an API to let users inspect multiple entities at once
- [] group entities by archetype similarity
- [] add on-screen instructions to log_style_inspection example
- [] add settings struct for each inspection method, allowing users to pass it in

## Inspector customization

- [] `InspectorSettings` trait for reflected type information
- [] customize display output based on `InspectorSettings`

## Filtering and search

- [] allow users to pass in a search parameter to settings for relevant inspection methods
- [] search by component presence
- [] search by component absence
- [] search by name

## BRP inspection

- [] Investigate and record how BRP works
- [] Investigate and record how to integrate a reflection-backed workflow

## Basic GUI

- [] Exclude entities that belong to the inspector GUI itself
- [] Display a list of entities
- [] Add pagination
- [] List components under each entity
- [] Add entity folding
- [] Crudely display component values

## Tabs

- [] Create a simple feathers-based tab abstraction
- [] Split entities into tabs based on categories
- [] Close and open tabs

## Pop-out

- [] Render a pop-up UI window
- [] Give the UI window a title
- [] Make it draggable
- [] Make it resizable

## Hierarchy

- [] Show entities in parent-child hierarchy structure

## Asset inspection

- [] Add inspection capabilities for assets
- [] Display values for components that contain Handle

## Important entities

- [] Pin entities to the top of the list
- [] Hide entities manually

## Summary statistics

- [] Add `World::summarize`
- [] List total number of entities
- [] List total number of archetypes
  - [] Show number of entities by archetype
  - [] Show sorted list
- [] List total number of resources

## Categories

- [] Define user-extensible categories for entities to be filtered by
- [] Return and log category as part of `EntityInspection`

## Out-of scope

- Value editing
  - Needs text input
  - Lots of work
- Spawn, despawn, insert, remove components, reparenting...
  - Really wants value editing to be useful

