use bevy_ecs::{prelude::*, system::EntityCommands};
use rltk::{FontCharType, Point};
use serde::{Deserialize, Serialize};

#[derive(Clone, Component, Default, Serialize, Deserialize)]
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

#[derive(Clone, Component, Default, Serialize, Deserialize)]
pub struct Renderable {
    pub glyph: FontCharType,
    pub fg: (u8, u8, u8),
    pub bg: (u8, u8, u8),
    pub render_order: i32,
}

#[derive(Debug, Clone, Component, Default, Serialize, Deserialize)]
pub struct Player {}

#[derive(Clone, Component, Default, Serialize, Deserialize)]
pub struct Viewshed {
    pub visible_tiles: Vec<Point>,
    pub range: i32,
    pub dirty: bool,
}

#[derive(Debug, Clone, Component, Default, Serialize, Deserialize)]
pub struct Monster {}

#[derive(Debug, Clone, Component, Default, Serialize, Deserialize)]
pub struct Name {
    pub name: String,
}

#[derive(Clone, Component, Default, Serialize, Deserialize)]
pub struct BlocksTile {}

#[derive(Clone, Component, Default, Serialize, Deserialize)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}

#[derive(Component)]
pub struct WantsToMelee {
    pub target: Entity,
}

#[derive(Component)]
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

#[derive(Clone, Component, Default, Serialize, Deserialize)]
pub struct Item {}

#[derive(Clone, Component, Default, Serialize, Deserialize)]
pub struct ProvidesHealing {
    pub heal_amount: i32,
}

#[derive(Clone, Component, Serialize, Deserialize)]
pub struct InBackpack {
    pub owner: Entity,
}

#[derive(Component)]
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

#[derive(Clone, Component, Default, Serialize, Deserialize)]
pub struct Consumable {}

#[derive(Clone, Component, Default, Serialize, Deserialize)]
pub struct Ranged {
    pub range: i32,
}

#[derive(Clone, Component, Default, Serialize, Deserialize)]
pub struct InflictsDamage {
    pub damage: i32,
}

#[derive(Clone, Component, Default, Serialize, Deserialize)]
pub struct AreaOfEffect {
    pub radius: i32,
}

#[derive(Clone, Component, Default, Serialize, Deserialize)]
pub struct Confusion {
    pub turns: i32,
}

#[derive(Clone, Component, Default, Serialize, Deserialize)]
pub struct Confused {
    pub turns: i32,
}

#[derive(Clone, Component, Default, Serialize, Deserialize)]
pub struct Waiting {}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum EquipmentSlot {
    Melee,
    Shield,
}

#[derive(Clone, Component, Serialize, Deserialize)]
pub struct Equippable {
    pub slot: EquipmentSlot,
}

#[derive(Clone, Component, Serialize, Deserialize)]
pub struct Equipped {
    pub owner: Entity,
    pub slot: EquipmentSlot,
}

#[derive(Clone, Component, Serialize, Deserialize)]
pub struct MeleePowerBonus {
    pub power: i32,
}

#[derive(Clone, Component, Serialize, Deserialize)]
pub struct DefenseBonus {
    pub defense: i32,
}

#[derive(Clone, Component, Serialize, Deserialize)]
pub struct WantsToRemoveItem {
    pub item: Entity,
}

#[derive(Clone, Component, Serialize, Deserialize)]
pub struct ParticleLifetime {
    pub lifetime_ms: f32,
}
