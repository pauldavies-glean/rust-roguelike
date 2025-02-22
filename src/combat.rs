use bevy_ecs::prelude::*;
use rltk::{to_cp437, BLACK, ORANGE};

use crate::{
    components::{
        CombatStats, DefenseBonus, Equipped, HungerClock, HungerState, MeleePowerBonus, Name,
        Position, WantsToMelee,
    },
    damage::DamageEvent,
    gamelog::GameLog,
    particle::ParticleBuilder,
};

pub fn melee_combat_system(
    mut commands: Commands,
    mut attackers: Query<(
        Entity,
        &WantsToMelee,
        &Name,
        &CombatStats,
        Option<&HungerClock>,
    )>,
    combatants: Query<&CombatStats>,
    power_bonuses: Query<(&MeleePowerBonus, &Equipped)>,
    defense_bonuses: Query<(&DefenseBonus, &Equipped)>,
    names: Query<&Name>,
    positions: Query<&Position>,
    mut log: ResMut<GameLog>,
    mut particle: ResMut<ParticleBuilder>,
    mut damage_writer: EventWriter<DamageEvent>,
) {
    for (attacker, wants_melee, name, stats, hunger) in attackers.iter_mut() {
        let victim = wants_melee.target;
        let target_stats = combatants.get(victim).unwrap();
        if target_stats.hp > 0 {
            let target_name = names.get(victim).unwrap();

            let mut power = stats.power;
            for (bonus, equipped) in power_bonuses.iter() {
                if equipped.owner == attacker {
                    power += bonus.power;
                }
            }

            if let Some(hunger) = hunger {
                if hunger.state == HungerState::WellFed {
                    power += 1;
                }
            }

            let mut defense = target_stats.defense;
            for (bonus, equipped) in defense_bonuses.iter() {
                if equipped.owner == victim {
                    defense += bonus.defense;
                }
            }

            let pos = positions.get(victim);
            if let Ok(pos) = pos {
                particle.request(pos.x, pos.y, ORANGE, BLACK, to_cp437('‼'), 200.0);
            }

            let damage = power - defense;
            if damage <= 0 {
                log.entries.push(format!(
                    "{} is unable to hurt {}",
                    &name.name, &target_name.name
                ));
            } else {
                log.entries.push(format!(
                    "{} hits {}, for {} hp.",
                    &name.name, &target_name.name, damage
                ));

                damage_writer.send(DamageEvent {
                    who: victim,
                    value: damage,
                });
            }
        }

        commands.entity(attacker).remove::<WantsToMelee>();
    }
}
