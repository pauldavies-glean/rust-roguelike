use bevy_ecs::prelude::*;
use rltk::console;

use crate::components::{CombatStats, Name, Player, SufferDamage};

pub fn damage_system(
    mut commands: Commands,
    mut victims: Query<(
        Entity,
        &mut CombatStats,
        &Name,
        &SufferDamage,
        Option<&Player>,
    )>,
) {
    for (victim, mut stats, name, damage, player) in victims.iter_mut() {
        stats.hp -= damage.amount.iter().sum::<i32>();

        if stats.hp < 1 {
            match player {
                None => {
                    commands.entity(victim).despawn();
                    console::log(&format!("{} dies horribly!", &name.name));
                }
                Some(_) => console::log("ya dead"),
            }
        } else {
            commands.entity(victim).remove::<SufferDamage>();
        }
    }
}
