//! [`Inspectable`] trait to customize the inspection behavior of types.
//!
//! This module is heavily inspired by bevy-inspector-egui's
//! [`InspectorOptions`](https://github.dev/jakobhellermann/bevy-inspector-egui/blob/main/crates/bevy-inspector-egui/examples/basic/inspector_options.rs

/// A trait used to customize the inspection behavior of types.
///
/// Types implementing this trait can provide additional metadata
/// or override default inspection logic when being inspected by various inspector front-ends.
///
/// This trait is optional: types that do not implement it will still be inspectable
/// as long as they derive [`Reflect`](bevy::reflect::Reflect).
pub trait Inspectable {}
