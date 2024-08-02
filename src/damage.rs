use bevy_ecs::prelude::*;

use crate::{
    components::{CombatStats, Name, Player, Position, SufferDamage},
    gamelog::GameLog,
    map::Map,
    RunState,
};

#[derive(Event)]
pub struct DamageEvent {
    pub who: Entity,
    pub value: i32,
}

pub fn damage_event_reader(
    mut commands: Commands,
    mut reader: EventReader<DamageEvent>,
    mut sufferers: Query<&mut SufferDamage>,
) {
    for event in reader.read() {
        if let Ok(mut suffer) = sufferers.get_mut(event.who) {
            suffer.amount.push(event.value);
        } else {
            commands.entity(event.who).insert(SufferDamage {
                amount: vec![event.value],
            });
        }
    }
}

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
