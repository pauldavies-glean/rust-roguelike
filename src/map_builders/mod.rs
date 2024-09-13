mod bsp_dungeon;
mod bsp_interior;
mod cellular_automata;
mod common;
mod dla;
mod drunkard;
mod maze;
mod prefab_builder;
mod simple_map;
mod voronoi;
mod waveform_collapse;

use super::{rect::Rect, spawner, Map, Position, TileType, SHOW_MAPGEN_VISUALIZER};
use bevy_ecs::prelude::*;
use bsp_dungeon::BspDungeonBuilder;
use bsp_interior::BspInteriorBuilder;
use cellular_automata::CellularAutomataBuilder;
use common::*;
use dla::DLABuilder;
use drunkard::DrunkardsWalkBuilder;
use maze::MazeBuilder;
use prefab_builder::PrefabBuilder;
use simple_map::SimpleMapBuilder;
use voronoi::VoronoiCellBuilder;
use waveform_collapse::WaveformCollapseBuilder;

pub trait MapBuilder {
    fn build_map(&mut self);
    fn get_map(&self) -> Map;
    fn get_starting_position(&self) -> Position;
    fn get_snapshot_history(&self) -> Vec<Map>;
    fn take_snapshot(&mut self);
    fn get_spawn_list(&self) -> &Vec<(usize, String)>;

    fn spawn_entities(&mut self, ecs: &mut World) {
        for (spawn_idx, spawn_name) in self.get_spawn_list().iter() {
            spawner::spawn_entity(ecs, &(spawn_idx, spawn_name));
        }
    }
}

pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    let mut rng = rltk::RandomNumberGenerator::new();
    let builder = rng.roll_dice(1, 17);

    let mut result: Box<dyn MapBuilder> = match builder {
        1 => Box::new(BspDungeonBuilder::new(new_depth)),
        2 => Box::new(BspInteriorBuilder::new(new_depth)),
        3 => Box::new(CellularAutomataBuilder::new(new_depth)),
        4 => Box::new(DrunkardsWalkBuilder::open_area(new_depth)),
        5 => Box::new(DrunkardsWalkBuilder::open_halls(new_depth)),
        6 => Box::new(DrunkardsWalkBuilder::winding_passages(new_depth)),
        7 => Box::new(DrunkardsWalkBuilder::fat_passages(new_depth)),
        8 => Box::new(DrunkardsWalkBuilder::fearful_symmetry(new_depth)),
        9 => Box::new(MazeBuilder::new(new_depth)),
        10 => Box::new(DLABuilder::walk_inwards(new_depth)),
        11 => Box::new(DLABuilder::walk_outwards(new_depth)),
        12 => Box::new(DLABuilder::central_attractor(new_depth)),
        13 => Box::new(DLABuilder::insectoid(new_depth)),
        14 => Box::new(VoronoiCellBuilder::pythagoras(new_depth)),
        15 => Box::new(VoronoiCellBuilder::manhattan(new_depth)),
        16 => Box::new(PrefabBuilder::constant(
            new_depth,
            prefab_builder::prefab_levels::WFC_POPULATED,
        )),
        _ => Box::new(SimpleMapBuilder::new(new_depth)),
    };

    if rng.roll_dice(1, 3) == 1 {
        result = Box::new(WaveformCollapseBuilder::derived_map(new_depth, result));
    }

    if rng.roll_dice(1, 20) == 1 {
        result = Box::new(PrefabBuilder::sectional(
            new_depth,
            prefab_builder::prefab_sections::UNDERGROUND_FORT,
            result,
        ));
    }

    result = Box::new(PrefabBuilder::vaults(new_depth, result));

    result
}
