#![allow(clippy::complexity)]

// TODO: menu state: display menu before starting the game
// TODO: display high-score
// TODO: save high-score
// TODO: sound on highscore
// TODO: tests in bevy?

use bevy::{log, prelude::*};
use std::path::PathBuf;
use structopt::StructOpt;

mod bevoids;
mod text;

use crate::bevoids::{settings::Settings, AssetPath, Bevoids};

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(long)]
    assets: Option<String>,
}

fn main() {
    let args = Args::from_args();
    let assets_path = args.assets.unwrap_or_else(|| {
        let mut pb = PathBuf::from(std::env::current_dir().unwrap());
        pb.push("assets");
        pb.display().to_string()
    });
    let settings: Settings = toml::from_str(&{
        let mut pb = PathBuf::from(&assets_path);
        pb.push("settings.toml");
        std::fs::read_to_string(pb).expect("unable to read settings")
    })
    .expect("unable to parse settings file");

    App::build()
        .insert_resource(WindowDescriptor {
            vsync: true,
            resizable: false,
            width: settings.window.width as f32,
            height: settings.window.height as f32,
            title: module_path!().into(),
            ..WindowDescriptor::default()
        })
        .insert_resource(AssetPath::from(assets_path))
        .insert_resource(settings)
        .add_plugins(DefaultPlugins)
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
