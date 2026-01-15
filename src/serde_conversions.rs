//! Serde helper modules for value conversions.

use bevy::ecs::{
    entity::EntityNotSpawnedError,
    query::{QueryEntityError, SpawnDetails},
};
use serde::Serialize;

/// Serde helper module to serialize [`ComponentId`] as its underlying integer index.
///
/// ## Usage
///
/// Add `#[serde(with = "crate::serde_conversions::component_id")]`
/// to the struct's [`ComponentId`] field.
///
/// [`ComponentId`]: bevy::ecs::component::ComponentId
pub mod component_id {
    use bevy::ecs::component::ComponentId;
    use serde::{Deserialize, Serialize};

    /// Serializes a [`ComponentId`] into its index.
    pub fn serialize<S>(id: &ComponentId, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        id.index().serialize(serializer)
    }

    /// Deserializes the index of a [`ComponentId`].
    pub fn deserialize<'de, D>(deserializer: D) -> Result<ComponentId, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let index = usize::deserialize(deserializer)?;
        Ok(ComponentId::new(index))
    }
}

/// Serde helper module to serialize [`ArchetypeId`] as its underlying integer index.
///
/// ## Usage
///
/// Add `#[serde(with = "crate::serde_conversions::archetype_id")]`
/// to the struct's [`ArchetypeId`] field.
///
/// [`ArchetypeId`]: bevy::ecs::archetype::ArchetypeId
pub mod archetype_id {
    use bevy::ecs::archetype::ArchetypeId;
    use serde::{Deserialize, Serialize};

    /// Serializes an [`ArchetypeId`] into its index.
    pub fn serialize<S>(id: &ArchetypeId, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        id.index().serialize(serializer)
    }

    /// Deserializes the index of an [`ArchetypeId`].
    pub fn deserialize<'de, D>(deserializer: D) -> Result<ArchetypeId, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let index = usize::deserialize(deserializer)?;
        Ok(ArchetypeId::new(index))
    }
}

/// Serde helper module to serialize a slice of [`ComponentId`]s as a `Vec` of indexes.
///
/// ## Usage
///
/// Add `#[serde(with = "crate::serde_conversions::vec_component_id")]`
/// to the struct's `Vec<ComponentId>` field.
///
/// [`ComponentId`]: bevy::ecs::component::ComponentId
pub mod slice_component_id {
    use bevy::ecs::component::ComponentId;
    use serde::{Deserialize, ser::SerializeSeq};

    /// Serializes a `Vec<[ComponentId]>` into a `Vec` of indexes.
    pub fn serialize<S>(ids: &[ComponentId], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(ids.len()))?;
        for id in ids {
            seq.serialize_element(&id.index())?;
        }
        seq.end()
    }

    /// Deserializes a `Vec` of indexes to a  `Vec<[ComponentId]>`.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<ComponentId>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let indexes: Vec<usize> = Vec::deserialize(deserializer)?;
        Ok(indexes.into_iter().map(ComponentId::new).collect())
    }
}

/// Serde helper module to serialize [`DebugName`] as a string.
///
/// ## Usage
///
/// Add `#[serde(with = "crate::serde_conversions::debug_name")]`
/// to the struct's [`DebugName`] field.
///
/// [`DebugName`]: bevy::utils::prelude::DebugName
pub mod debug_name {
    use bevy::utils::prelude::DebugName;
    use serde::{Deserialize, Serialize};

    /// Serializes a [`DebugName`] into a [`String`].
    pub fn serialize<S>(name: &DebugName, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        name.to_string().serialize(serializer)
    }

    /// Deserializes a [`String`] into a [`DebugName`].
    pub fn deserialize<'de, D>(deserializer: D) -> Result<DebugName, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let name = String::deserialize(deserializer)?;
        Ok(DebugName::from(name))
    }
}

/// Serde helper module to serialize [`StorageType`] as a string.
///
/// ## Usage
///
/// Add `#[serde(with = "crate::serde_conversions::storage_type")]`
/// to the struct's [`StorageType`] field.
///
/// [`StorageType`]: bevy::ecs::component::StorageType
pub mod storage_type {
    use bevy::ecs::component::StorageType;
    use serde::{Deserialize, Serialize};

    /// Serializes a [`StorageType`] into a [`String`].
    pub fn serialize<S>(storage_type: &StorageType, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        format!("{storage_type:?}").serialize(serializer)
    }

    /// Deserializes a [`String`] into a [`StorageType`].
    pub fn deserialize<'de, D>(deserializer: D) -> Result<StorageType, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "Table" => Ok(StorageType::Table),
            "SparseSet" => Ok(StorageType::SparseSet),
            other => Err(serde::de::Error::unknown_variant(
                other,
                &["Table", "SparseSet"],
            )),
        }
    }
}

/// Serde helper module to serialize a `HashMap<ComponentId, ComponentTypeMetadata>`
/// where keys are serialized as integer indexes.
///
/// ## Usage
///
/// Add `#[serde(with = "crate::serde_conversions::hash_map_component_id_component_type_metadata")]`
/// to the struct's `HashMap<ComponentId, ComponentTypeMetadata>` field.
pub mod hash_map_component_id_component_type_metadata {
    use crate::component_inspection::ComponentTypeMetadata;
    use bevy::ecs::component::ComponentId;
    use bevy::platform::collections::HashMap;
    use serde::ser::SerializeMap;
    use serde::{Deserialize, Deserializer, Serializer};

    /// Serializes a `HashMap<ComponentId, ComponentTypeMetadata>` using `usize` as keys.
    pub fn serialize<S>(
        map: &HashMap<ComponentId, ComponentTypeMetadata>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_map(Some(map.len()))?;
        for (key, value) in map {
            seq.serialize_entry(&key.index(), value)?;
        }
        seq.end()
    }

    /// Deserializes a map of indexes into `HashMap<ComponentId, ComponentTypeMetadata>`.
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<HashMap<ComponentId, ComponentTypeMetadata>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let index_to_metadata: HashMap<usize, ComponentTypeMetadata> =
            HashMap::deserialize(deserializer)?;
        let component_id_to_metadata = index_to_metadata
            .into_iter()
            .map(|(key, value)| (ComponentId::new(key), value))
            .collect();
        Ok(component_id_to_metadata)
    }
}

pub mod option_vec_debug_name {
    use bevy::utils::prelude::DebugName;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &Option<Vec<DebugName>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(debug_names) => {
                let strings: Vec<String> = debug_names
                    .iter()
                    .map(|debug_name| debug_name.to_string())
                    .collect();
                serializer.serialize_some(&strings)
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<DebugName>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt_strings: Option<Vec<String>> = Option::deserialize(deserializer)?;
        Ok(opt_strings.map(|strings| strings.into_iter().map(DebugName::from).collect()))
    }
}

/// Serializes [`SpawnDetails`].
pub fn serialize_spawn_details<S>(
    spawn_details: &SpawnDetails,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    format!("{spawn_details:?}").serialize(serializer)
}

/// Serializes [`EntityNotSpawnedError`].
pub fn serialize_entity_not_spawned_error<S>(
    error: &EntityNotSpawnedError,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    format!("{error:?}").serialize(serializer)
}

/// Serializes [`QueryEntityError`].
pub fn serialize_query_entity_error<S>(
    error: &QueryEntityError,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    format!("{error:?}").serialize(serializer)
}
