use bevy::prelude::*;
use bevy_mod_picking::{PickingCamera, PickingPlugin};
use board::BoardPlugin;
use camera::CameraPlugin;
use pieces::PiecesPlugin;
use ui::UIPlugin;

mod board;
mod camera;
mod pieces;
mod ui;

fn main() {
    App::build()
        // Set antialiasing to use 4 samples
        .insert_resource(Msaa { samples: 4 })
        // Set WindowDescriptor Resource to change title and size
        .insert_resource(WindowDescriptor {
            title: "Chess!".to_string(),
            width: 600.,
            height: 600.,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .init_resource::<PickingCamera>()
        .add_plugin(PickingPlugin)
        .add_plugin(BoardPlugin)
        .add_plugin(PiecesPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(UIPlugin)
        .run();
}
