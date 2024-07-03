use std::{collections::HashMap, fs::File, path::Path};

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    components::{
        AreaOfEffect, BlocksTile, CombatStats, Confused, Confusion, Consumable, DefenseBonus,
        Equippable, Equipped, InBackpack, InflictsDamage, Item, MeleePowerBonus, Monster, Name,
        Player, Position, ProvidesHealing, Ranged, Renderable, Viewshed,
    },
    map::{Map, MAPCOUNT},
};

#[derive(Clone, Serialize, Deserialize)]
struct EntityRecord {
    id: Entity,
    area_of_effect: Option<AreaOfEffect>,
    blocks_tile: Option<BlocksTile>,
    combat_stats: Option<CombatStats>,
    confused: Option<Confused>,
    confusion: Option<Confusion>,
    consumable: Option<Consumable>,
    defense_bonus: Option<DefenseBonus>,
    equippable: Option<Equippable>,
    equipped: Option<Equipped>,
    in_backpack: Option<InBackpack>,
    inflicts_damage: Option<InflictsDamage>,
    item: Option<Item>,
    melee_power_bonus: Option<MeleePowerBonus>,
    monster: Option<Monster>,
    name: Option<Name>,
    player: Option<Player>,
    position: Option<Position>,
    provides_healing: Option<ProvidesHealing>,
    ranged: Option<Ranged>,
    renderable: Option<Renderable>,
    viewshed: Option<Viewshed>,
}

#[derive(Serialize, Deserialize)]
pub struct SavedGame {
    map: Map,
    entities: Vec<EntityRecord>,
}

const SAVE_FILE_NAME: &str = "./savegame.json";

pub fn save_game(world: &mut World) -> Result<(), serde_json::Error> {
    let map = world.resource::<Map>();

    let entities = world
        .iter_entities()
        .map(|e| EntityRecord {
            id: e.id(),
            area_of_effect: e.get::<AreaOfEffect>().cloned(),
            blocks_tile: e.get::<BlocksTile>().cloned(),
            combat_stats: e.get::<CombatStats>().cloned(),
            confused: e.get::<Confused>().cloned(),
            confusion: e.get::<Confusion>().cloned(),
            consumable: e.get::<Consumable>().cloned(),
            defense_bonus: e.get::<DefenseBonus>().cloned(),
            equippable: e.get::<Equippable>().cloned(),
            equipped: e.get::<Equipped>().cloned(),
            in_backpack: e.get::<InBackpack>().cloned(),
            inflicts_damage: e.get::<InflictsDamage>().cloned(),
            item: e.get::<Item>().cloned(),
            melee_power_bonus: e.get::<MeleePowerBonus>().cloned(),
            monster: e.get::<Monster>().cloned(),
            name: e.get::<Name>().cloned(),
            player: e.get::<Player>().cloned(),
            position: e.get::<Position>().cloned(),
            provides_healing: e.get::<ProvidesHealing>().cloned(),
            ranged: e.get::<Ranged>().cloned(),
            renderable: e.get::<Renderable>().cloned(),
            viewshed: e.get::<Viewshed>().cloned(),
        })
        .collect();

    let save = SavedGame {
        map: map.clone(),
        entities,
    };

    let writer = File::create(SAVE_FILE_NAME).unwrap();
    serde_json::to_writer(writer, &save)
}

pub fn does_save_exist() -> bool {
    Path::new(SAVE_FILE_NAME).exists()
}

pub fn load_game(world: &mut World) {
    let reader = File::open(SAVE_FILE_NAME).unwrap();
    let save: SavedGame = serde_json::from_reader(reader).unwrap();

    let mut id_transfer: HashMap<Entity, Entity> = HashMap::new();

    let mut map = save.map.clone();
    map.tile_content = vec![Vec::new(); MAPCOUNT];

    world.insert_resource(map);
    world.clear_entities();
    for entity in save.entities {
        let mut e = world.spawn_empty();
        id_transfer.insert(entity.id, e.id());

        if let Some(c) = entity.area_of_effect {
            e.insert(c);
        }
        if let Some(c) = entity.blocks_tile {
            e.insert(c);
        }
        if let Some(c) = entity.combat_stats {
            e.insert(c);
        }
        if let Some(c) = entity.confused {
            e.insert(c);
        }
        if let Some(c) = entity.confusion {
            e.insert(c);
        }
        if let Some(c) = entity.consumable {
            e.insert(c);
        }
        if let Some(c) = entity.defense_bonus {
            e.insert(c);
        }
        if let Some(c) = entity.equippable {
            e.insert(c);
        }
        if let Some(c) = entity.equipped {
            e.insert(c);
        }
        if let Some(c) = entity.in_backpack {
            e.insert(c);
        }
        if let Some(c) = entity.inflicts_damage {
            e.insert(c);
        }
        if let Some(c) = entity.item {
            e.insert(c);
        }
        if let Some(c) = entity.melee_power_bonus {
            e.insert(c);
        }
        if let Some(c) = entity.monster {
            e.insert(c);
        }
        if let Some(c) = entity.name {
            e.insert(c);
        }
        if let Some(c) = entity.player {
            e.insert(c);
        }
        if let Some(c) = entity.position {
            e.insert(c);
        }
        if let Some(c) = entity.provides_healing {
            e.insert(c);
        }
        if let Some(c) = entity.ranged {
            e.insert(c);
        }
        if let Some(c) = entity.renderable {
            e.insert(c);
        }
        if let Some(c) = entity.viewshed {
            e.insert(c);
        }
    }

    // refresh IDs for components that refer to other entities
    for backpack in world.query::<Option<&mut InBackpack>>().iter_mut(world) {
        if let Some(mut backpack) = backpack {
            backpack.owner = *id_transfer.get(&backpack.owner).unwrap();
        }
    }
    for equipped in world.query::<Option<&mut Equipped>>().iter_mut(world) {
        if let Some(mut equipped) = equipped {
            equipped.owner = *id_transfer.get(&equipped.owner).unwrap();
        }
    }
}

pub fn delete_save() {
    if Path::new(SAVE_FILE_NAME).exists() {
        std::fs::remove_file(SAVE_FILE_NAME).expect("Unable to delete file");
    }
}
