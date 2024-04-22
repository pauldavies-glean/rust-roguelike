use bevy_ecs::prelude::*;

use crate::{BlocksTile, Map, Position};

pub fn map_indexing_system(
    entities: Query<(Entity, &Position, Option<&BlocksTile>)>,
    mut map: ResMut<Map>,
) {
    map.populate_blocked();
    map.clear_content_index();
    for (entity, position, blocks) in entities.iter() {
        let idx = map.xy_idx(position.x, position.y);

        if blocks.is_some() {
            map.blocked[idx] = true;
        }

        map.tile_content[idx].push(entity);
    }
}
