use bevy_ecs::prelude::*;
use rltk::{GameState, Rltk, VirtualKeyCode, RGB};
use std::cmp::{max, min};

#[derive(PartialEq, Copy, Clone)]
enum TileType {
    Wall,
    Floor,
}

pub fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * 80) + x as usize
}

type Key = Option<VirtualKeyCode>;
type Map = Vec<TileType>;

fn new_map() -> Map {
    let mut map = vec![TileType::Floor; 80 * 50];

    // Make the boundaries walls
    for x in 0..80 {
        map[xy_idx(x, 0)] = TileType::Wall;
        map[xy_idx(x, 49)] = TileType::Wall;
    }
    for y in 0..50 {
        map[xy_idx(0, y)] = TileType::Wall;
        map[xy_idx(79, y)] = TileType::Wall;
    }

    // Now we'll randomly splat a bunch of walls. It won't be pretty, but it's a decent illustration.
    // First, obtain the thread-local RNG:
    let mut rng = rltk::RandomNumberGenerator::new();

    for _i in 0..400 {
        let x = rng.roll_dice(1, 79);
        let y = rng.roll_dice(1, 49);
        let idx = xy_idx(x, y);
        if idx != xy_idx(40, 25) {
            map[idx] = TileType::Wall;
        }
    }

    map
}

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

        self.world.insert_non_send_resource(ctx.key);
        self.schedule.run(&mut self.world);

        draw_map(self, ctx);

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

fn try_move_player(pos: &mut Position, map: &Map, delta_x: i32, delta_y: i32) {
    let destination_idx = xy_idx(pos.x + delta_x, pos.y + delta_y);
    if map[destination_idx] != TileType::Wall {
        pos.x = min(79, max(0, pos.x + delta_x));
        pos.y = min(49, max(0, pos.y + delta_y));
    }
}

fn player_input(
    mut query: Query<&mut Position, With<Player>>,
    map: NonSend<Map>,
    key: NonSend<Key>,
) {
    let mut binding = query.single_mut();
    let pos = binding.as_mut();
    match key.to_owned() {
        None => {}
        Some(key) => match key {
            VirtualKeyCode::Left => try_move_player(pos, &map, -1, 0),
            VirtualKeyCode::Right => try_move_player(pos, &map, 1, 0),
            VirtualKeyCode::Up => try_move_player(pos, &map, 0, -1),
            VirtualKeyCode::Down => try_move_player(pos, &map, 0, 1),
            _ => {}
        },
    }
}

fn draw_map(state: &State, ctx: &mut Rltk) {
    let map = state.world.non_send_resource::<Map>();
    let mut y = 0;
    let mut x = 0;
    for tile in map.iter() {
        // Render a tile depending upon the tile type
        match tile {
            TileType::Floor => {
                ctx.set(
                    x,
                    y,
                    RGB::from_f32(0.5, 0.5, 0.5),
                    RGB::from_f32(0., 0., 0.),
                    rltk::to_cp437('.'),
                );
            }
            TileType::Wall => {
                ctx.set(
                    x,
                    y,
                    RGB::from_f32(0.0, 1.0, 0.0),
                    RGB::from_f32(0., 0., 0.),
                    rltk::to_cp437('#'),
                );
            }
        }

        // Move the coordinates
        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
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
