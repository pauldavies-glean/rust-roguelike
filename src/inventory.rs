use bevy_ecs::prelude::*;

use crate::{
    components::{
        AreaOfEffect, CombatStats, Confused, Confusion, Consumable, InBackpack, InflictsDamage,
        Item, Name, Player, Position, ProvidesHealing, SufferDamage, WantsToDropItem,
        WantsToPickupItem, WantsToUseItem,
    },
    gamelog::GameLog,
    map::Map,
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
    users: Query<(Entity, Option<&WantsToUseItem>, Option<&Player>)>,
    mut combatants: Query<&mut CombatStats>,
    mut mobs: Query<(&Name, Option<&mut SufferDamage>)>,
    consumables: Query<(
        &Name,
        Option<&Consumable>,
        Option<&ProvidesHealing>,
        Option<&InflictsDamage>,
        Option<&Confusion>,
        Option<&AreaOfEffect>,
    )>,
    mut log: ResMut<GameLog>,
    map: Res<Map>,
) {
    for (user, use_item, player) in users.iter() {
        if let Some(use_item) = use_item {
            if let Ok((item_name, consumable, healing, inflict, confusion, aoe)) =
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
                                let mut blast_tiles =
                                    rltk::field_of_view(target, aoe.radius, &*map);
                                blast_tiles.retain(|p| {
                                    p.x > 0
                                        && p.x < map.width - 1
                                        && p.y > 0
                                        && p.y < map.height - 1
                                });
                                for tile_idx in blast_tiles.iter() {
                                    let idx = map.xy_idx(tile_idx.x, tile_idx.y);
                                    for mob in map.tile_content[idx]
                                        .iter()
                                        .filter(|&p| combatants.contains(*p))
                                    {
                                        targets.push(*mob);
                                    }
                                }
                            }
                        }
                    }
                }

                let mut used_up = false;

                if let Some(healing) = healing {
                    for target in targets.iter() {
                        if let Ok(mut stats) = combatants.get_mut(*target) {
                            // TODO don't use if full hp!
                            stats.hp = i32::min(stats.max_hp, stats.hp + healing.heal_amount);
                            used_up = true;

                            if player.is_some() {
                                log.entries.push(format!(
                                    "You drink the {}, healing {} hp.",
                                    item_name.name, healing.heal_amount
                                ));
                            }
                        }
                    }
                }

                if let Some(inflict) = inflict {
                    for target in targets.iter() {
                        if let Ok((mob_name, suffering)) = mobs.get_mut(*target) {
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
                        }
                    }
                }

                if let Some(confusion) = confusion {
                    for target in targets.iter() {
                        if let Ok((mob_name, _suffering)) = mobs.get_mut(*target) {
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
                        }
                    }
                }

                if used_up && consumable.is_some() {
                    commands.entity(use_item.item).despawn();
                }
            }

            commands.entity(user).remove::<WantsToUseItem>();
        }
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
