//! UI panels for the inspector.
//!
//! These modules are the "view" components of the inspector UI,
//! responsible for rendering the various panels and their contents.
//!
//! This data is driven by the central [`InspectorState`](super::state::InspectorState) resource,
//! and updated via systems defined in these modules.

pub mod detail_panel;
pub mod object_list;

pub use detail_panel::*;
pub use object_list::*;
