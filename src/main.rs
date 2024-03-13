use bevy_ecs::prelude::*;
use rltk::{GameState, Rltk, VirtualKeyCode, RGB};
use std::cmp::{max, min};

#[derive(Component)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Renderable {
    glyph: rltk::FontCharType,
    fg: RGB,
    bg: RGB,
}

#[derive(Component)]
struct LeftMover {}

#[derive(Component, Debug)]
struct Player {}

struct State {
    world: World,
    schedule: Schedule,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        player_input(self, ctx);
        self.schedule.run(&mut self.world);

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

fn try_move_player(delta_x: i32, delta_y: i32, world: &mut World) {
    let mut query = world.query::<(&mut Position, &Player)>();
    for (mut pos, _player) in query.iter_mut(world) {
        pos.x = min(79, max(0, pos.x + delta_x));
        pos.y = min(49, max(0, pos.y + delta_y));
    }
}

fn player_input(state: &mut State, context: &mut Rltk) {
    match context.key {
        None => {}
        Some(key) => match key {
            VirtualKeyCode::Left => try_move_player(-1, 0, &mut state.world),
            VirtualKeyCode::Right => try_move_player(1, 0, &mut state.world),
            VirtualKeyCode::Up => try_move_player(0, -1, &mut state.world),
            VirtualKeyCode::Down => try_move_player(0, 1, &mut state.world),
            _ => {}
        },
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

    state.schedule.add_systems(left_walker_system);

    rltk::main_loop(context, state)
}
