use bevy_ecs::prelude::*;
use bevy_ecs::system::EntityCommands;
use lazy_static::lazy_static;
use rltk::VirtualKeyCode;
use std::cmp::{max, min};
use std::collections::HashMap;

use crate::components::{Item, Monster, Player, Waiting, WantsToMelee, WantsToPickupItem};
use crate::gamelog::GameLog;
use crate::map::TileType;
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

fn get_item(
    mut commands: EntityCommands,
    items: Query<(Entity, &Position), With<Item>>,
    player: Entity,
    player_pos: &Position,
    log: &mut GameLog,
) -> bool {
    let mut target_item: Option<Entity> = None;
    for (item_entity, position) in items.iter() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
            break;
        }
    }

    match target_item {
        None => {
            log.entries
                .push("There is nothing here to pick up.".to_string());
            return false;
        }
        Some(item) => {
            commands.insert(WantsToPickupItem {
                collected_by: player,
                item,
            });
            return true;
        }
    }
}

pub fn try_next_level(player_pos: &Position, map: Res<Map>, mut log: ResMut<GameLog>) -> bool {
    let player_idx = map.xy_idx(player_pos.x, player_pos.y);
    if map.tiles[player_idx] == TileType::DownStairs {
        true
    } else {
        log.entries
            .push("There is no way down from here.".to_string());
        false
    }
}

pub fn waiting_system(
    mut commands: Commands,
    mut waiters: Query<(Entity, &mut CombatStats, &Viewshed), With<Waiting>>,
    monsters: Query<Entity, With<Monster>>,
    map: Res<Map>,
) {
    for (entity, mut combat, viewshed) in waiters.iter_mut() {
        let mut can_heal = true;

        for tile in viewshed.visible_tiles.iter() {
            let idx = map.xy_idx(tile.x, tile.y);
            for entity_id in map.tile_content[idx].iter() {
                if monsters.contains(*entity_id) {
                    can_heal = false;
                    break;
                }
            }
        }

        if can_heal {
            combat.hp = i32::min(combat.hp + 1, combat.max_hp);
        }

        commands.entity(entity).remove::<Waiting>();
    }
}

pub fn player_input_system(
    mut commands: Commands,
    mut players: Query<(Entity, &mut Position, &mut Viewshed), (With<Player>, Without<Item>)>,
    enemies: Query<&CombatStats>,
    items: Query<(Entity, &Position), With<Item>>,
    map: Res<Map>,
    key: NonSend<Key>,
    mut state: ResMut<RunState>,
    mut log: ResMut<GameLog>,
) {
    if *state != RunState::AwaitingInput {
        return;
    }

    let (player, mut pos, mut viewshed) = players.single_mut();
    let mut player_commands = commands.entity(player);
    let mut new_state = RunState::AwaitingInput;

    if let Some(k) = *key {
        if let Some(delta) = MOVEMENT_KEYS.get(&k) {
            if try_move_player(
                player_commands,
                enemies,
                &mut pos,
                &mut viewshed,
                &map,
                delta.0,
                delta.1,
            ) {
                new_state = RunState::PlayerTurn;
            }
        } else {
            match k {
                VirtualKeyCode::G => {
                    if get_item(player_commands, items, player, &pos, &mut log) {
                        new_state = RunState::PlayerTurn;
                    }
                }
                VirtualKeyCode::I => new_state = RunState::ShowInventory,
                VirtualKeyCode::D => new_state = RunState::ShowDropItem,
                VirtualKeyCode::R => new_state = RunState::ShowRemoveItem,

                VirtualKeyCode::Escape => new_state = RunState::SaveGame,

                VirtualKeyCode::Period => {
                    if try_next_level(pos.as_ref(), map, log) {
                        new_state = RunState::NextLevel;
                    }
                }

                VirtualKeyCode::Numpad5 | VirtualKeyCode::Space => {
                    player_commands.insert(Waiting {});
                    new_state = RunState::PlayerTurn;
                }

                _ => {}
            }
        }
    }

    *state = new_state;
}
