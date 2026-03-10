//! Cache management for the inspector UI.

use crate::{
    gui::{config::InspectorConfig, plugin::RefreshCache, state::InspectorState},
    inspection::component_inspection::ComponentMetadataMap,
};
use bevy::prelude::*;

pub mod snapshot;
pub(crate) mod systems;

pub use snapshot::WorldSnapshot;
pub use systems::update_inspector_cache;

/// Cached data for the inspector.
///
/// This is regularly invalidated, but this resource helps avoid repeated allocations.
#[derive(Resource)]
pub struct InspectorCache {
    /// Cached object list after filtering.
    pub filtered_objects: Vec<crate::gui::state::ObjectListEntry>,
    /// Cached metadata map (reused across inspections).
    pub metadata_map: Option<ComponentMetadataMap>,
    /// Snapshot of the world state.
    pub snapshot: WorldSnapshot,
    /// Tracks whether the cache should be refreshed.
    pub timer: Option<Timer>,
}

impl FromWorld for InspectorCache {
    fn from_world(world: &mut World) -> Self {
        let timer = world
            .resource::<InspectorConfig>()
            .refresh_interval
            .map(|duration| Timer::new(duration, TimerMode::Repeating));

        Self {
            filtered_objects: Vec::default(),
            metadata_map: None,
            snapshot: WorldSnapshot::default(),
            timer,
        }
    }
}

/// A system which periodically sends a [`RefreshCache`] message.
///
/// The frequency is controlled by the [`refresh_interval`] field in [`InspectorConfig`].
///
/// [`refresh_interval`]: InspectorConfig::refresh_interval
pub fn periodically_refresh_cache(
    mut message_writer: MessageWriter<RefreshCache>,
    time: Res<Time>,
    mut is_timer_ticking: Local<bool>,
    state: Res<InspectorState>,
    mut cache: ResMut<InspectorCache>,
) {
    let Some(ref mut timer) = cache.timer else {
        return;
    };
    if state.is_paused {
        *is_timer_ticking = false;
        return;
    }

    if !*is_timer_ticking {
        *is_timer_ticking = true;
        timer.reset();
    }

    timer.tick(time.delta());
    if timer.just_finished() {
        message_writer.write(RefreshCache);
    }
}
