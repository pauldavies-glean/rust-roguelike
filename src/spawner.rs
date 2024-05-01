use bevy_ecs::prelude::*;
use rltk::{to_cp437, FontCharType, RandomNumberGenerator, BLACK, MAGENTA, RED, YELLOW};

use crate::{
    components::{
        BlocksTile, CombatStats, Item, Monster, Name, Player, Position, Potion, Renderable,
        Viewshed,
    },
    map::MAPWIDTH,
    rect::Rect,
};

const MAX_MONSTERS: i32 = 4;
const MAX_ITEMS: i32 = 2;

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

/// Spawns a random monster at a given location
pub fn random_monster(world: &mut World, x: i32, y: i32) {
    let roll: i32;
    {
        let mut rng = world.non_send_resource_mut::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 2);
    }
    match roll {
        1 => orc(world, x, y),
        _ => goblin(world, x, y),
    }
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

/// Fills a room with stuff!
pub fn spawn_room(world: &mut World, room: &Rect) {
    let mut monster_spawn_points: Vec<usize> = Vec::new();
    let mut item_spawn_points: Vec<usize> = Vec::new();

    // Scope to keep the borrow checker happy
    {
        let mut rng = world.non_send_resource_mut::<RandomNumberGenerator>();
        let num_monsters = rng.roll_dice(1, MAX_MONSTERS + 2) - 3;
        let num_items = rng.roll_dice(1, MAX_ITEMS + 2) - 3;

        for _i in 0..num_monsters {
            let mut added = false;
            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAPWIDTH) + x;
                if !monster_spawn_points.contains(&idx) {
                    monster_spawn_points.push(idx);
                    added = true;
                }
            }
        }

        for _i in 0..num_items {
            let mut added = false;
            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAPWIDTH) + x;
                if !item_spawn_points.contains(&idx) {
                    item_spawn_points.push(idx);
                    added = true;
                }
            }
        }
    }

    // Actually spawn the monsters
    for idx in monster_spawn_points.iter() {
        let x = *idx % MAPWIDTH;
        let y = *idx / MAPWIDTH;
        random_monster(world, x as i32, y as i32);
    }

    // Actually spawn the potions
    for idx in item_spawn_points.iter() {
        let x = *idx % MAPWIDTH;
        let y = *idx / MAPWIDTH;
        health_potion(world, x as i32, y as i32);
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
        Potion { heal_amount: 8 },
    ));
}
