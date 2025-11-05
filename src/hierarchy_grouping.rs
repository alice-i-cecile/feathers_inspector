use bevy::prelude::*;

use crate::entity_grouping::EntityGrouping;

pub(crate) fn group(
    _world: &World,
    _entities: impl ExactSizeIterator<Item = Entity>,
) -> EntityGrouping {
    todo!()
}
