mod components;
mod map;
mod player;
mod rect;
pub use components::*;
pub use map::*;
pub use player::*;
pub use rect::*;

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

        draw_map(self.world.non_send_resource::<Tiles>(), ctx);

        let mut query = self.world.query::<(&Position, &Renderable)>();
        for (pos, render) in query.iter(&self.world) {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
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

    let (rooms, map) = new_map_rooms_and_corridors();
    state.world.insert_non_send_resource(map);

    let (player_x, player_y) = rooms[0].center();

    state.world.spawn((
        Position {
            x: player_x,
            y: player_y,
        },
        Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        },
        Player {},
    ));

    state.schedule.add_systems(player_input);

    rltk::main_loop(context, state)
}
