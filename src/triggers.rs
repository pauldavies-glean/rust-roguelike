use bevy_ecs::prelude::*;
use rltk::{to_cp437, BLACK, ORANGE};

use crate::{
    components::{
        EntityMoved, EntryTrigger, Hidden, InflictsDamage, Name, Position, SingleActivation,
    },
    damage::DamageEvent,
    gamelog::GameLog,
    map::Map,
    particle::ParticleBuilder,
};

pub fn trigger_system(
    mut commands: Commands,
    mut movers: Query<(Entity, &Position), With<EntityMoved>>,
    triggers: Query<
        (&Name, Option<&InflictsDamage>, Option<&SingleActivation>),
        With<EntryTrigger>,
    >,
    map: Res<Map>,
    mut log: ResMut<GameLog>,
    mut particle: ResMut<ParticleBuilder>,
    mut damage_writer: EventWriter<DamageEvent>,
) {
    for (entity, pos) in movers.iter_mut() {
        let idx = map.xy_idx(pos.x, pos.y);
        for other in map.tile_content[idx].iter() {
            if entity != *other {
                if let Ok((name, inflict, single)) = triggers.get(*other) {
                    log.entries.push(format!("{} triggers!", &name.name));
                    commands.entity(*other).remove::<Hidden>();

                    if let Some(inflict) = inflict {
                        particle.request(pos.x, pos.y, ORANGE, BLACK, to_cp437('‼'), 200.0);
                        damage_writer.send(DamageEvent {
                            who: entity,
                            value: inflict.damage,
                        });
                    }

                    if let Some(_) = single {
                        commands.entity(*other).despawn();
                    }
                }
            }
        }

        commands.entity(entity).remove::<EntityMoved>();
    }
}
