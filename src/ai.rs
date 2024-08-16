use bevy_ecs::prelude::*;
use rltk::{a_star_search, to_cp437, DistanceAlg, Point, BLACK, MAGENTA};

use crate::{
    components::{
        AsPoint, Confused, EntityMoved, Monster, Player, Position, Viewshed, WantsToMelee,
    },
    map::Map,
    particle::ParticleBuilder,
    RunState,
};

pub fn monster_ai_system(
    mut commands: Commands,
    mut monsters: Query<
        (Entity, &mut Viewshed, &mut Position, Option<&mut Confused>),
        (With<Monster>, Without<Player>),
    >,
    players: Query<(Entity, &Position), With<Player>>,
    mut map: ResMut<Map>,
    state: Res<RunState>,
    mut particle: ResMut<ParticleBuilder>,
) {
    if *state != RunState::MonsterTurn {
        return;
    }

    let (player, player_pos) = players.single();
    let player_point = player_pos.as_point();

    for (monster, mut viewshed, mut pos, confused) in monsters.iter_mut() {
        let mut can_act = true;
        if let Some(mut confused) = confused {
            confused.turns -= 1;
            if confused.turns < 1 {
                // TODO warn player?
                commands.entity(monster).remove::<Confused>();
            }
            can_act = false;

            particle.request(pos.x, pos.y, MAGENTA, BLACK, to_cp437('?'), 200.0);
        }

        if can_act {
            let distance =
                DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), player_point);
            if distance < 1.5 {
                // Attack goes here
                commands
                    .entity(monster)
                    .insert(WantsToMelee { target: player });
            } else if viewshed.visible_tiles.contains(&player_point) {
                let start_idx = map.xy_idx(pos.x, pos.y);

                let path = a_star_search(start_idx, map.xy_idx(player_pos.x, player_pos.y), &*map);
                if path.success && path.steps.len() > 1 {
                    let next_idx = path.steps[1];
                    pos.x = next_idx as i32 % map.width;
                    pos.y = next_idx as i32 / map.width;
                    commands.entity(monster).insert(EntityMoved {});
                    viewshed.dirty = true;

                    map.blocked[start_idx] = false;
                    map.blocked[next_idx] = true;
                }
            }
        }
    }
}
