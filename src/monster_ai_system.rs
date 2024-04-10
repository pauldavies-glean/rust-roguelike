use bevy_ecs::prelude::*;
use rltk::console;

use crate::{AsPoint, GameTime, Monster, Name, Player, Position, Viewshed};

pub fn monster_ai_system(
    mut monsters: Query<(&Viewshed, &Name), With<Monster>>,
    players: Query<&Position, With<Player>>,
    go: NonSend<GameTime>,
    mut my: Local<GameTime>,
) {
    if go.time <= my.time {
        return;
    }
    my.time = go.time;

    let player_pos = players.single().as_point();

    for (viewshed, name) in monsters.iter_mut() {
        if viewshed.visible_tiles.contains(&player_pos) {
            console::log(&format!("{} shouts insults", name.name));
        }
    }
}
