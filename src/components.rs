use bevy_ecs::{prelude::*, system::EntityCommands};
use rltk::{FontCharType, Point};

#[derive(Component, Default)]
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
    pub fg: (u8, u8, u8),
    pub bg: (u8, u8, u8),
    pub render_order: i32,
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

#[derive(Component, Debug)]
pub struct BlocksTile {}

#[derive(Component, Debug)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}

#[derive(Component, Debug, Clone)]
pub struct WantsToMelee {
    pub target: Entity,
}

#[derive(Component, Debug)]
pub struct SufferDamage {
    pub amount: Vec<i32>,
}

impl SufferDamage {
    pub fn new_damage(
        mut commands: EntityCommands,
        victim_suffering: Option<&mut SufferDamage>,
        value: i32,
    ) {
        if let Some(suffering) = victim_suffering {
            suffering.amount.push(value);
        } else {
            commands.insert(SufferDamage {
                amount: vec![value],
            });
        }
    }
}

#[derive(Component, Debug)]
pub struct Item {}

#[derive(Component, Debug)]
pub struct ProvidesHealing {
    pub heal_amount: i32,
}

#[derive(Component, Debug, Clone)]
pub struct InBackpack {
    pub owner: Entity,
}

#[derive(Component, Debug, Clone)]
pub struct WantsToPickupItem {
    pub collected_by: Entity,
    pub item: Entity,
}

#[derive(Component, Debug)]
pub struct WantsToUseItem {
    pub item: Entity,
    pub target: Option<Point>,
}

#[derive(Component, Debug, Clone)]
pub struct WantsToDropItem {
    pub item: Entity,
}

#[derive(Component, Debug)]
pub struct Consumable {}

#[derive(Component, Debug)]
pub struct Ranged {
    pub range: i32,
}

#[derive(Component, Debug)]
pub struct InflictsDamage {
    pub damage: i32,
}

#[derive(Component, Debug)]
pub struct AreaOfEffect {
    pub radius: i32,
}

#[derive(Component, Debug)]
pub struct Confusion {
    pub turns: i32,
}

#[derive(Component, Debug)]
pub struct Confused {
    pub turns: i32,
}
