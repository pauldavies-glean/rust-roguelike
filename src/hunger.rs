use bevy_ecs::prelude::*;

use crate::{
    components::{HungerClock, HungerState, Player, SufferDamage},
    gamelog::GameLog,
    RunState,
};

pub fn hunger_system(
    mut commands: Commands,
    mut hungry: Query<(
        Entity,
        &mut HungerClock,
        Option<&Player>,
        Option<&mut SufferDamage>,
    )>,
    state: Res<RunState>,
    mut log: ResMut<GameLog>,
) {
    for (entity, mut clock, player, suffering) in hungry.iter_mut() {
        let proceed = match *state {
            RunState::PlayerTurn => player.is_some(),
            RunState::MonsterTurn => player.is_none(),
            _ => false,
        };

        if proceed {
            clock.duration -= 1;
            if clock.duration < 1 {
                match clock.state {
                    HungerState::WellFed => {
                        clock.state = HungerState::Normal;
                        clock.duration = 200;
                        if player.is_some() {
                            log.entries.push("You are no longer well fed.".to_string())
                        }
                    }
                    HungerState::Normal => {
                        clock.state = HungerState::Hungry;
                        clock.duration = 200;
                        if player.is_some() {
                            log.entries.push("You are hungry.".to_string())
                        }
                    }
                    HungerState::Hungry => {
                        clock.state = HungerState::Starving;
                        clock.duration = 200;
                        if player.is_some() {
                            log.entries.push("You are staving!".to_string())
                        }
                    }
                    HungerState::Starving => {
                        // Inflict damage from hunger
                        if player.is_some() {
                            log.entries.push(
                                "Your hunger pangs are getting painful! You suffer 1 hp damage."
                                    .to_string(),
                            )
                        }
                        SufferDamage::new_damage(
                            commands.entity(entity),
                            suffering.map_or(None, |x| Some(x.into_inner())),
                            1,
                        )
                    }
                }
            }
        }
    }
}
