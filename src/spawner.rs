use std::collections::HashMap;

use bevy_ecs::prelude::*;
use rltk::{
    to_cp437, FontCharType, RandomNumberGenerator, BLACK, CYAN, MAGENTA, ORANGE, PINK, RED, YELLOW,
};

use crate::{
    components::{
        AreaOfEffect, BlocksTile, CombatStats, Confusion, Consumable, InflictsDamage, Item,
        Monster, Name, Player, Position, ProvidesHealing, Ranged, Renderable, Viewshed,
    },
    map::MAPWIDTH,
    random_table::RandomTable,
    rect::Rect,
};

const MAX_SPAWNS: i32 = 4;

/// Spawns the player and returns his/her entity object.
pub fn player(world: &mut World, player_x: i32, player_y: i32) {
    world.spawn((
        Position {
            x: player_x,
            y: player_y,
        },
        Renderable {
            glyph: to_cp437('@'),
            fg: YELLOW,
            bg: BLACK,
            render_order: 0,
        },
        Player {},
        Name {
            name: "Player".to_string(),
        },
        Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        },
        CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        },
    ));
}

fn orc(world: &mut World, x: i32, y: i32) {
    monster(world, x, y, to_cp437('o'), "Orc");
}
fn goblin(world: &mut World, x: i32, y: i32) {
    monster(world, x, y, to_cp437('g'), "Goblin");
}

fn monster<S: ToString>(world: &mut World, x: i32, y: i32, glyph: FontCharType, name: S) {
    world.spawn((
        Position { x, y },
        Renderable {
            glyph,
            fg: RED,
            bg: BLACK,
            render_order: 1,
        },
        Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        },
        Monster {},
        Name {
            name: name.to_string(),
        },
        BlocksTile {},
        CombatStats {
            max_hp: 16,
            hp: 16,
            defense: 1,
            power: 4,
        },
    ));
}

fn room_table(map_depth: i32) -> RandomTable {
    RandomTable::new()
        .add("Goblin", 10)
        .add("Orc", 1 + map_depth)
        .add("Health Potion", 7)
        .add("Fireball Scroll", 2 + map_depth)
        .add("Confusion Scroll", 2 + map_depth)
        .add("Magic Missile Scroll", 4)
}

/// Fills a room with stuff!
pub fn spawn_room(world: &mut World, room: &Rect, map_depth: i32) {
    let spawn_table = room_table(map_depth);
    let mut spawn_points: HashMap<usize, String> = HashMap::new();

    // Scope to keep the borrow checker happy
    {
        let mut rng = world.non_send_resource_mut::<RandomNumberGenerator>();
        let num_spawns = rng.roll_dice(1, MAX_SPAWNS + 3) + (map_depth - 1) - 3;

        for _i in 0..num_spawns {
            let mut added = false;
            let mut tries = 0;
            while !added && tries < 20 {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAPWIDTH) + x;
                if !spawn_points.contains_key(&idx) {
                    spawn_points.insert(idx, spawn_table.roll(&mut rng));
                    added = true;
                } else {
                    tries += 1;
                }
            }
        }
    }

    // Actually spawn the monsters
    for (idx, name) in spawn_points.iter() {
        let x = (*idx % MAPWIDTH) as i32;
        let y = (*idx / MAPWIDTH) as i32;

        match name.as_ref() {
            "Goblin" => goblin(world, x, y),
            "Orc" => orc(world, x, y),
            "Health Potion" => health_potion(world, x, y),
            "Fireball Scroll" => fireball_scroll(world, x, y),
            "Confusion Scroll" => confusion_scroll(world, x, y),
            "Magic Missile Scroll" => magic_missile_scroll(world, x, y),
            _ => {}
        }
    }
}

fn health_potion(world: &mut World, x: i32, y: i32) {
    world.spawn((
        Position { x, y },
        Renderable {
            glyph: to_cp437('¡'),
            fg: MAGENTA,
            bg: BLACK,
            render_order: 2,
        },
        Name {
            name: "Health Potion".to_string(),
        },
        Item {},
        Consumable {},
        ProvidesHealing { heal_amount: 8 },
    ));
}

fn magic_missile_scroll(world: &mut World, x: i32, y: i32) {
    world.spawn((
        Position { x, y },
        Renderable {
            glyph: to_cp437(')'),
            fg: CYAN,
            bg: BLACK,
            render_order: 2,
        },
        Name {
            name: "Magic Missile Scroll".to_string(),
        },
        Item {},
        Consumable {},
        Ranged { range: 6 },
        InflictsDamage { damage: 8 },
    ));
}

fn fireball_scroll(world: &mut World, x: i32, y: i32) {
    world.spawn((
        Position { x, y },
        Renderable {
            glyph: to_cp437(')'),
            fg: ORANGE,
            bg: BLACK,
            render_order: 2,
        },
        Name {
            name: "Fireball Scroll".to_string(),
        },
        Item {},
        Consumable {},
        Ranged { range: 6 },
        InflictsDamage { damage: 20 },
        AreaOfEffect { radius: 3 },
    ));
}

fn confusion_scroll(world: &mut World, x: i32, y: i32) {
    world.spawn((
        Position { x, y },
        Renderable {
            glyph: to_cp437(')'),
            fg: PINK,
            bg: BLACK,
            render_order: 2,
        },
        Name {
            name: "Confusion Scroll".to_string(),
        },
        Item {},
        Consumable {},
        Ranged { range: 6 },
        Confusion { turns: 4 },
    ));
}
