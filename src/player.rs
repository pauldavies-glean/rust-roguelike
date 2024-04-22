use bevy_ecs::prelude::*;
use bevy_ecs::system::EntityCommands;
use lazy_static::lazy_static;
use rltk::VirtualKeyCode;
use std::cmp::{max, min};
use std::collections::HashMap;

use crate::components::{Player, WantsToMelee};
use crate::{
    components::{CombatStats, Position, Viewshed},
    map::Map,
};
use crate::{Key, RunState};

fn try_move_player(
    mut commands: EntityCommands,
    enemy_query: Query<&CombatStats>,
    pos: &mut Position,
    viewshed: &mut Viewshed,
    map: &Map,
    delta_x: i32,
    delta_y: i32,
) -> bool {
    let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

    for target in map.tile_content[destination_idx].iter() {
        if enemy_query.contains(*target) {
            commands.insert(WantsToMelee { target: *target });
            return true;
        }
    }

    if !map.blocked[destination_idx] {
        pos.x = min(79, max(0, pos.x + delta_x));
        pos.y = min(49, max(0, pos.y + delta_y));

        viewshed.dirty = true;
        return true;
    }

    false
}

struct Translation(i32, i32);
lazy_static! {
    static ref MOVEMENT_KEYS: HashMap<VirtualKeyCode, Translation> = {
        let mut m = HashMap::new();
        m.insert(VirtualKeyCode::Left, Translation(-1, 0));
        m.insert(VirtualKeyCode::Numpad4, Translation(-1, 0));
        m.insert(VirtualKeyCode::H, Translation(-1, 0));
        m.insert(VirtualKeyCode::Right, Translation(1, 0));
        m.insert(VirtualKeyCode::Numpad6, Translation(1, 0));
        m.insert(VirtualKeyCode::L, Translation(1, 0));
        m.insert(VirtualKeyCode::Up, Translation(0, -1));
        m.insert(VirtualKeyCode::Numpad8, Translation(0, -1));
        m.insert(VirtualKeyCode::K, Translation(0, -1));
        m.insert(VirtualKeyCode::Down, Translation(0, 1));
        m.insert(VirtualKeyCode::Numpad2, Translation(0, 1));
        m.insert(VirtualKeyCode::J, Translation(0, 1));
        m.insert(VirtualKeyCode::Numpad9, Translation(1, -1));
        m.insert(VirtualKeyCode::U, Translation(1, -1));
        m.insert(VirtualKeyCode::Numpad7, Translation(-1, -1));
        m.insert(VirtualKeyCode::Y, Translation(-1, -1));
        m.insert(VirtualKeyCode::Numpad3, Translation(1, 1));
        m.insert(VirtualKeyCode::N, Translation(1, 1));
        m.insert(VirtualKeyCode::Numpad1, Translation(-1, 1));
        m.insert(VirtualKeyCode::B, Translation(-1, 1));
        m
    };
}

pub fn player_input_system(
    mut commands: Commands,
    mut players: Query<(Entity, &mut Position, &mut Viewshed), With<Player>>,
    enemy_query: Query<&CombatStats>,
    map: Res<Map>,
    key: NonSend<Key>,
    mut state: ResMut<RunState>,
) {
    if *state != RunState::AwaitingInput {
        return;
    }

    if let Some(k) = *key {
        if let Some(delta) = MOVEMENT_KEYS.get(&k) {
            let (player, mut pos, mut viewshed) = players.single_mut();
            if try_move_player(
                commands.entity(player),
                enemy_query,
                &mut pos,
                &mut viewshed,
                &map,
                delta.0,
                delta.1,
            ) {
                *state = RunState::PlayerTurn;
            }
        }
    }
}
