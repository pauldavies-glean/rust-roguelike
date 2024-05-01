use bevy_ecs::prelude::*;

use crate::{
    components::{CombatStats, Name, Player, SufferDamage},
    gamelog::GameLog,
};

pub fn damage_system(
    mut commands: Commands,
    mut victims: Query<(
        Entity,
        &mut CombatStats,
        &Name,
        &SufferDamage,
        Option<&Player>,
    )>,
    mut log: ResMut<GameLog>,
) {
    for (victim, mut stats, name, damage, player) in victims.iter_mut() {
        stats.hp -= damage.amount.iter().sum::<i32>();

        if stats.hp < 1 {
            match player {
                None => {
                    commands.entity(victim).despawn();
                    log.entries.push(format!("{} dies horribly!", &name.name));
                }
                Some(_) => log.entries.push("ya dead".to_owned()),
            }
        }

        commands.entity(victim).remove::<SufferDamage>();
    }
}
