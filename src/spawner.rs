use std::collections::HashMap;

use bevy_ecs::prelude::*;
use rltk::{
    to_cp437, FontCharType, RandomNumberGenerator, BLACK, CYAN, CYAN3, GREEN, MAGENTA, ORANGE,
    PINK, RED, YELLOW,
};

use crate::{
    components::{
        AreaOfEffect, BlocksTile, CombatStats, Confusion, Consumable, DefenseBonus, EntryTrigger,
        EquipmentSlot, Equippable, Hidden, HungerClock, HungerState, InflictsDamage, Item,
        MagicMapper, MeleePowerBonus, Monster, Name, Player, Position, ProvidesFood,
        ProvidesHealing, Ranged, Renderable, SingleActivation, Viewshed,
    },
    map::{Map, TileType, MAPWIDTH},
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
            render_order: 1,
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
        HungerClock {
            state: HungerState::WellFed,
            duration: 20,
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
            render_order: 5,
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
        .add("Dagger", 3)
        .add("Shield", 3)
        .add("Longsword", map_depth - 1)
        .add("Tower Shield", map_depth - 1)
        .add("Rations", 10)
        .add("Magic Mapping Scroll", 2)
        .add("Bear Trap", 2)
}

/// Fills a room with stuff!
pub fn spawn_room(
    map: &Map,
    rng: &mut RandomNumberGenerator,
    room: &Rect,
    map_depth: i32,
    spawn_list: &mut Vec<(usize, String)>,
) {
    let mut possible_targets: Vec<usize> = Vec::new();
    {
        for y in room.y1 + 1..room.y2 {
            for x in room.x1 + 1..room.x2 {
                let idx = map.xy_idx(x, y);
                if map.tiles[idx] == TileType::Floor {
                    possible_targets.push(idx);
                }
            }
        }
    }

    spawn_region(rng, &possible_targets, map_depth, spawn_list);
}

fn health_potion(world: &mut World, x: i32, y: i32) {
    world.spawn((
        Position { x, y },
        Renderable {
            glyph: to_cp437('¡'),
            fg: MAGENTA,
            bg: BLACK,
            render_order: 10,
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
            render_order: 10,
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
            render_order: 10,
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
            render_order: 10,
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

fn dagger(world: &mut World, x: i32, y: i32) {
    world.spawn((
        Position { x, y },
        Renderable {
            glyph: to_cp437('/'),
            fg: CYAN,
            bg: BLACK,
            render_order: 10,
        },
        Name {
            name: "Dagger".to_string(),
        },
        Item {},
        Equippable {
            slot: EquipmentSlot::Melee,
        },
        MeleePowerBonus { power: 2 },
    ));
}

fn longsword(world: &mut World, x: i32, y: i32) {
    world.spawn((
        Position { x, y },
        Renderable {
            glyph: to_cp437('/'),
            fg: YELLOW,
            bg: BLACK,
            render_order: 10,
        },
        Name {
            name: "Longsword".to_string(),
        },
        Item {},
        Equippable {
            slot: EquipmentSlot::Melee,
        },
        MeleePowerBonus { power: 4 },
    ));
}

fn shield(world: &mut World, x: i32, y: i32) {
    world.spawn((
        Position { x, y },
        Renderable {
            glyph: to_cp437('('),
            fg: CYAN,
            bg: BLACK,
            render_order: 10,
        },
        Name {
            name: "Shield".to_string(),
        },
        Item {},
        Equippable {
            slot: EquipmentSlot::Shield,
        },
        DefenseBonus { defense: 1 },
    ));
}

fn tower_shield(world: &mut World, x: i32, y: i32) {
    world.spawn((
        Position { x, y },
        Renderable {
            glyph: to_cp437('('),
            fg: YELLOW,
            bg: BLACK,
            render_order: 10,
        },
        Name {
            name: "Shield".to_string(),
        },
        Item {},
        Equippable {
            slot: EquipmentSlot::Shield,
        },
        DefenseBonus { defense: 3 },
    ));
}

fn rations(world: &mut World, x: i32, y: i32) {
    world.spawn((
        Position { x, y },
        Renderable {
            glyph: to_cp437('%'),
            fg: GREEN,
            bg: BLACK,
            render_order: 10,
        },
        Name {
            name: "Rations".to_string(),
        },
        Item {},
        ProvidesFood {},
        Consumable {},
    ));
}

fn magic_mapping_scroll(world: &mut World, x: i32, y: i32) {
    world.spawn((
        Position { x, y },
        Renderable {
            glyph: to_cp437(')'),
            fg: CYAN3,
            bg: BLACK,
            render_order: 10,
        },
        Name {
            name: "Scroll of Magic Mapping".to_string(),
        },
        Item {},
        MagicMapper {},
        Consumable {},
    ));
}

fn bear_trap(world: &mut World, x: i32, y: i32) {
    world.spawn((
        Position { x, y },
        Renderable {
            glyph: to_cp437('^'),
            fg: RED,
            bg: BLACK,
            render_order: 9,
        },
        Name {
            name: "Bear Trap".to_string(),
        },
        Hidden {},
        EntryTrigger {},
        SingleActivation {},
        InflictsDamage { damage: 6 },
    ));
}

/// Spawns a named entity (name in tuple.1) at the location in (tuple.0)
pub fn spawn_entity(ecs: &mut World, (spawn_idx, spawn_name): &(&usize, &String)) {
    let x = (*spawn_idx % MAPWIDTH) as i32;
    let y = (*spawn_idx / MAPWIDTH) as i32;

    match spawn_name.as_ref() {
        "Goblin" => goblin(ecs, x, y),
        "Orc" => orc(ecs, x, y),
        "Health Potion" => health_potion(ecs, x, y),
        "Fireball Scroll" => fireball_scroll(ecs, x, y),
        "Confusion Scroll" => confusion_scroll(ecs, x, y),
        "Magic Missile Scroll" => magic_missile_scroll(ecs, x, y),
        "Dagger" => dagger(ecs, x, y),
        "Shield" => shield(ecs, x, y),
        "Longsword" => longsword(ecs, x, y),
        "Tower Shield" => tower_shield(ecs, x, y),
        "Rations" => rations(ecs, x, y),
        "Magic Mapping Scroll" => magic_mapping_scroll(ecs, x, y),
        "Bear Trap" => bear_trap(ecs, x, y),
        _ => {}
    }
}

/// Fills a region with stuff!
pub fn spawn_region(
    rng: &mut RandomNumberGenerator,
    area: &[usize],
    map_depth: i32,
    spawn_list: &mut Vec<(usize, String)>,
) {
    let spawn_table = room_table(map_depth);
    let mut spawn_points: HashMap<usize, String> = HashMap::new();
    let mut areas: Vec<usize> = Vec::from(area);

    // Scope to keep the borrow checker happy
    {
        let num_spawns = i32::min(
            areas.len() as i32,
            rng.roll_dice(1, MAX_SPAWNS + 3) + (map_depth - 1) - 3,
        );
        if num_spawns == 0 {
            return;
        }

        for _i in 0..num_spawns {
            let array_index = if areas.len() == 1 {
                0usize
            } else {
                (rng.roll_dice(1, areas.len() as i32) - 1) as usize
            };

            let map_idx = areas[array_index];
            spawn_points.insert(map_idx, spawn_table.roll(rng));
            areas.remove(array_index);
        }
    }

    // Actually spawn the monsters
    for (spawn_idx, spawn_name) in spawn_points.iter() {
        spawn_list.push((*spawn_idx, spawn_name.to_string()));
    }
}
