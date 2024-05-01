use bevy_ecs::prelude::*;

use crate::{
    components::{
        CombatStats, InBackpack, Item, Name, Player, Position, Potion, WantsToDrinkPotion,
        WantsToDropItem, WantsToPickupItem,
    },
    gamelog::GameLog,
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

pub fn potion_use_system(
    mut commands: Commands,
    mut drinkers: Query<(&WantsToDrinkPotion, &mut CombatStats, Option<&Player>)>,
    potions: Query<(&Name, &Potion)>,
    mut log: ResMut<GameLog>,
) {
    for (drink, mut stats, player) in drinkers.iter_mut() {
        let potion_stuff = potions.get(drink.potion).ok();
        match potion_stuff {
            None => {}
            Some((name, potion)) => {
                stats.hp = i32::min(stats.max_hp, stats.hp + potion.heal_amount);
                if player.is_some() {
                    log.entries.push(format!(
                        "You drink the {}, healing {} hp.",
                        name.name, potion.heal_amount
                    ));
                }

                commands.entity(drink.potion).despawn();
            }
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
