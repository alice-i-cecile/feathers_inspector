//! Rules and strategies for determining the inspection-displayed name of an entity.

use crate::entity_inspection::EntityInspection;

impl EntityInspection {
    /// Determines the name to display for this entity.
    ///
    /// If the [`Name`](bevy::prelude::Name) component is present, its value will be used.
    /// Otherwise, a default string "Entity" will be returned.
    pub fn resolve_name(&self) -> &str {
        if let Some(custom_name) = &self.name {
            return custom_name.as_str();
        } else {
            "Entity"
        }
    }
}
