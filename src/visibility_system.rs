use crate::{Map, Player, Position, Viewshed};
use bevy_ecs::prelude::*;
use rltk::{field_of_view, Point};

pub fn visibility_system(
    mut query: Query<(&Position, &mut Viewshed, Option<&Player>)>,
    mut map: NonSendMut<Map>,
) {
    for (pos, mut viewshed, player) in query.iter_mut() {
        if !viewshed.dirty {
            continue;
        };

        viewshed.dirty = false;
        viewshed.visible_tiles.clear();
        viewshed.visible_tiles = field_of_view(Point::new(pos.x, pos.y), viewshed.range, &*map);
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
