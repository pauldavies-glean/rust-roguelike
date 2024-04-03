use crate::{xy_idx, Key, Map, Player, Position, TileType};
use bevy_ecs::prelude::*;
use rltk::VirtualKeyCode;
use std::cmp::{max, min};

pub fn try_move_player(pos: &mut Position, map: &Map, delta_x: i32, delta_y: i32) {
    let destination_idx = xy_idx(pos.x + delta_x, pos.y + delta_y);
    if map[destination_idx] != TileType::Wall {
        pos.x = min(79, max(0, pos.x + delta_x));
        pos.y = min(49, max(0, pos.y + delta_y));
    }
}

pub fn player_input(
    mut query: Query<&mut Position, With<Player>>,
    map: NonSend<Map>,
    key: NonSend<Key>,
) {
    let mut binding = query.single_mut();
    let pos = binding.as_mut();
    match key.to_owned() {
        None => {}
        Some(key) => match key {
            VirtualKeyCode::Left => try_move_player(pos, &map, -1, 0),
            VirtualKeyCode::Right => try_move_player(pos, &map, 1, 0),
            VirtualKeyCode::Up => try_move_player(pos, &map, 0, -1),
            VirtualKeyCode::Down => try_move_player(pos, &map, 0, 1),
            _ => {}
        },
    }
}
