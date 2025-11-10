//! [`Inspectable`] trait to customize the inspection behavior of types.
//!
//! ## Motivation
//!
//! When inspecting types in our game world, we want fine-grained control over how our values are displayed and manipulated. We might wish to:
//!
//! - set minimum and maximum values
//! - change how the types are displayed, altering their color, number of digits shown, or string representation
//! - change how values are modified, such as by changing the increment by which they advance or by adding validation
//!
//! Ordinarily, this sort of "inspector interface" should be automatically generated on the basis of the types contained within our type of interest.
//! If a `Life` component contains an `f32`, we should be walking down its reflected tree of types, discovering that we have an `f32`, and then displaying the right widget with the correct settings automatically.
//!
//! However, not all floats are created equal!
//! The same `f32` type might represent a value that should range from 0.0 to 1.0 with three decimal places in one struct, while representing any positive, finite value in another!
//!
//! The natural instinct in Rust is to attempt to refine our types further: storing your `rating: u32` field as a `U32Range<1, 10>`, rather than trying to validate at runtime that we're in the range of 1 to 10.
//! While this can be a good solution, it comes with some problems, particularly in the context of game development.
//! These strongly-typed wrappers can:
//!
//! - add cognitive complexity when reading and writing code
//! - add severe boilerplate when creating and converting values
//! - cause serious problems during interoperation with other languages and assets
//! - increase compile time
//!
//! In the end, we need to pragmatically accept that we cannot rely solely on the types contained within our structs to determine how inspectors should handle them.
//!
//! ## Strategy
//!
//! Instead, this crate has chosen to use a specific-overrides-general model of configuration,
//! searching first for a [`Inspectable`] implementation for the component or resource of interest,
//! then for an implementation for any of its fields, then for any item within those fields and so on,
//! before finally falling back to an implementation for the primitive types
//! that compose our inspectable objects.
//!
//! While useful and seemingly natural, this model is rather at odds with Rust's stock approach to traits!
//! There's no built-in way to ask "does my field implement some trait",
//! using that behavior if so, and falling back to a default strategy if it does not.
//!
//! Even when working within a macro, the trait resolution information simply does not exist
//! at the time of parsing: all we have is a stream of tokens.
//!
//! Instead we must turn, yet again, to the dark mirror of runtime reflection.
//! The basic strategy is as follows:
//!
//! - each type may implement the [`Inspectable`] trait, which tells inspector front-ends how it should be displayed and manipulated
//!   - various helper attribute macros, such as `#[inspectable(min = 0, max = 10)]` are provided to make this faster and easier
//! - this trait implementation is registered as type data in a [`TypeRegistry`](bevy::reflect::TypeRegistry) via the [`reflect_trait`](bevy::reflect::reflect_trait) machinery
//!   - this process is semi-automated: using the [`Reflect`](bevy::reflect::Reflect) derive macro, the `#[reflect(Inspectable)]` annotation, and automatic type registration that looks for types that implement `Reflect` in your binary
//! - when attempting to resolve how a type should be inspected, we access the type registry, and recursively walk it
//!   - if an [`Inspectable`] implementation is found for the current root type, we use it
//!   - otherwise we break the type into its fields, using any implementation of `Inspectable` found for those fields
//!   - this recurses until we either hit a base type (like `f32` or `String`) which we provide a base implementation for, or we hit an unknown type that we cannot drill down further into

/// A trait used to customize the inspection behavior of types.
///
/// Types implementing this trait can provide additional metadata
/// or override default inspection logic when being inspected by various inspector front-ends.
///
/// This trait is optional: types that do not implement it will still be inspectable
/// as long as they derive [`Reflect`](bevy::reflect::Reflect).
///
/// See the [module docs](super) for more information about the fallback strategy used,
/// and the machinery involved.
pub trait Inspectable {}
