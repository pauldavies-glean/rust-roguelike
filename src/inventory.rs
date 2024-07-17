use bevy_ecs::prelude::*;
use rltk::{to_cp437, BLACK, GREEN, MAGENTA, ORANGE, RED};

use crate::{
    components::{
        AreaOfEffect, CombatStats, Confused, Confusion, Consumable, Equippable, Equipped,
        HungerClock, HungerState, InBackpack, InflictsDamage, Item, Name, Player, Position,
        ProvidesFood, ProvidesHealing, SufferDamage, WantsToDropItem, WantsToPickupItem,
        WantsToRemoveItem, WantsToUseItem,
    },
    gamelog::GameLog,
    map::Map,
    particle::ParticleBuilder,
};

pub fn inventory_system(
    mut commands: Commands,
    wants_pickup: Query<(Entity, &WantsToPickupItem, Option<&Player>)>,
    items: Query<&Name, With<Item>>,
    mut log: ResMut<GameLog>,
) {
    for (entity, pickup, player) in wants_pickup.iter() {
        commands.entity(pickup.item).remove::<Position>();
        commands.entity(pickup.item).insert(InBackpack {
            owner: pickup.collected_by,
        });

        if player.is_some() {
            let entity_name = items.get(pickup.item).unwrap();

            log.entries
                .push(format!("You pick up the {}.", entity_name.name));
        }

        commands.entity(entity).remove::<WantsToPickupItem>();
    }
}

pub fn item_use_system(
    mut commands: Commands,
    mut users: Query<(
        Entity,
        &WantsToUseItem,
        Option<&mut HungerClock>,
        Option<&Player>,
    )>,
    mut combatants: Query<(&mut CombatStats, Option<&Position>)>,
    mut mobs: Query<(&Name, Option<&mut SufferDamage>, Option<&Position>)>,
    consumables: Query<(
        &Name,
        Option<&Consumable>,
        Option<&ProvidesHealing>,
        Option<&InflictsDamage>,
        Option<&Confusion>,
        Option<&ProvidesFood>,
        Option<&AreaOfEffect>,
    )>,
    equippables: Query<(&Name, &Equippable)>,
    equipped_items: Query<(Entity, &Equipped, &Name)>,
    mut log: ResMut<GameLog>,
    map: Res<Map>,
    mut particle: ResMut<ParticleBuilder>,
) {
    for (user, use_item, hunger, player) in users.iter_mut() {
        if let Ok((item_name, consumable, healing, inflict, confusion, edible, aoe)) =
            consumables.get(use_item.item)
        {
            // Targeting
            let mut targets: Vec<Entity> = Vec::new();
            match use_item.target {
                None => {
                    targets.push(user);
                }
                Some(target) => {
                    match aoe {
                        None => {
                            // Single target in tile
                            let idx = map.xy_idx(target.x, target.y);
                            for mob in map.tile_content[idx]
                                .iter()
                                .filter(|&p| combatants.contains(*p))
                            {
                                targets.push(*mob);
                            }
                        }
                        Some(aoe) => {
                            // AoE
                            let mut blast_tiles = rltk::field_of_view(target, aoe.radius, &*map);
                            blast_tiles.retain(|p| {
                                p.x > 0 && p.x < map.width - 1 && p.y > 0 && p.y < map.height - 1
                            });
                            for tile in blast_tiles.iter() {
                                let idx = map.xy_idx(tile.x, tile.y);
                                for mob in map.tile_content[idx]
                                    .iter()
                                    .filter(|&p| combatants.contains(*p))
                                {
                                    targets.push(*mob);
                                }

                                particle.request(
                                    tile.x,
                                    tile.y,
                                    ORANGE,
                                    BLACK,
                                    to_cp437('░'),
                                    200.0,
                                );
                            }
                        }
                    }
                }
            }

            let mut used_up = false;

            if let Some(healing) = healing {
                for target in targets.iter() {
                    if let Ok((mut stats, pos)) = combatants.get_mut(*target) {
                        // TODO don't use if full hp!
                        stats.hp = i32::min(stats.max_hp, stats.hp + healing.heal_amount);
                        used_up = true;

                        if player.is_some() {
                            log.entries.push(format!(
                                "You drink the {}, healing {} hp.",
                                item_name.name, healing.heal_amount
                            ));
                        }

                        if let Some(pos) = pos {
                            particle.request(pos.x, pos.y, GREEN, BLACK, to_cp437('♥'), 200.0);
                        }
                    }
                }
            }

            if let Some(inflict) = inflict {
                for target in targets.iter() {
                    if let Ok((mob_name, suffering, pos)) = mobs.get_mut(*target) {
                        used_up = true;

                        SufferDamage::new_damage(
                            commands.entity(*target),
                            suffering.map_or(None, |x| Some(x.into_inner())),
                            inflict.damage,
                        );

                        if player.is_some() {
                            log.entries.push(format!(
                                "You use {} on {}, inflicting {} hp.",
                                item_name.name, mob_name.name, inflict.damage
                            ));
                        }

                        if let Some(pos) = pos {
                            particle.request(pos.x, pos.y, RED, BLACK, to_cp437('‼'), 200.0);
                        }
                    }
                }
            }

            if let Some(confusion) = confusion {
                for target in targets.iter() {
                    if let Ok((mob_name, _suffering, pos)) = mobs.get_mut(*target) {
                        used_up = true;

                        commands.entity(*target).insert(Confused {
                            turns: confusion.turns,
                        });

                        if player.is_some() {
                            log.entries.push(format!(
                                "You use {} on {}, confusing them.",
                                item_name.name, mob_name.name,
                            ));
                        }

                        if let Some(pos) = pos {
                            particle.request(pos.x, pos.y, MAGENTA, BLACK, to_cp437('?'), 200.0);
                        }
                    }
                }
            }

            if edible.is_some() {
                if let Some(mut hunger) = hunger {
                    used_up = true;

                    hunger.state = HungerState::WellFed;
                    hunger.duration = 20;

                    if player.is_some() {
                        log.entries.push(format!("You eat the {}.", item_name.name));
                    }
                }
            }

            if used_up && consumable.is_some() {
                commands.entity(use_item.item).despawn();
            }
        }

        if let Ok((item_name, can_equip)) = equippables.get(use_item.item) {
            let target_slot = can_equip.slot;

            // Remove any items in the same slot
            let mut to_unequip: Vec<Entity> = Vec::new();
            for (item_entity, already_equipped, name) in equipped_items.iter() {
                if already_equipped.owner == user && already_equipped.slot == target_slot {
                    to_unequip.push(item_entity);
                    if player.is_some() {
                        log.entries.push(format!("You unequip {}.", name.name));
                    }
                }
            }
            for item in to_unequip.iter() {
                commands
                    .entity(*item)
                    .remove::<Equipped>()
                    .insert(InBackpack { owner: user });
            }

            // Wield the item
            commands
                .entity(use_item.item)
                .insert(Equipped {
                    owner: user,
                    slot: target_slot,
                })
                .remove::<InBackpack>();
            if player.is_some() {
                log.entries.push(format!("You equip {}.", item_name.name))
            }
        }

        commands.entity(user).remove::<WantsToUseItem>();
    }
}

pub fn drop_system(
    mut commands: Commands,
    droppers: Query<(Entity, &WantsToDropItem, &Position, Option<&Player>)>,
    items: Query<&Name, With<Item>>,
    mut log: ResMut<GameLog>,
) {
    for (entity, intent, position, player) in droppers.iter() {
        commands.entity(intent.item).insert(Position {
            x: position.x,
            y: position.y,
        });
        commands.entity(entity).remove::<WantsToDropItem>();

        if player.is_some() {
            let item_name = items.get(intent.item).unwrap();
            log.entries
                .push(format!("You drop the {}.", item_name.name));
        }
    }
}

pub fn item_remove_system(
    mut commands: Commands,
    removers: Query<(Entity, &WantsToRemoveItem, Option<&Player>)>,
    names: Query<&Name>,
    mut log: ResMut<GameLog>,
) {
    for (entity, intent, player) in removers.iter() {
        commands
            .entity(intent.item)
            .remove::<Equipped>()
            .insert(InBackpack { owner: entity });
        commands.entity(entity).remove::<WantsToRemoveItem>();

        if player.is_some() {
            log.entries.push(format!(
                "You remove the {}.",
                names.get(intent.item).unwrap().name,
            ));
        }
    }
}
