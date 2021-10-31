#![allow(clippy::complexity)]
use assets::AssetPath;
use bevy::{log, prelude::*, sprite::SpriteSettings};
use bevy_kira_audio::*;
use bullet_plugin::BulletPlugin;
use derive_more::Display;
use gameover_plugin::GameoverPlugin;
use scoreboard::ScoreBoardPlugin;
use std::path::PathBuf;
use structopt::StructOpt;
use textures::TextureLoaderPlugin;

use crate::{
    assets::LoadRelative, asteroid_plugin::AsteroidPlugin, bounds::Bounds,
    fade_despawn_plugin::FadeDespawnPlugin, hit_test::HitTestPlugin,
    movement_plugin::MovementPlugin, player_plugin::PlayerPlugin,
};

mod assets;
mod asteroid_plugin;
mod bounds;
mod bullet_plugin;
mod constants;
mod fade_despawn_plugin;
mod gameover_plugin;
mod hit_test;
mod movement_plugin;
mod player_plugin;
mod scoreboard;
mod text;
mod textures;

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
                p
            })
            .unwrap_or_else(|| {
                let mut p = PathBuf::from(std::env::current_dir().expect("no current dir")); //from("assets");
                p.push("assets");
                p.push(path.as_ref());
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
        .add_plugin(GameoverPlugin)
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

fn initialize(
    mut commands: Commands,
    mut windows: ResMut<Windows>,
    asset_server: Res<AssetServer>,
    args: Res<Args>,
) {
    log::info!("initializing game");
    let _assets = asset_server
        .load_relative_folder(&"sounds", &*args)
        .expect("missing sounds");

    let window = windows.get_primary_mut().unwrap();
    window.set_resizable(false);
    window.set_vsync(true);
    window.set_title(module_path!().into());

    commands.insert_resource(Bounds::from_window(window));

    // Spawns the camera
    commands
        .spawn()
        .insert_bundle(OrthographicCameraBundle::new_2d())
        .insert(Transform::from_xyz(0.0, 0.0, 1000.0));
}
