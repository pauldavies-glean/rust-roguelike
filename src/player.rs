use bevy_ecs::prelude::*;
use rltk::VirtualKeyCode;
use std::cmp::{max, min};

use crate::{GameTime, Key, Map, Player, Position, TileType, Viewshed};

fn try_move_player(
    pos: &mut Position,
    viewshed: &mut Viewshed,
    map: &Map,
    delta_x: i32,
    delta_y: i32,
) -> bool {
    let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);
    if map.tiles[destination_idx] != TileType::Wall {
        pos.x = min(79, max(0, pos.x + delta_x));
        pos.y = min(49, max(0, pos.y + delta_y));

        viewshed.dirty = true;
        return true;
    }

    false
}

fn player_input(pos: &mut Position, viewshed: &mut Viewshed, map: &Map, key: Key) -> bool {
    match key {
        None => {}
        Some(key) => match key {
            VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::H => {
                return try_move_player(pos, viewshed, &map, -1, 0);
            }

            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::L => {
                return try_move_player(pos, viewshed, &map, 1, 0);
            }

            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
                return try_move_player(pos, viewshed, &map, 0, -1);
            }

            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                return try_move_player(pos, viewshed, &map, 0, 1);
            }

            _ => {}
        },
    }

    false
}

pub fn player_input_system(
    mut players: Query<(&mut Position, &mut Viewshed), With<Player>>,
    map: NonSend<Map>,
    key: NonSend<Key>,
    mut go: NonSendMut<GameTime>,
) {
    let (mut pos, mut viewshed) = players.single_mut();
    if player_input(&mut pos, &mut viewshed, &map, *key) {
        go.time += 1;
    }
}
