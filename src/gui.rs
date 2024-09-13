use bevy_ecs::prelude::*;
use rltk::{
    to_cp437, DistanceAlg, FontCharType, Point, Rltk, VirtualKeyCode, BLACK, BLUE, CYAN, GREEN,
    GREY, MAGENTA, ORANGE, RED, WHEAT, WHITE, YELLOW,
};

use crate::{
    components::{
        AsPoint, CombatStats, Equipped, Hidden, HungerClock, HungerState, InBackpack, Name, Player,
        Position, Viewshed,
    },
    gamelog::GameLog,
    map::Map,
    rex_assets::RexAssets,
    saveload,
};

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuSelection {
    NewGame,
    LoadGame,
    Quit,
}

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuResult {
    NoSelection { selected: MainMenuSelection },
    Selected { selected: MainMenuSelection },
}

pub fn draw_ui(world: &mut World, ctx: &mut Rltk) {
    ctx.draw_box(0, 43, 79, 6, WHITE, BLACK);

    {
        let map = world.resource::<Map>();
        ctx.print_color(2, 43, YELLOW, BLACK, format!("Depth: {}", map.depth));
    }

    for (stats, hunger) in world
        .query_filtered::<(&CombatStats, &HungerClock), With<Player>>()
        .iter(world)
    {
        let health = format!(" HP: {} / {} ", stats.hp, stats.max_hp);
        ctx.print_color(12, 43, YELLOW, BLACK, &health);
        ctx.draw_bar_horizontal(28, 43, 51, stats.hp, stats.max_hp, RED, BLACK);

        match hunger.state {
            HungerState::WellFed => ctx.print_color(71, 42, GREEN, BLACK, "Well Fed"),
            HungerState::Normal => {}
            HungerState::Hungry => ctx.print_color(71, 42, ORANGE, BLACK, "Hungry"),
            HungerState::Starving => ctx.print_color(71, 42, RED, BLACK, "Starving"),
        }
    }

    let log = world.resource::<GameLog>();
    let mut y = 44;
    for s in log.entries.iter().rev() {
        if y < 49 {
            ctx.print(2, y, s);
        }
        y += 1;
    }

    let named_entities = world
        .query_filtered::<(&Name, &Position), Without<Hidden>>()
        .iter(world)
        .collect();
    draw_tooltips(world, ctx, named_entities);
}

const SPACE: &str = " ";
const LEFT_ARROW: &str = "<-";
const RIGHT_ARROW: &str = "->";

fn draw_tooltips(world: &World, ctx: &mut Rltk, named_entities: Vec<(&Name, &Position)>) {
    let map = world.resource::<Map>();

    let (mouse_x, mouse_y) = ctx.mouse_pos();
    if mouse_x >= map.width || mouse_y >= map.height {
        return;
    }

    ctx.set_bg(mouse_x, mouse_y, MAGENTA);

    let mut tooltip: Vec<String> = Vec::new();
    for (name, position) in named_entities {
        let idx = map.xy_idx(position.x, position.y);
        if position.x == mouse_x && position.y == mouse_y && map.visible_tiles[idx] {
            tooltip.push(name.name.to_string());
        }
    }

    if !tooltip.is_empty() {
        let mut width: i32 = 0;
        for s in tooltip.iter() {
            if width < s.len() as i32 {
                width = s.len() as i32;
            }
        }
        width += 3;

        if mouse_x > 40 {
            let arrow_pos = Point::new(mouse_x - 2, mouse_y);
            let left_x = mouse_x - width;
            let mut y = mouse_y;

            for s in tooltip.iter() {
                ctx.print_color(left_x, y, WHITE, GREY, s);
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(arrow_pos.x - i, y, WHITE, GREY, SPACE);
                }
                y += 1;
            }
            ctx.print_color(arrow_pos.x, arrow_pos.y, WHITE, GREY, RIGHT_ARROW);
        } else {
            let arrow_pos = Point::new(mouse_x + 1, mouse_y);
            let left_x = mouse_x + 3;
            let mut y = mouse_y;
            for s in tooltip.iter() {
                ctx.print_color(left_x + 1, y, WHITE, GREY, s);
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(arrow_pos.x + 1 + i, y, WHITE, GREY, SPACE);
                }
                y += 1;
            }
            ctx.print_color(arrow_pos.x, arrow_pos.y, WHITE, GREY, LEFT_ARROW);
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected,
}

pub fn show_inventory(world: &mut World, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = world.query_filtered::<Entity, With<Player>>().single(world);
    let mut held_items = world.query::<(&InBackpack, &Name, Entity)>();

    let inventory = held_items
        .iter(world)
        .filter(|(pack, _, _)| pack.owner == player_entity);
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y - 2, 31, (count + 3) as i32, WHITE, BLACK);
    ctx.print_color(18, y - 2, YELLOW, BLACK, "Inventory");
    ctx.print_color(18, y + count as i32 + 1, YELLOW, BLACK, "ESCAPE to cancel");

    let mut equippable: Vec<Entity> = Vec::new();
    let mut j = 0;
    for (_pack, name, entity) in held_items
        .iter(world)
        .filter(|(pack, _, _)| pack.owner == player_entity)
    {
        ctx.set(17, y, WHITE, BLACK, to_cp437('('));
        ctx.set(18, y, YELLOW, BLACK, 97 + j as FontCharType);
        ctx.set(19, y, WHITE, BLACK, to_cp437(')'));

        ctx.print(21, y, &name.name.to_string());
        equippable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return (
                        ItemMenuResult::Selected,
                        Some(equippable[selection as usize]),
                    );
                }
                (ItemMenuResult::NoResponse, None)
            }
        },
    }
}

pub fn drop_menu_item(world: &mut World, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = world.query_filtered::<Entity, With<Player>>().single(world);
    let mut held_items = world.query::<(&InBackpack, &Name, Entity)>();

    let inventory = held_items
        .iter(world)
        .filter(|(pack, _, _)| pack.owner == player_entity);
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y - 2, 31, (count + 3) as i32, WHITE, BLACK);
    ctx.print_color(18, y - 2, YELLOW, BLACK, "Drop Which Item?");
    ctx.print_color(18, y + count as i32 + 1, YELLOW, BLACK, "ESCAPE to cancel");

    let mut droppable: Vec<Entity> = Vec::new();
    let mut j = 0;
    for (_pack, name, entity) in held_items
        .iter(world)
        .filter(|(pack, _, _)| pack.owner == player_entity)
    {
        ctx.set(17, y, WHITE, BLACK, to_cp437('('));
        ctx.set(18, y, YELLOW, BLACK, 97 + j as FontCharType);
        ctx.set(19, y, WHITE, BLACK, to_cp437(')'));

        ctx.print(21, y, &name.name.to_string());
        droppable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return (
                        ItemMenuResult::Selected,
                        Some(droppable[selection as usize]),
                    );
                }
                (ItemMenuResult::NoResponse, None)
            }
        },
    }
}

pub fn ranged_target(
    world: &mut World,
    ctx: &mut Rltk,
    range: i32,
) -> (ItemMenuResult, Option<Point>) {
    let (visible, player_pos) = world
        .query_filtered::<(&Viewshed, &Position), With<Player>>()
        .single(world);

    ctx.print_color(5, 0, YELLOW, BLACK, "Select Target:");

    // Highlight available target cells
    let mut available_cells = Vec::new();
    for idx in visible.visible_tiles.iter() {
        let distance = DistanceAlg::Pythagoras.distance2d(player_pos.as_point(), *idx);
        if distance <= range as f32 {
            ctx.set_bg(idx.x, idx.y, BLUE);
            available_cells.push(idx);
        }
    }

    // Draw mouse cursor
    let (mouse_x, mouse_y) = ctx.mouse_pos();
    let mut valid_target = false;
    for p in available_cells.iter() {
        if p.x == mouse_x && p.y == mouse_y {
            valid_target = true;
        }
    }
    if valid_target {
        ctx.set_bg(mouse_x, mouse_y, CYAN);
        if ctx.left_click {
            return (ItemMenuResult::Selected, Some(Point::new(mouse_x, mouse_y)));
        }
    } else {
        ctx.set_bg(mouse_x, mouse_y, RED);
        if ctx.left_click {
            return (ItemMenuResult::Cancel, None);
        }
    }

    (ItemMenuResult::NoResponse, None)
}

pub fn main_menu(
    selection: MainMenuSelection,
    ctx: &mut Rltk,
    assets: &RexAssets,
) -> MainMenuResult {
    let save_exists = saveload::does_save_exist();

    ctx.render_xp_sprite(&assets.menu, 0, 0);

    ctx.draw_box_double(24, 18, 31, 10, WHEAT, BLACK);

    ctx.print_color_centered(20, rltk::YELLOW, rltk::BLACK, "Rust Roguelike Tutorial");
    ctx.print_color_centered(21, rltk::CYAN, rltk::BLACK, "by Lag.Com");
    ctx.print_color_centered(22, rltk::GREY, rltk::BLACK, "Use Up/Down Arrows and Enter");

    let mut y = 24;

    if selection == MainMenuSelection::NewGame {
        ctx.print_color_centered(y, rltk::MAGENTA, rltk::BLACK, "Begin New Game");
    } else {
        ctx.print_color_centered(y, rltk::WHITE, rltk::BLACK, "Begin New Game");
    }
    y += 1;

    if save_exists {
        if selection == MainMenuSelection::LoadGame {
            ctx.print_color_centered(y, rltk::MAGENTA, rltk::BLACK, "Load Game");
        } else {
            ctx.print_color_centered(y, rltk::WHITE, rltk::BLACK, "Load Game");
        }
        y += 1;
    }

    if selection == MainMenuSelection::Quit {
        ctx.print_color_centered(y, rltk::MAGENTA, rltk::BLACK, "Quit");
    } else {
        ctx.print_color_centered(y, rltk::WHITE, rltk::BLACK, "Quit");
    }

    match ctx.key {
        None => {
            return MainMenuResult::NoSelection {
                selected: selection,
            }
        }
        Some(key) => match key {
            VirtualKeyCode::Escape => {
                return MainMenuResult::NoSelection {
                    selected: MainMenuSelection::Quit,
                }
            }
            VirtualKeyCode::Up => {
                let mut new_selection;
                match selection {
                    MainMenuSelection::NewGame => new_selection = MainMenuSelection::Quit,
                    MainMenuSelection::LoadGame => new_selection = MainMenuSelection::NewGame,
                    MainMenuSelection::Quit => new_selection = MainMenuSelection::LoadGame,
                }
                if new_selection == MainMenuSelection::LoadGame && !save_exists {
                    new_selection = MainMenuSelection::NewGame;
                }

                return MainMenuResult::NoSelection {
                    selected: new_selection,
                };
            }
            VirtualKeyCode::Down => {
                let mut new_selection;
                match selection {
                    MainMenuSelection::NewGame => new_selection = MainMenuSelection::LoadGame,
                    MainMenuSelection::LoadGame => new_selection = MainMenuSelection::Quit,
                    MainMenuSelection::Quit => new_selection = MainMenuSelection::NewGame,
                }
                if new_selection == MainMenuSelection::LoadGame && !save_exists {
                    new_selection = MainMenuSelection::Quit;
                }

                return MainMenuResult::NoSelection {
                    selected: new_selection,
                };
            }
            VirtualKeyCode::Return => {
                return MainMenuResult::Selected {
                    selected: selection,
                }
            }
            _ => {
                return MainMenuResult::NoSelection {
                    selected: selection,
                }
            }
        },
    }
}

pub fn remove_item_menu(world: &mut World, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = world.query_filtered::<Entity, With<Player>>().single(world);
    let mut equipped_items = world.query::<(&Equipped, &Name, Entity)>();

    let inventory = equipped_items
        .iter(world)
        .filter(|(eq, _, _)| eq.owner == player_entity);
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y - 2, 31, (count + 3) as i32, WHITE, BLACK);
    ctx.print_color(18, y - 2, YELLOW, BLACK, "Remove Which Item?");
    ctx.print_color(18, y + count as i32 + 1, YELLOW, BLACK, "ESCAPE to cancel");

    let mut removable: Vec<Entity> = Vec::new();
    let mut j = 0;
    for (_pack, name, entity) in equipped_items
        .iter(world)
        .filter(|(eq, _, _)| eq.owner == player_entity)
    {
        ctx.set(17, y, WHITE, BLACK, to_cp437('('));
        ctx.set(18, y, YELLOW, BLACK, 97 + j as FontCharType);
        ctx.set(19, y, WHITE, BLACK, to_cp437(')'));

        ctx.print(21, y, &name.name.to_string());
        removable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return (
                        ItemMenuResult::Selected,
                        Some(removable[selection as usize]),
                    );
                }
                (ItemMenuResult::NoResponse, None)
            }
        },
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum GameOverResult {
    NoSelection,
    QuitToMenu,
}

pub fn game_over(ctx: &mut Rltk) -> GameOverResult {
    ctx.print_color_centered(15, YELLOW, BLACK, "Your journey has ended!");
    ctx.print_color_centered(
        17,
        WHITE,
        BLACK,
        "One day, we'll tell you all about how you did.",
    );
    ctx.print_color_centered(
        18,
        WHITE,
        BLACK,
        "That day, sadly, is not in this chapter...",
    );

    ctx.print_color_centered(20, MAGENTA, BLACK, "Press any key to return to the menu.");

    match ctx.key {
        None => GameOverResult::NoSelection,
        Some(_) => GameOverResult::QuitToMenu,
    }
}
