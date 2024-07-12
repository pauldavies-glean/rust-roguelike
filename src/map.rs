use bevy_ecs::prelude::*;
use rltk::{
    to_cp437, Algorithm2D, BaseMap, DistanceAlg, Point, RandomNumberGenerator, Rltk, SmallVec, RGB,
};
use serde::{Deserialize, Serialize};
use std::{
    cmp::{max, min},
    collections::HashSet,
};

use crate::{
    components::{BlocksTile, Position},
    rect::Rect,
};

#[derive(PartialEq, Serialize, Deserialize, Clone)]
pub enum TileType {
    Wall,
    Floor,
    DownStairs,
}

#[derive(Resource, Default, Serialize, Deserialize, Clone)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked: Vec<bool>,
    pub depth: i32,
    pub bloodstains: HashSet<usize>,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub tile_content: Vec<Vec<Entity>>,
}

pub const MAPWIDTH: usize = 80;
pub const MAPHEIGHT: usize = 43;
pub const MAPCOUNT: usize = MAPHEIGHT * MAPWIDTH;

impl Map {
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    pub fn contains_point(&self, p: Point) -> bool {
        p.x >= 0 && p.x < self.width && p.y >= 0 && p.y < self.height
    }

    fn apply_room_to_map(&mut self, room: &Rect) {
        for y in room.y1 + 1..=room.y2 {
            for x in room.x1 + 1..=room.x2 {
                let idx = self.xy_idx(x, y);
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2)..=max(x1, x2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < self.width as usize * self.height as usize {
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < self.width as usize * self.height as usize {
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    /// Makes a new map using the algorithm from http://rogueliketutorials.com/tutorials/tcod/part-3/
    /// This gives a handful of random rooms and corridors joining them together.
    pub fn new_map_rooms_and_corridors(new_depth: i32, rng: &mut RandomNumberGenerator) -> Map {
        let mut map = Map {
            tiles: vec![TileType::Wall; MAPCOUNT],
            rooms: Vec::new(),
            width: MAPWIDTH as i32,
            height: MAPHEIGHT as i32,
            revealed_tiles: vec![false; MAPCOUNT],
            visible_tiles: vec![false; MAPCOUNT],
            blocked: vec![false; MAPCOUNT],
            tile_content: vec![Vec::new(); MAPCOUNT],
            depth: new_depth,
            bloodstains: HashSet::new(),
        };

        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        for _i in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, map.width - w - 1) - 1;
            let y = rng.roll_dice(1, map.height - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in map.rooms.iter() {
                if new_room.intersect(other_room) {
                    ok = false
                }
            }
            if ok {
                map.apply_room_to_map(&new_room);

                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len() - 1].center();
                    if rng.range(0, 2) == 1 {
                        map.apply_horizontal_tunnel(prev_x, new_x, prev_y);
                        map.apply_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        map.apply_vertical_tunnel(prev_y, new_y, prev_x);
                        map.apply_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }

                map.rooms.push(new_room);
            }
        }

        let (stairs_x, stairs_y) = map.rooms[map.rooms.len() - 1].center();
        let stairs_idx = map.xy_idx(stairs_x, stairs_y);
        map.tiles[stairs_idx] = TileType::DownStairs;

        map
    }

    pub fn populate_blocked(&mut self) {
        for (i, tile) in self.tiles.iter_mut().enumerate() {
            self.blocked[i] = *tile == TileType::Wall;
        }
    }

    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 {
            return false;
        }
        let idx = self.xy_idx(x, y);
        !self.blocked[idx]
    }

    fn is_revealed_and_wall(&self, x: i32, y: i32) -> bool {
        let idx = self.xy_idx(x, y);
        self.tiles[idx] == TileType::Wall && self.revealed_tiles[idx]
    }

    fn wall_glyph(&self, x: i32, y: i32) -> rltk::FontCharType {
        if x < 1 || x > self.width - 2 || y < 1 || y > self.height - 2 as i32 {
            return 35;
        }
        let mut mask: u8 = 0;

        if self.is_revealed_and_wall(x, y - 1) {
            mask += 1;
        }
        if self.is_revealed_and_wall(x, y + 1) {
            mask += 2;
        }
        if self.is_revealed_and_wall(x - 1, y) {
            mask += 4;
        }
        if self.is_revealed_and_wall(x + 1, y) {
            mask += 8;
        }

        match mask {
            0 => 9,    // Pillar because we can't see neighbors
            1 => 186,  // Wall only to the north
            2 => 186,  // Wall only to the south
            3 => 186,  // Wall to the north and south
            4 => 205,  // Wall only to the west
            5 => 188,  // Wall to the north and west
            6 => 187,  // Wall to the south and west
            7 => 185,  // Wall to the north, south and west
            8 => 205,  // Wall only to the east
            9 => 200,  // Wall to the north and east
            10 => 201, // Wall to the south and east
            11 => 204, // Wall to the north, south and east
            12 => 205, // Wall to the east and west
            13 => 202, // Wall to the east, west, and south
            14 => 203, // Wall to the east, west, and north
            15 => 206, // ╬ Wall on all sides
            _ => 35,   // We missed one?
        }
    }

    pub fn draw(&self, ctx: &mut Rltk) {
        let mut y = 0;
        let mut x = 0;
        for (idx, tile) in self.tiles.iter().enumerate() {
            if self.revealed_tiles[idx] {
                let glyph;
                let mut fg;
                let mut bg = RGB::from_f32(0., 0., 0.);
                match tile {
                    TileType::Floor => {
                        glyph = to_cp437('.');
                        fg = RGB::from_f32(0.0, 0.5, 0.5);
                    }
                    TileType::Wall => {
                        glyph = self.wall_glyph(x, y);
                        fg = RGB::from_f32(0., 1.0, 0.);
                    }
                    TileType::DownStairs => {
                        glyph = to_cp437('>');
                        fg = RGB::from_f32(0., 1.0, 1.0);
                    }
                }
                if self.bloodstains.contains(&idx) {
                    bg = RGB::from_f32(0.75, 0., 0.);
                }
                if !self.visible_tiles[idx] {
                    fg = fg.to_greyscale()
                }
                ctx.set(x, y, fg, bg, glyph);
            }
            // Move the coordinates
            x += 1;
            if x >= self.width {
                x = 0;
                y += 1;
            }
        }
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx] == TileType::Wall
    }

    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let mut exits = SmallVec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width as usize;

        // Cardinal directions
        if self.is_exit_valid(x - 1, y) {
            exits.push((idx - 1, 1.0))
        };
        if self.is_exit_valid(x + 1, y) {
            exits.push((idx + 1, 1.0))
        };
        if self.is_exit_valid(x, y - 1) {
            exits.push((idx - w, 1.0))
        };
        if self.is_exit_valid(x, y + 1) {
            exits.push((idx + w, 1.0))
        };

        // Diagonals
        if self.is_exit_valid(x - 1, y - 1) {
            exits.push(((idx - w) - 1, 1.45));
        }
        if self.is_exit_valid(x + 1, y - 1) {
            exits.push(((idx - w) + 1, 1.45));
        }
        if self.is_exit_valid(x - 1, y + 1) {
            exits.push(((idx + w) - 1, 1.45));
        }
        if self.is_exit_valid(x + 1, y + 1) {
            exits.push(((idx + w) + 1, 1.45));
        }

        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

pub fn map_indexing_system(
    entities: Query<(Entity, &Position, Option<&BlocksTile>)>,
    mut map: ResMut<Map>,
) {
    map.populate_blocked();
    map.clear_content_index();
    for (entity, position, blocks) in entities.iter() {
        let idx = map.xy_idx(position.x, position.y);

        if blocks.is_some() {
            map.blocked[idx] = true;
        }

        map.tile_content[idx].push(entity);
    }
}
