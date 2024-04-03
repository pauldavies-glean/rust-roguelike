mod components;
mod map;
mod player;
pub use components::*;
pub use map::*;
pub use player::*;

use bevy_ecs::prelude::*;
use rltk::{GameState, Rltk, RGB};

struct State {
    world: World,
    schedule: Schedule,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        self.world.insert_non_send_resource(ctx.key);
        self.schedule.run(&mut self.world);

        draw_map(self.world.non_send_resource::<Map>(), ctx);

        let mut query = self.world.query::<(&Position, &Renderable)>();
        for (pos, render) in query.iter(&self.world) {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}

fn left_walker_system(mut query: Query<(&mut Position, &LeftMover)>) {
    for (mut pos, _left) in &mut query {
        pos.x -= 1;
        if pos.x < 0 {
            pos.x = 79;
        }
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;

    let mut state = State {
        world: World::new(),
        schedule: Schedule::default(),
    };

    let map = new_map();
    state.world.insert_non_send_resource(map);

    state.world.spawn((
        Position { x: 40, y: 25 },
        Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        },
        Player {},
    ));

    for i in 0..10 {
        state.world.spawn((
            Position { x: i * 7, y: 20 },
            Renderable {
                glyph: rltk::to_cp437('☺'),
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            },
            LeftMover {},
        ));
    }

    state
        .schedule
        .add_systems((player_input, left_walker_system));

    rltk::main_loop(context, state)
}
