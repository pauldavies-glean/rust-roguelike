use bevy_ecs::prelude::*;

use crate::{
    components::{CombatStats, Name, SufferDamage, WantsToMelee},
    gamelog::GameLog,
};

pub fn melee_combat_system(
    mut commands: Commands,
    mut attackers: Query<(Entity, &WantsToMelee, &Name, &CombatStats)>,
    combatants: Query<&CombatStats>,
    mut sufferers: Query<&mut SufferDamage>,
    names: Query<&Name>,
    mut log: ResMut<GameLog>,
) {
    for (attacker, wants_melee, name, stats) in attackers.iter_mut() {
        let victim = wants_melee.target;
        let target_stats = combatants.get(victim).unwrap();
        if target_stats.hp > 0 {
            let target_name = names.get(victim).unwrap();

            let damage = stats.power - target_stats.defense;
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

                SufferDamage::new_damage(
                    commands.entity(victim),
                    sufferers
                        .get_mut(victim)
                        .map_or(None, |x| Some(x.into_inner())),
                    damage,
                );
            }
        }

        commands.entity(attacker).remove::<WantsToMelee>();
    }
}
