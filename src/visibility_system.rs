use crate::{AsPoint, Map, Player, Position, Viewshed};
use bevy_ecs::prelude::*;
use rltk::field_of_view;

pub fn visibility_system(
    mut viewers: Query<(&Position, &mut Viewshed, Option<&Player>)>,
    mut map: ResMut<Map>,
) {
    for (pos, mut viewshed, player) in viewers.iter_mut() {
        if !viewshed.dirty {
            continue;
        };

        viewshed.dirty = false;
        viewshed.visible_tiles.clear();
        viewshed.visible_tiles = field_of_view(pos.as_point(), viewshed.range, &*map);
        viewshed.visible_tiles.retain(|p| map.contains_point(*p));

        // Reveal player sight
        if player.is_some() {
            for t in map.visible_tiles.iter_mut() {
                *t = false;
            }

            for vis in viewshed.visible_tiles.iter() {
                let idx = map.xy_idx(vis.x, vis.y);
                map.revealed_tiles[idx] = true;
                map.visible_tiles[idx] = true;
            }
        }
    }
}
