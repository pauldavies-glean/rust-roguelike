use bevy_ecs::prelude::*;
use rltk::FontCharType;

use crate::{
    components::{ParticleLifetime, Position, Renderable},
    FrameTime,
};

struct ParticleRequest {
    x: i32,
    y: i32,
    fg: (u8, u8, u8),
    bg: (u8, u8, u8),
    glyph: rltk::FontCharType,
    lifetime: f32,
}

#[derive(Resource)]
pub struct ParticleBuilder {
    requests: Vec<ParticleRequest>,
}

impl ParticleBuilder {
    pub fn new() -> ParticleBuilder {
        ParticleBuilder {
            requests: Vec::new(),
        }
    }

    pub fn request(
        &mut self,
        x: i32,
        y: i32,
        fg: (u8, u8, u8),
        bg: (u8, u8, u8),
        glyph: FontCharType,
        lifetime: f32,
    ) {
        self.requests.push(ParticleRequest {
            x,
            y,
            fg,
            bg,
            glyph,
            lifetime,
        });
    }
}

pub fn cull_dead_particles_system(
    mut commands: Commands,
    mut particles: Query<(Entity, &mut ParticleLifetime)>,
    time: NonSend<FrameTime>,
) {
    for (entity, mut lifetime) in particles.iter_mut() {
        lifetime.lifetime_ms -= *time;
        if lifetime.lifetime_ms < 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn spawn_particles_system(mut commands: Commands, mut builder: ResMut<ParticleBuilder>) {
    for request in builder.requests.iter() {
        commands.spawn((
            Position {
                x: request.x,
                y: request.y,
            },
            Renderable {
                fg: request.fg,
                bg: request.bg,
                glyph: request.glyph,
                render_order: 0,
            },
            ParticleLifetime {
                lifetime_ms: request.lifetime,
            },
        ));
    }

    builder.requests.clear();
}
