#![allow(clippy::complexity)]
use assets::AssetPath;
use bevy::{app::Events, log, prelude::*, sprite::SpriteSettings, window::WindowResized};
use bevy_kira_audio::*;
use derive_more::Display;
use std::path::PathBuf;
use structopt::StructOpt;

use crate::{
    assets::LoadRelative,
    bounds::Bounds,
    constants::{AUDIO_EXPLOSION_ASTEROID, AUDIO_EXPLOSION_SHIP, AUDIO_LASER, AUDIO_THRUSTER},
    plugins::{
        AsteroidPlugin, BulletPlugin, FadeDespawnPlugin, GameOverPlugin, HitTestPlugin,
        MovementPlugin, PlayerPlugin, ScoreBoardPlugin, TextureLoaderPlugin,
    },
};

mod assets;
mod bounds;
mod constants;
mod plugins;
mod text;

#[derive(Debug, StructOpt)]
pub(crate) struct Args {
    #[structopt(long)]
    assets: Option<String>,
}

impl AssetPath for Args {
    fn asset_path<T: AsRef<str>>(&self, path: &T) -> PathBuf {
        self.assets
            .as_ref()
            .map(|assets| {
                let mut p = PathBuf::from(assets);
                p.push(path.as_ref());
                let assets = p.display().to_string();
                log::info!(?assets);
                p
            })
            .unwrap_or_else(|| {
                let mut p = PathBuf::from(std::env::current_dir().expect("no current dir"));
                p.push("assets");
                p.push(path.as_ref());
                let assets = p.display().to_string();
                log::info!(?assets);
                p
            })
    }
}

#[derive(Debug, Display, Copy, Clone, Eq, PartialEq, Hash)]
pub(crate) enum GameState {
    Initialize,
    InGame,
    GameOver,
}

fn main() {
    App::new()
        // set the starting state & general systems
        .add_state(GameState::Initialize)
        .add_startup_system_to_stage(StartupStage::PreStartup, initialize.system())
        .add_system(resized.system())
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        // .add_plugin(bevy::diagnostic::LogDiagnosticsPlugin::default())
        // .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        //
        // game plugins
        .add_plugin(FadeDespawnPlugin)
        .add_plugin(MovementPlugin)
        .add_plugin(TextureLoaderPlugin)
        .add_plugin(ScoreBoardPlugin)
        .add_plugin(BulletPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(AsteroidPlugin)
        .add_plugin(HitTestPlugin)
        .add_plugin(GameOverPlugin)
        //
        // resources
        .insert_resource(Args::from_args())
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(WindowDescriptor {
            width: constants::WIN_WIDTH,
            height: constants::WIN_HEIGHT,
            ..WindowDescriptor::default()
        })
        .insert_resource(SpriteSettings {
            frustum_culling_enabled: true,
        })
        .run();
}

fn resized(resize_event: Res<Events<WindowResized>>, mut bounds: ResMut<Bounds>) {
    let mut reader = resize_event.get_reader();
    for e in reader.iter(&resize_event) {
        *bounds = Bounds::from_pos_and_size(Vec2::ZERO, Vec2::new(e.width, e.height));
    }
}

fn initialize(
    mut commands: Commands,
    mut windows: ResMut<Windows>,
    asset_server: Res<AssetServer>,
    args: Res<Args>,
) {
    log::info!("initializing game");
    let _ = asset_server.load_relative::<AudioSource, _, _>(&AUDIO_LASER, &*args);
    let _ = asset_server.load_relative::<AudioSource, _, _>(&AUDIO_THRUSTER, &*args);
    let _ = asset_server.load_relative::<AudioSource, _, _>(&AUDIO_EXPLOSION_SHIP, &*args);
    let _ = asset_server.load_relative::<AudioSource, _, _>(&AUDIO_EXPLOSION_ASTEROID, &*args);

    let window = windows.get_primary_mut().unwrap();
    window.set_resizable(true);
    window.set_vsync(true);
    window.set_title(module_path!().into());

    commands.insert_resource(Bounds::from_window(window));

    // Spawns the camera
    commands
        .spawn()
        .insert_bundle(OrthographicCameraBundle::new_2d())
        .insert(Transform::from_xyz(0.0, 0.0, 1000.0));
}
