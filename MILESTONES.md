# Milestones

## EntityCommands::inspect

- [] `Inspection` type that holds output
- [] `impl Display for Inspection`
- [] EntityRef::inspect
- [] EntityCommands::inspect
- [] Test in example
- [] Displays entity name
- [] Displays list of components
- [] Inspect-all example

## User-friendly names

- [] Trait-based design
- [] Proof-of-concept for custom component types
- [] Implement on foreign types from Bevy

## Categories

- [] Define categories for entities to be filtered by
- [] Return category as part of `Inspection`
- [] User-extensible categories

## Spawn location

- [] add spawn location to `Inspection`

## Inspector customization

- [] `InspectorSettings` trait
- [] Customize display output
- [] Importance score

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

## Out-of scope

- Search
  - Needs text input
- Value editing
  - Needs text input
  - Lots of work
- Spawn, despawn, insert, remove components, reparenting...
  - Really wants value editing to be useful
