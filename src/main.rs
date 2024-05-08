mod ai;
mod combat;
mod components;
mod damage;
mod gamelog;
mod gui;
mod inventory;
mod map;
mod player;
mod rect;
mod spawner;
mod visibility;

use bevy_ecs::prelude::*;
use components::{Player, Position, Ranged, Renderable, WantsToDropItem, WantsToUseItem};
use gamelog::GameLog;
use map::{Map, MAPCOUNT};
use rltk::{
    main_loop, BError, GameState, RandomNumberGenerator, Rltk, RltkBuilder, VirtualKeyCode,
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
    ShowInventory,
    ShowDropItem,
    ShowTargeting { range: i32, item: Entity },
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        let state = *self.world.resource::<RunState>();
        let mut new_state = match state {
            RunState::PlayerTurn => RunState::MonsterTurn,
            RunState::ShowInventory => RunState::ShowInventory,
            RunState::ShowDropItem => RunState::ShowDropItem,
            RunState::ShowTargeting { range, item } => RunState::ShowTargeting { range, item },
            _ => RunState::AwaitingInput,
        };

        self.world.insert_non_send_resource::<Key>(ctx.key);
        self.schedule.run(&mut self.world);

        let mut things = self.world.query::<(&Position, &Renderable)>();

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

    world.insert_resource(RunState::PreRun);
    world.insert_resource(GameLog {
        entries: vec!["Welcome to Rusty Roguelike".to_string()],
    });

    let map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();
    spawner::player(&mut world, player_x, player_y);

    // don't put a monster where the player is!
    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut world, room);
    }
    world.insert_resource(map);

    let mut state = State {
        world,
        schedule: Schedule::default(),
    };

    state.schedule.add_systems(
        (
            inventory::item_use_system,
            inventory::drop_system,
            player::player_input_system,
            inventory::inventory_system,
            visibility::visibility_system,
            ai::monster_ai_system,
            combat::melee_combat_system,
            damage::damage_system,
            map::map_indexing_system,
        )
            .chain(),
    );

    main_loop(context, state)
}
