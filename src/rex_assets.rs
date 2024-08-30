use bevy_ecs::prelude::*;
use rltk::rex::XpFile;

rltk::embedded_resource!(SMALL_DUNGEON, "../resources/rust.xp");

#[derive(Resource)]
pub struct RexAssets {
    pub menu: XpFile,
}

impl RexAssets {
    pub fn new() -> RexAssets {
        rltk::link_resource!(SMALL_DUNGEON, "../resources/rust.xp");

        RexAssets {
            menu: XpFile::from_resource("../resources/rust.xp").unwrap(),
        }
    }
}
