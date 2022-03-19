#![allow(clippy::complexity)]

use bevoids_assets::{SpriteAssetIo, AsteroidAssetIo, BackgroundAssetIo, SoundAssetIo};
use bevy::{
    log,
    prelude::*,
    render::camera::{DepthCalculation, OrthographicProjection, ScalingMode},
};
use bevy_effects::{animation::TextureAtlasMap, sound::SoundEffectSettings};
use bevy_embasset::*;

mod bevoids;
mod bounds;

use crate::bevoids::Bevoids;

use self::bevoids::settings::Settings;

fn main() {
    let settings: Settings = serde_json::from_slice(include_bytes!("settings.json"))
        .expect("unable to parse settings file");

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Msaa { samples: 4 })
        .init_resource::<TextureAtlasMap>() // TODO: auto add if not added by user
        .init_resource::<SoundEffectSettings>() // TODO: auto add if not added by user
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
        .add_embasset_plugin(|io| {
            io.add_handler(SpriteAssetIo::new().into());
            io.add_handler(AsteroidAssetIo::new().into());
            io.add_handler(BackgroundAssetIo::new().into());
            io.add_handler(SoundAssetIo::new().into());
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
