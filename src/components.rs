use bevy_ecs::prelude::*;
use rltk::{FontCharType, Point, RGB};

#[derive(Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

pub(crate) trait AsPoint {
    fn as_point(&self) -> Point;
}

impl AsPoint for Position {
    fn as_point(&self) -> Point {
        Point {
            x: self.x,
            y: self.y,
        }
    }
}

#[derive(Component)]
pub struct Renderable {
    pub glyph: FontCharType,
    pub fg: RGB,
    pub bg: RGB,
}

#[derive(Component, Debug)]
pub struct Player {}

#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles: Vec<Point>,
    pub range: i32,
    pub dirty: bool,
}

#[derive(Component, Debug)]
pub struct Monster {}

#[derive(Component, Debug)]
pub struct Name {
    pub name: String,
}
