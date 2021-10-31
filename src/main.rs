#![allow(clippy::complexity)]
use bevy::{log, prelude::*, sprite::SpriteSettings};
use derive_more::Display;
use scoreboard::ScoreBoardPlugin;
use structopt::StructOpt;
use textures::TextureLoaderPlugin;

use crate::{
    asteroid_plugin::AsteroidPlugin, bounds::Bounds, fade_plugin::FadePlugin,
    hit_test::HitTestPlugin, movement_plugin::MovementPlugin, player_plugin::PlayerPlugin,
};

mod asset_helper;
mod asteroid_plugin;
mod bounds;
mod constants;
mod fade_plugin;
mod hit_test;
mod movement_plugin;
mod player_plugin;
mod scoreboard;
mod textures;

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(long)]
    assets: Option<String>,
}

#[derive(Debug, Display, Copy, Clone, Eq, PartialEq, Hash)]
enum GameState {
    Initialize,
    InGame,
    GameOver,
}

#[derive(Debug, Component, Display)]
struct Despawn;

// TODO: scoring
// TODO: respawn player / lives / levels

fn main() {
    App::new()
        // set the starting state & general systems
        .add_state(GameState::Initialize)
        .add_startup_system_to_stage(StartupStage::PreStartup, initialize.system())
        .add_system_to_stage(CoreStage::PostUpdate, despawn.system())
        .add_plugins(DefaultPlugins)
        // .add_plugin(bevy::diagnostic::LogDiagnosticsPlugin::default())
        // .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        //
        // game plugins
        .add_plugin(ScoreBoardPlugin)
        .add_plugin(TextureLoaderPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(AsteroidPlugin)
        .add_plugin(MovementPlugin)
        .add_plugin(FadePlugin)
        .add_plugin(HitTestPlugin)
        .add_system_set(
            SystemSet::on_update(GameState::GameOver).with_system(restart_on_enter.system()),
        )
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

fn initialize(mut commands: Commands, mut windows: ResMut<Windows>) {
    log::info!("initializing game");
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

fn despawn(mut commands: Commands, query: Query<Entity, With<Despawn>>) {
    for entity in query.iter() {
        log::debug!(?entity, "despawning");
        commands.entity(entity).despawn_recursive();
    }
}

fn restart_on_enter(kb: Res<Input<KeyCode>>, mut state: ResMut<State<GameState>>) {
    if kb.pressed(KeyCode::Return) {
        state.set(GameState::InGame).unwrap();
    }
}
