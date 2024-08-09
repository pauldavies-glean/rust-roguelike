mod common;
mod simple_map;

use crate::{components::Position, map::Map};
use bevy_ecs::prelude::*;
use simple_map::SimpleMapBuilder;

pub trait MapBuilder {
    fn build_map(&mut self);
    fn spawn_entities(&mut self, ecs: &mut World);
    fn get_map(&mut self) -> Map;
    fn get_starting_position(&mut self) -> Position;
}

pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    // some day we will have another type
    Box::new(SimpleMapBuilder::new(new_depth))
}
