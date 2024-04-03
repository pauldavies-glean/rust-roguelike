use bevy_ecs::prelude::*;
use rltk::VirtualKeyCode;
use std::cmp::{max, min};

use crate::{Key, Map, Player, Position, TileType, Viewshed};

pub fn try_move_player(
    pos: &mut Position,
    viewshed: &mut Viewshed,
    map: &Map,
    delta_x: i32,
    delta_y: i32,
) {
    let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);
    if map.tiles[destination_idx] != TileType::Wall {
        pos.x = min(79, max(0, pos.x + delta_x));
        pos.y = min(49, max(0, pos.y + delta_y));

        viewshed.dirty = true;
    }
}

pub fn player_input(
    mut query: Query<(&mut Position, &mut Viewshed), With<Player>>,
    map: NonSend<Map>,
    key: NonSend<Key>,
) {
    let binding = query.single_mut();
    let (mut pos, mut viewshed) = binding;
    match key.to_owned() {
        None => {}
        Some(key) => match key {
            VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::H => {
                try_move_player(&mut pos, &mut viewshed, &map, -1, 0)
            }

            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::L => {
                try_move_player(&mut pos, &mut viewshed, &map, 1, 0)
            }

            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
                try_move_player(&mut pos, &mut viewshed, &map, 0, -1)
            }

            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                try_move_player(&mut pos, &mut viewshed, &map, 0, 1)
            }

            _ => {}
        },
    }
}
