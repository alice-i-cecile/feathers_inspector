# PR Strategy

## Cleanup

- [] `ShortName` should be orderable to allow for easy sorting
- [] Rename `SystemIdMarker` to something more inspector-friendly
- [] Make it easy to get a `SpawnDetails` from an `EntityRef` without needing a whole query

## Foundations

- [] Split apart `Name` (sensu animation and scenes) from `InspectorName` (X-Controversial)

## Nice to have

- [] Allow users to register dynamically typed resources and components
- [] Add time of spawning information to `SpawnDetails` in dev mode

## Log-style inspection

- [] Implement `Display` for `TypeInfo` and contained types
- [] Pick and implement an entity name resolution strategy
