use bevy_ecs::prelude::*;
use rltk::rex::XpFile;

rltk::embedded_resource!(SMALL_DUNGEON, "../resources/rust.xp");
rltk::embedded_resource!(WFC_DEMO_IMAGE1, "../resources/wfc-demo1.xp");
rltk::embedded_resource!(WFC_POPULATED, "../resources/wfc-populated.xp");

#[derive(Resource)]
pub struct RexAssets {
    pub menu: XpFile,
}

impl RexAssets {
    pub fn new() -> RexAssets {
        rltk::link_resource!(SMALL_DUNGEON, "../resources/rust.xp");
        rltk::link_resource!(WFC_DEMO_IMAGE1, "../resources/wfc-demo1.xp");
        rltk::link_resource!(WFC_POPULATED, "../resources/wfc-populated.xp");

        RexAssets {
            menu: XpFile::from_resource("../resources/rust.xp").unwrap(),
        }
    }
}
