#![allow(clippy::complexity)]

// TODO: tests in bevy?

use bevy::{
    log,
    prelude::*,
    render::camera::{DepthCalculation, OrthographicProjection, ScalingMode},
};

mod asset_io;
mod bevoids;
//mod text;

use crate::bevoids::{Bevoids};

use self::bevoids::settings::Settings;

fn main() {
    let settings: Settings = serde_json::from_slice(include_bytes!("settings.json")).expect("unable to parse settings file");

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
            group.add_before::<bevy::asset::AssetPlugin, _>(asset_io::InMemoryAssetPlugin)
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
    commands.spawn().insert_bundle(new_camera_2d());
}

pub fn new_camera_2d() -> OrthographicCameraBundle {
    let far = 1000.0;
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.orthographic_projection = OrthographicProjection {
        far,
        depth_calculation: DepthCalculation::ZDifference,
        scaling_mode: ScalingMode::WindowSize,
        ..Default::default()
    };
    //camera.transform.scale = Vec3::new(400., 400., 1.);
    camera
}
