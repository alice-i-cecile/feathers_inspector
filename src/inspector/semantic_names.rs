//! Semantic field name registry for tuple structs.
//!
//! Provides human-readable field names (x, y, z) instead of tuple indices (.0, .1, .2)
//! for common math types like Vec2, Vec3, Vec4, Quat, etc.

use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use core::any::TypeId;

/// Registry mapping TypeIds to semantic field names for tuple structs.
///
/// This allows the inspector to display "x", "y", "z" instead of ".0", ".1", ".2"
/// for types like Vec3, making the UI more intuitive.
#[derive(Resource)]
pub struct SemanticFieldNames {
    overrides: HashMap<TypeId, Vec<&'static str>>,
}

impl Default for SemanticFieldNames {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticFieldNames {
    /// Creates a new registry pre-populated with common Bevy math types.
    pub fn new() -> Self {
        let mut registry = Self {
            overrides: HashMap::default(),
        };

        // Float vectors
        registry.register::<Vec2>(&["x", "y"]);
        registry.register::<Vec3>(&["x", "y", "z"]);
        registry.register::<Vec4>(&["x", "y", "z", "w"]);

        // Integer vectors
        registry.register::<IVec2>(&["x", "y"]);
        registry.register::<IVec3>(&["x", "y", "z"]);
        registry.register::<IVec4>(&["x", "y", "z", "w"]);

        // Unsigned integer vectors
        registry.register::<UVec2>(&["x", "y"]);
        registry.register::<UVec3>(&["x", "y", "z"]);
        registry.register::<UVec4>(&["x", "y", "z", "w"]);

        // Double vectors (use glam types directly since not re-exported by bevy::prelude)
        registry.register::<bevy::math::DVec2>(&["x", "y"]);
        registry.register::<bevy::math::DVec3>(&["x", "y", "z"]);
        registry.register::<bevy::math::DVec4>(&["x", "y", "z", "w"]);

        // Quaternion
        registry.register::<Quat>(&["x", "y", "z", "w"]);

        registry
    }

    /// Register semantic field names for a type.
    ///
    /// The names slice should match the order of fields in the tuple struct.
    pub fn register<T: 'static>(&mut self, names: &[&'static str]) {
        self.overrides.insert(TypeId::of::<T>(), names.to_vec());
    }

    /// Get the semantic field name for a tuple struct field.
    ///
    /// Returns None if the type has no registered override or the index is out of bounds.
    pub fn get_field_name(&self, type_id: TypeId, index: usize) -> Option<&'static str> {
        self.overrides.get(&type_id)?.get(index).copied()
    }

    /// Check if a type has semantic field names registered.
    pub fn has_override(&self, type_id: TypeId) -> bool {
        self.overrides.contains_key(&type_id)
    }
}
