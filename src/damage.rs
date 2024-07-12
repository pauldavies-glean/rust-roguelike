use bevy_ecs::prelude::*;

use crate::{
    components::{CombatStats, Name, Player, Position, SufferDamage},
    gamelog::GameLog,
    map::Map,
    RunState,
};

pub fn damage_system(
    mut commands: Commands,
    mut victims: Query<(
        Entity,
        &mut CombatStats,
        &Name,
        &SufferDamage,
        &Position,
        Option<&Player>,
    )>,
    mut log: ResMut<GameLog>,
    mut state: ResMut<RunState>,
    mut map: ResMut<Map>,
) {
    for (victim, mut stats, name, damage, pos, player) in victims.iter_mut() {
        stats.hp -= damage.amount.iter().sum::<i32>();

        let idx = map.xy_idx(pos.x, pos.y);
        map.bloodstains.insert(idx);

        if stats.hp < 1 {
            match player {
                None => {
                    commands.entity(victim).despawn();
                    log.entries.push(format!("{} dies horribly!", &name.name));
                }
                Some(_) => {
                    *state = RunState::GameOver;
                }
            }
        }

        commands.entity(victim).remove::<SufferDamage>();
    }
}
