mod components;
mod map;
mod player;
mod rect;
mod visibility_system;
pub use components::*;
pub use map::*;
pub use player::*;
pub use rect::*;
pub use visibility_system::*;

use bevy_ecs::prelude::*;
use rltk::{GameState, Rltk, VirtualKeyCode, RGB};

struct State {
    world: World,
    schedule: Schedule,
}

pub type Key = Option<VirtualKeyCode>;

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        self.world.insert_non_send_resource::<Key>(ctx.key);
        self.schedule.run(&mut self.world);

        let map = self.world.non_send_resource::<Map>();
        draw_map(map, ctx);

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

    let mut world = World::new();

    let map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();
    world.insert_non_send_resource(map);

    world.spawn((
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
        Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        },
    ));

    let mut state = State {
        world,
        schedule: Schedule::default(),
    };

    state
        .schedule
        .add_systems((player_input, visibility_system));

    rltk::main_loop(context, state)
}
