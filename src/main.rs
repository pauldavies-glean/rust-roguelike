mod ai;
mod combat;
mod components;
mod damage;
mod map;
mod player;
mod rect;
mod visibility;

use bevy_ecs::prelude::*;
use components::{BlocksTile, CombatStats, Monster, Name, Player, Position, Renderable, Viewshed};
use map::Map;
use rltk::{
    main_loop, to_cp437, BError, FontCharType, GameState, RandomNumberGenerator, Rltk, RltkBuilder,
    VirtualKeyCode, BLACK, RED, RGB, YELLOW,
};

struct State {
    world: World,
    schedule: Schedule,
}

pub type Key = Option<VirtualKeyCode>;

#[derive(PartialEq, Copy, Clone, Resource)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        let state = *self.world.resource::<RunState>();
        let new_state = match state {
            RunState::PlayerTurn => RunState::MonsterTurn,
            _ => RunState::AwaitingInput,
        };

        self.world.insert_non_send_resource::<Key>(ctx.key);
        self.schedule.run(&mut self.world);
        if *self.world.resource::<RunState>() == state {
            self.world.insert_resource(new_state);
        }

        let mut query = self.world.query::<(&Position, &Renderable)>();

        let map = self.world.resource::<Map>();
        map.draw(ctx);

        for (pos, render) in query.iter(&self.world) {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }
    }
}

fn main() -> BError {
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
            glyph: to_cp437('@'),
            fg: RGB::named(YELLOW),
            bg: RGB::named(BLACK),
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
        CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        },
    ));

    let mut rng = RandomNumberGenerator::new();

    // don't put a monster where the player is!
    for (i, room) in map.rooms.iter().skip(1).enumerate() {
        let (x, y) = room.center();

        let glyph: FontCharType;
        let name: String;
        let roll = rng.roll_dice(1, 2);
        match roll {
            1 => {
                glyph = to_cp437('g');
                name = "Goblin".to_string();
            }
            _ => {
                glyph = to_cp437('o');
                name = "Orc".to_string();
            }
        }

        world.spawn((
            Position { x, y },
            Renderable {
                glyph,
                fg: RGB::named(RED),
                bg: RGB::named(BLACK),
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
            BlocksTile {},
            CombatStats {
                max_hp: 16,
                hp: 16,
                defense: 1,
                power: 4,
            },
        ));
    }

    world.insert_resource(map);
    world.insert_resource(RunState::PreRun);

    let mut state = State {
        world,
        schedule: Schedule::default(),
    };

    state.schedule.add_systems((
        player::player_input_system,
        visibility::visibility_system,
        ai::monster_ai_system,
        combat::melee_combat_system,
        damage::damage_system,
        map::map_indexing_system,
    ));

    main_loop(context, state)
}
