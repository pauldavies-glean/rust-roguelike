use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct GameLog {
    pub entries: Vec<String>,
}
