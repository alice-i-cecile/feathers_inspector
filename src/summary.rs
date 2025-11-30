//! Statistical summaries about the [`World`].

use std::cmp::Reverse;

use bevy::{
    ecs::{archetype::ArchetypeId, component::ComponentId},
    prelude::*,
};

/// Settings for [`WorldSummary`].
#[derive(Clone, Debug, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SummarySettings {
    /// Whether to use component names
    /// for formatting archetype signatures.
    pub include_component_names: bool,
    /// Whether to include archetypes with no entities.
    pub include_empty_archetypes: bool,
    /// Optional output limit for archetype listing.
    pub max_archetype_rows: Option<usize>,
}

impl Default for SummarySettings {
    fn default() -> Self {
        const DEFAULT_ARCHETYPE_ROWS: usize = 15;
        Self {
            include_component_names: true,
            include_empty_archetypes: false,
            max_archetype_rows: Some(DEFAULT_ARCHETYPE_ROWS),
        }
    }
}

/// Per-archetype data in an inspection summary.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ArchetypeSummary {
    /// The id of this archetype.
    #[cfg_attr(
        feature = "serde",
        serde(with = "crate::serde_conversions::archetype_id")
    )]
    pub archetype_id: ArchetypeId,
    /// How many entities are in this archetype.
    pub entity_count: usize,
    /// What components define this archetype.
    #[cfg_attr(
        feature = "serde",
        serde(with = "crate::serde_conversions::slice_component_id")
    )]
    pub component_ids: Vec<ComponentId>,
    /// The names of the components defining this archetype.
    ///
    /// Optional value determined by [`SummarySettings::include_component_names`].
    #[cfg_attr(
        feature = "serde",
        serde(with = "crate::serde_conversions::option_vec_debug_name")
    )]
    pub component_names: Option<Vec<DebugName>>,
}

impl ArchetypeSummary {
    /// Returns a human-readable archetype signature for display.
    pub fn signature_short(&self) -> String {
        match &self.component_names {
            Some(names) => {
                let mut components: Vec<String> = names
                    .iter()
                    .map(|name| name.shortname().to_string())
                    .collect();
                components.sort();
                let components_joined = components.join(", ");
                let archetype_id = self.archetype_id.index();
                format!("{components_joined} (#{archetype_id})")
            }
            None => format!("#{}", self.archetype_id.index()),
        }
    }
}

impl std::fmt::Display for ArchetypeSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut display = self.signature_short();
        let entity_count = self.entity_count;
        display.push_str(&format!(" ({entity_count} entities)"));
        display.push('\n');
        write!(f, "{display}")
    }
}

/// [`World`] data summary result.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WorldSummary {
    /// The number of entities.
    pub total_entities: u32,
    /// The number of all archetypes, empty or not.
    pub total_archetypes: usize,
    /// The number of empty archetypes.
    pub empty_archetypes: usize,
    /// The number of [`Send`] resources.
    pub total_send_resources: usize,
    /// The number of non-[`Send`] resources.
    pub total_non_send_resources: usize,
    /// Information about archetypes.
    pub archetype_summaries: Vec<ArchetypeSummary>,
    /// Limit of displayed archetypes.
    max_archetype_rows: Option<usize>,
}

impl std::fmt::Display for WorldSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut display = String::new();
        display.push_str("Summary:");
        display.push('\n');
        let entity_count = self.total_entities;
        display.push_str(&format!("Entity count: {entity_count}"));
        display.push('\n');
        let archetype_count = self.total_archetypes;
        let empty_archetype_count = self.empty_archetypes;
        display.push_str(&format!(
            "Archetype count: {archetype_count} ({empty_archetype_count} empty)"
        ));
        display.push('\n');
        let send_resource_count = self.total_send_resources;
        let non_send_resource_count = self.total_non_send_resources;
        let total_resource_count = send_resource_count + non_send_resource_count;
        display.push_str(&format!("Resource count: {total_resource_count} ({send_resource_count} `Send` + {non_send_resource_count} non-`Send`)"));
        display.push('\n');
        display.push_str("Archetypes:");
        display.push('\n');
        let archetype_display_limit = self.max_archetype_rows.unwrap_or(usize::MAX);
        for (i, archetype_summary) in self
            .archetype_summaries
            .iter()
            .take(archetype_display_limit)
            .enumerate()
        {
            let position = i + 1;
            display.push_str(&format!("{position}. {archetype_summary}"));
        }
        if self.archetype_summaries.len() > archetype_display_limit {
            let remaining_archetypes = self.archetype_summaries.len() - archetype_display_limit;
            display.push_str(&format!("... and {remaining_archetypes} more archetypes."));
        }
        write!(f, "{display}")
    }
}

/// Adds summary methods to [`World`].
pub trait WorldSummaryExt {
    /// Summarizes data about this [`World`].
    fn summarize(&self, settings: SummarySettings) -> WorldSummary;
}

impl WorldSummaryExt for World {
    fn summarize(&self, settings: SummarySettings) -> WorldSummary {
        let total_entities = self.entities().len();
        let total_send_resources = self.storages().resources.len();
        let total_non_send_resources = self.storages().non_send_resources.len();
        let total_archetypes = self.archetypes().len();
        let mut archetype_summaries: Vec<ArchetypeSummary> = self
            .archetypes()
            .iter()
            .map(|archetype| ArchetypeSummary {
                archetype_id: archetype.id(),
                entity_count: archetype.entities().len(),
                component_ids: archetype.components().to_vec(),
                component_names: settings.include_component_names.then_some(
                    archetype
                        .components()
                        .iter()
                        .map(|component_id| {
                            self.components()
                                .get_name(*component_id)
                                .unwrap_or_else(|| {
                                    let component_index = component_id.index();
                                    DebugName::owned(format!("Component #{component_index}"))
                                })
                        })
                        .collect(),
                ),
            })
            .filter(|archetype_summary| {
                settings.include_empty_archetypes || archetype_summary.entity_count > 0
            })
            .collect();
        archetype_summaries.sort_by_key(|archetype_summary| {
            (
                Reverse(archetype_summary.entity_count),
                archetype_summary.component_ids.len(),
                archetype_summary.archetype_id.index(),
            )
        });
        let empty_archetypes = if settings.include_empty_archetypes {
            archetype_summaries
                .iter()
                .filter(|archetype_summary| archetype_summary.entity_count == 0)
                .count()
        } else {
            self.archetypes().len() - archetype_summaries.len()
        };
        WorldSummary {
            total_entities,
            total_archetypes,
            empty_archetypes,
            total_send_resources,
            total_non_send_resources,
            archetype_summaries,
            max_archetype_rows: settings.max_archetype_rows,
        }
    }
}

/// Adds summary methods for [`Commands`].
pub trait CommandsSummaryExt {
    /// Summarizes data about the [`World`].
    fn summarize(&mut self, settings: SummarySettings);
}

impl CommandsSummaryExt for Commands<'_, '_> {
    fn summarize(&mut self, settings: SummarySettings) {
        self.queue(move |world: &mut World| {
            let inspection_summary = world.summarize(settings);
            info!("{inspection_summary}");
        });
    }
}
