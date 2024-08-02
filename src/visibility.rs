use bevy_ecs::prelude::*;
use rltk::{field_of_view, RandomNumberGenerator};

use crate::{
    components::{AsPoint, Hidden, Name, Player, Position, Viewshed},
    gamelog::GameLog,
    map::Map,
};

pub fn visibility_system(
    mut commands: Commands,
    mut viewers: Query<(&Position, &mut Viewshed, Option<&Player>)>,
    hidden: Query<&Name, With<Hidden>>,
    mut map: ResMut<Map>,
    mut log: ResMut<GameLog>,
    mut rng: NonSendMut<RandomNumberGenerator>,
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

                for e in map.tile_content[idx].iter() {
                    if let Ok(name) = hidden.get(*e) {
                        if rng.roll_dice(1, 24) == 1 {
                            log.entries.push(format!("You spotted a {}.", &name.name));
                            commands.entity(*e).remove::<Hidden>();
                        }
                    }
                }
            }
        }
    }
}
