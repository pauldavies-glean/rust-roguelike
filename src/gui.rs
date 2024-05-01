use bevy_ecs::prelude::*;
use rltk::{
    to_cp437, FontCharType, Point, Rltk, VirtualKeyCode, BLACK, GREY, MAGENTA, RED, WHITE, YELLOW,
};

use crate::{
    components::{CombatStats, InBackpack, Name, Player, Position},
    gamelog::GameLog,
    map::Map,
};

pub fn draw_ui(world: &mut World, ctx: &mut Rltk) {
    ctx.draw_box(0, 43, 79, 6, WHITE, BLACK);

    for stats in world
        .query_filtered::<&CombatStats, With<Player>>()
        .iter(world)
    {
        let health = format!(" HP: {} / {} ", stats.hp, stats.max_hp);
        ctx.print_color(12, 43, YELLOW, BLACK, &health);
        ctx.draw_bar_horizontal(28, 43, 51, stats.hp, stats.max_hp, RED, BLACK);
    }

    let log = world.resource::<GameLog>();
    let mut y = 44;
    for s in log.entries.iter().rev() {
        if y < 49 {
            ctx.print(2, y, s);
        }
        y += 1;
    }

    let named_entities = world.query::<(&Name, &Position)>().iter(world).collect();
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
        .filter(|item| item.0.owner == player_entity);
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y - 2, 31, (count + 3) as i32, WHITE, BLACK);
    ctx.print_color(18, y - 2, YELLOW, BLACK, "Inventory");
    ctx.print_color(18, y + count as i32 + 1, YELLOW, BLACK, "ESCAPE to cancel");

    let mut equippable: Vec<Entity> = Vec::new();
    let mut j = 0;
    for (_pack, name, entity) in held_items
        .iter(world)
        .filter(|item| item.0.owner == player_entity)
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
        .filter(|item| item.0.owner == player_entity);
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y - 2, 31, (count + 3) as i32, WHITE, BLACK);
    ctx.print_color(18, y - 2, YELLOW, BLACK, "Drop Which Item?");
    ctx.print_color(18, y + count as i32 + 1, YELLOW, BLACK, "ESCAPE to cancel");

    let mut droppable: Vec<Entity> = Vec::new();
    let mut j = 0;
    for (_pack, name, entity) in held_items
        .iter(world)
        .filter(|item| item.0.owner == player_entity)
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
