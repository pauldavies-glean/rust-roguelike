mod ai;
mod combat;
mod components;
mod damage;
mod gamelog;
mod gui;
mod hunger;
mod inventory;
mod map;
mod map_builders;
mod particle;
mod player;
mod random_table;
mod rect;
mod rex_assets;
mod saveload;
mod spawner;
mod triggers;
mod visibility;

extern crate serde;

use bevy_ecs::prelude::*;
use components::{
    CombatStats, Equipped, Hidden, InBackpack, Player, Position, Ranged, Renderable, Viewshed,
    WantsToDropItem, WantsToRemoveItem, WantsToUseItem,
};
use damage::DamageEvent;
use gamelog::GameLog;
use map::{Map, MAPCOUNT, MAPHEIGHT, MAPWIDTH};
use rex_assets::RexAssets;
use rltk::{
    main_loop, BError, GameState, RandomNumberGenerator, Rltk, RltkBuilder, VirtualKeyCode,
};

const SHOW_MAPGEN_VISUALIZER: bool = false;

struct State {
    world: World,
    schedule: Schedule,
    mapgen_history: Vec<Map>,
    mapgen_index: usize,
    mapgen_timer: f32,
}

impl State {
    fn entities_to_remove_on_level_change(&mut self) -> Vec<Entity> {
        let mut entities = self.world.query::<Entity>();
        let player_entity = self
            .world
            .query_filtered::<Entity, With<Player>>()
            .single(&self.world);
        let mut backpack = self.world.query::<&InBackpack>();
        let mut equipped = self.world.query::<&Equipped>();

        let mut to_delete: Vec<Entity> = Vec::new();
        for entity in entities.iter(&self.world) {
            let mut should_delete = true;

            // Don't delete the player
            if entity == player_entity {
                should_delete = false;
            }

            // Don't delete the player's equipment
            let bp = backpack.get(&self.world, entity);
            if let Ok(bp) = bp {
                if bp.owner == player_entity {
                    should_delete = false;
                }
            }

            let eq = equipped.get(&self.world, entity);
            if let Ok(eq) = eq {
                if eq.owner == player_entity {
                    should_delete = false;
                }
            }

            if should_delete {
                to_delete.push(entity);
            }
        }

        to_delete
    }

    fn init_game(&mut self) {
        let start = self.create_map(1);
        spawner::player(&mut self.world, start.x, start.y);
    }

    fn create_map(&mut self, new_depth: i32) -> Position {
        self.mapgen_history.clear();
        self.mapgen_index = 0;
        self.mapgen_timer = 0.0;

        let mut builder = map_builders::random_builder(new_depth);
        builder.build_map();
        self.mapgen_history = builder.get_snapshot_history();

        builder.spawn_entities(&mut self.world);

        self.world.insert_resource(builder.get_map()); // TODO this is stupid

        builder.get_starting_position()
    }

    fn goto_next_level(&mut self) {
        // Delete entities that aren't the player or his/her equipment
        let to_delete = self.entities_to_remove_on_level_change();
        for target in to_delete {
            self.world.despawn(target);
        }

        // Build a new map and place the player
        let old_map = self.world.resource::<Map>();
        let current_depth = old_map.depth;
        let start = self.create_map(current_depth + 1);

        // Find the player
        let (mut player_position, mut player_viewshed, mut player_stats) = self
            .world
            .query_filtered::<(&mut Position, &mut Viewshed, &mut CombatStats), With<Player>>()
            .single_mut(&mut self.world);

        // Place the player and update resources
        player_position.x = start.x;
        player_position.y = start.y;

        // Mark the player's visibility as dirty
        player_viewshed.dirty = true;

        // Let them recover a bit
        player_stats.hp = i32::max(player_stats.hp, player_stats.max_hp / 2);

        // Notify the player
        let mut log = self.world.resource_mut::<GameLog>();
        log.entries
            .push("You descend to the next level, and take a moment to heal.".to_string());
    }

    fn game_over_cleanup(&mut self) {
        self.world.clear_entities();
    }

    fn draw_to_screen(&mut self, ctx: &mut Rltk) {
        let mut things = self
            .world
            .query_filtered::<(&Position, &Renderable), Without<Hidden>>();

        let map = self.world.resource::<Map>();
        map.draw(ctx);

        let mut priority = vec![100; MAPCOUNT];
        for (pos, render) in things.iter(&self.world) {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] && priority[idx] > render.render_order {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
                priority[idx] = render.render_order
            }
        }

        gui::draw_ui(&mut self.world, ctx);
    }
}

pub type Key = Option<VirtualKeyCode>;
pub type FrameTime = f32;

#[derive(PartialEq, Copy, Clone, Resource)]
pub enum RunState {
    AwaitingInput,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem,
    ShowTargeting {
        range: i32,
        item: Entity,
    },
    MainMenu {
        menu_selection: gui::MainMenuSelection,
    },
    SaveGame,
    NextLevel,
    ShowRemoveItem,
    GameOver,
    MagicMapReveal {
        row: i32,
    },
    MapGeneration,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        let state = *self.world.resource::<RunState>();
        let mut new_state = match state {
            RunState::PlayerTurn => RunState::MonsterTurn,
            RunState::ShowInventory => state,
            RunState::ShowDropItem => state,
            RunState::ShowTargeting { range: _, item: _ } => state,
            RunState::MainMenu { menu_selection: _ } => state,
            RunState::SaveGame => RunState::MainMenu {
                menu_selection: gui::MainMenuSelection::LoadGame,
            },
            RunState::ShowRemoveItem => state,
            RunState::GameOver => state,
            RunState::MapGeneration => state,
            RunState::NextLevel => RunState::MapGeneration,
            RunState::MagicMapReveal { row } => RunState::MagicMapReveal { row: row + 1 },
            _ => RunState::AwaitingInput,
        };

        match state {
            RunState::MainMenu { menu_selection } => {
                let result =
                    gui::main_menu(menu_selection, ctx, self.world.resource::<RexAssets>());
                match result {
                    gui::MainMenuResult::NoSelection { selected } => {
                        new_state = RunState::MainMenu {
                            menu_selection: selected,
                        }
                    }
                    gui::MainMenuResult::Selected { selected } => match selected {
                        gui::MainMenuSelection::NewGame => {
                            self.world.insert_resource(GameLog {
                                entries: vec!["Welcome to Rusty Roguelike".to_string()],
                            });
                            self.init_game();
                            new_state = RunState::MapGeneration;
                        }
                        gui::MainMenuSelection::LoadGame => {
                            saveload::load_game(&mut self.world);
                            self.world.insert_resource(GameLog {
                                entries: vec!["Welcome back to Rusty Roguelike".to_string()],
                            });
                            new_state = RunState::PlayerTurn;
                            saveload::delete_save();
                        }
                        gui::MainMenuSelection::Quit => {
                            ::std::process::exit(0);
                        }
                    },
                }
            }

            RunState::SaveGame => {
                saveload::save_game(&mut self.world).unwrap();
                self.world.clear_entities();
            }

            RunState::NextLevel => {
                self.goto_next_level();
            }

            RunState::GameOver => {
                let result = gui::game_over(ctx);
                match result {
                    gui::GameOverResult::NoSelection => {}
                    gui::GameOverResult::QuitToMenu => {
                        self.game_over_cleanup();
                        new_state = RunState::MainMenu {
                            menu_selection: gui::MainMenuSelection::NewGame,
                        }
                    }
                }
            }

            RunState::MagicMapReveal { row } => {
                let mut map = self.world.resource_mut::<Map>();
                for x in 0..MAPWIDTH {
                    let idx = map.xy_idx(x as i32, row);
                    map.revealed_tiles[idx] = true;
                }

                if row as usize >= MAPHEIGHT - 1 {
                    new_state = RunState::MonsterTurn;
                }

                self.draw_to_screen(ctx);
            }

            RunState::MapGeneration => {
                if !SHOW_MAPGEN_VISUALIZER {
                    new_state = RunState::AwaitingInput;
                } else {
                    ctx.cls();
                    self.mapgen_history[self.mapgen_index].draw(ctx);

                    self.mapgen_timer += ctx.frame_time_ms;
                    if self.mapgen_timer > 300.0 {
                        self.mapgen_timer = 0.0;
                        self.mapgen_index += 1;
                        if self.mapgen_index >= self.mapgen_history.len() {
                            new_state = RunState::AwaitingInput;
                        }
                    }
                }
            }

            _ => {
                self.world.insert_non_send_resource::<Key>(ctx.key);
                self.world
                    .insert_non_send_resource::<FrameTime>(ctx.frame_time_ms);
                self.schedule.run(&mut self.world);

                self.draw_to_screen(ctx);
            }
        }

        match state {
            RunState::ShowInventory => {
                let (result, item) = gui::show_inventory(&mut self.world, ctx);
                match result {
                    gui::ItemMenuResult::Cancel => new_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let player_entity = self
                            .world
                            .query_filtered::<Entity, With<Player>>()
                            .single(&self.world);

                        let item = item.unwrap();

                        if let Some(targeting) = self.world.get::<Ranged>(item) {
                            new_state = RunState::ShowTargeting {
                                range: targeting.range,
                                item,
                            }
                        } else {
                            self.world
                                .entity_mut(player_entity)
                                .insert(WantsToUseItem { item, target: None });
                            new_state = RunState::PlayerTurn;
                        }
                    }
                }
            }
            RunState::ShowDropItem => {
                let (result, item) = gui::drop_menu_item(&mut self.world, ctx);
                match result {
                    gui::ItemMenuResult::Cancel => new_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let player_entity = self
                            .world
                            .query_filtered::<Entity, With<Player>>()
                            .single(&self.world);

                        self.world
                            .entity_mut(player_entity)
                            .insert(WantsToDropItem {
                                item: item.unwrap(),
                            });

                        new_state = RunState::PlayerTurn;
                    }
                }
            }
            RunState::ShowTargeting { range, item } => {
                let (result, target) = gui::ranged_target(&mut self.world, ctx, range);
                match result {
                    gui::ItemMenuResult::Cancel => new_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let player_entity = self
                            .world
                            .query_filtered::<Entity, With<Player>>()
                            .single(&self.world);

                        self.world
                            .entity_mut(player_entity)
                            .insert(WantsToUseItem { item, target });

                        new_state = RunState::PlayerTurn;
                    }
                }
            }
            RunState::ShowRemoveItem => {
                let (result, item) = gui::remove_item_menu(&mut self.world, ctx);
                match result {
                    gui::ItemMenuResult::Cancel => new_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let player_entity = self
                            .world
                            .query_filtered::<Entity, With<Player>>()
                            .single(&self.world);

                        self.world
                            .entity_mut(player_entity)
                            .insert(WantsToRemoveItem {
                                item: item.unwrap(),
                            });

                        new_state = RunState::PlayerTurn;
                    }
                }
            }
            _ => {}
        }

        if *self.world.resource::<RunState>() == state {
            self.world.insert_resource(new_state);
        }
    }
}

fn main() -> BError {
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;

    let mut world = World::new();

    let rng = RandomNumberGenerator::new();
    world.insert_non_send_resource(rng);

    world.insert_resource(RunState::MainMenu {
        menu_selection: gui::MainMenuSelection::NewGame,
    });
    world.insert_resource(particle::ParticleBuilder::new());
    world.insert_resource(RexAssets::new());

    world.insert_resource(Events::<DamageEvent>::default());

    let mut state = State {
        world,
        schedule: Schedule::default(),
        mapgen_index: 0,
        mapgen_history: Vec::new(),
        mapgen_timer: 0.0,
    };

    state.schedule.add_systems(
        (
            inventory::item_use_system,
            inventory::drop_system,
            inventory::item_remove_system,
            hunger::hunger_system,
            player::player_input_system,
            inventory::inventory_system,
            visibility::visibility_system,
            player::waiting_system,
            ai::monster_ai_system,
            triggers::trigger_system,
            combat::melee_combat_system,
            damage::damage_event_reader,
            damage::damage_system,
            map::map_indexing_system,
            particle::cull_dead_particles_system,
            particle::spawn_particles_system,
        )
            .chain(),
    );

    main_loop(context, state)
}
