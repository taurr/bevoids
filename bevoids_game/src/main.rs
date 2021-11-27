#![allow(clippy::complexity)]

// TODO: countdown when starting
// TODO: tests in bevy?

use bevy::{log, prelude::*};
use bevy_asset_map::EmbeddedAssetPlugin;

mod bevoids;
//mod text;

use crate::bevoids::{Bevoids, GameAssets};

fn main() {
    let settings = GameAssets::get_settings();

    App::build()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            vsync: true,
            resizable: false,
            width: settings.window.width as f32,
            height: settings.window.height as f32,
            title: module_path!().into(),
            ..WindowDescriptor::default()
        })
        .insert_resource(settings)
        //
        .add_plugins_with(DefaultPlugins, |group| {
            group.add_before::<bevy::asset::AssetPlugin, _>(
                EmbeddedAssetPlugin::<GameAssets>::default(),
            )
        })
        .add_startup_system(initialize_camera.system())
        //
        .add_plugin(Bevoids::default())
        //
        .run();
}

fn initialize_camera(mut commands: Commands) {
    log::info!("initializing game");
    // Spawns the camera
    commands
        .spawn()
        .insert_bundle(OrthographicCameraBundle::new_2d())
        .insert(Transform::from_xyz(0.0, 0.0, 1000.0));
}
