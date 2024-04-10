mod components;
mod map;
mod monster_ai_system;
mod player;
mod rect;
mod visibility_system;
pub use components::*;
pub use map::*;
pub use monster_ai_system::*;
pub use player::*;
pub use rect::*;
pub use visibility_system::*;

use bevy_ecs::prelude::*;
use rltk::{GameState, Rltk, VirtualKeyCode, RGB};

struct State {
    world: World,
    schedule: Schedule,
}

#[derive(Default)]
pub struct GameTime {
    time: u32,
}

pub type Key = Option<VirtualKeyCode>;

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        self.world.insert_non_send_resource::<Key>(ctx.key);
        self.schedule.run(&mut self.world);

        let mut query = self.world.query::<(&Position, &Renderable)>();

        let map = self.world.non_send_resource::<Map>();
        draw_map(map, ctx);

        for (pos, render) in query.iter(&self.world) {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
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
        Name {
            name: "Player".to_string(),
        },
        Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        },
    ));

    let mut rng = rltk::RandomNumberGenerator::new();

    // don't put a monster where the player is!
    for (i, room) in map.rooms.iter().skip(1).enumerate() {
        let (x, y) = room.center();

        let glyph: rltk::FontCharType;
        let name: String;
        let roll = rng.roll_dice(1, 2);
        match roll {
            1 => {
                glyph = rltk::to_cp437('g');
                name = "Goblin".to_string();
            }
            _ => {
                glyph = rltk::to_cp437('o');
                name = "Orc".to_string();
            }
        }

        world.spawn((
            Position { x, y },
            Renderable {
                glyph,
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            },
            Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
                dirty: true,
            },
            Monster {},
            Name {
                name: format!("{} #{}", &name, i),
            },
        ));
    }

    world.insert_non_send_resource(map);
    world.insert_non_send_resource(GameTime { time: 0 });

    let mut state = State {
        world,
        schedule: Schedule::default(),
    };

    state
        .schedule
        .add_systems((player_input_system, visibility_system, monster_ai_system));

    rltk::main_loop(context, state)
}
