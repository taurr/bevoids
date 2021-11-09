#![allow(clippy::complexity)]

// TODO: move all constants to settings file
// TODO: use structopt to specify settings
// TODO: menu state: display menu before starting the game
// TODO: split general functionality into own crate
// TODO: tests in bevy?
// TODO: can we get event once a sound stops when not playing in a loop / whenever the loop resets?

use bevy::{log, prelude::*};
use derive_more::{AsRef, Deref, Display, From, Into};
use smol_str::SmolStr;
use std::path::PathBuf;
use structopt::StructOpt;

mod constants;
mod effects;
mod plugins;
mod resources;
mod text;

use crate::{
    constants::{
        AUDIO_EXPLOSION_ASTEROID_VOLUME, AUDIO_EXPLOSION_SHIP_VOLUME, AUDIO_LASER_VOLUME,
        AUDIO_THRUSTER_VOLUME,
    },
    effects::{AnimationEffectPlugin, SoundEffectsPlugin},
    plugins::{
        AsteroidPlugin, FadeDespawnPlugin, GameOverPlugin, HitTestPlugin, LaserPlugin,
        MovementPlugin, PlayerPlugin, ScoreBoardPlugin,
    },
    resources::{
        AtlasAssetMap, AtlasAssetMapPlugin, AtlasDefinition, AudioAssetMap, AudioAssetMapPlugin,
        AudioPaths, Bounds, BoundsPlugin, TextureAssetMap, TextureAssetMapPlugin,
        TextureAtlasPaths, TexturePaths,
    },
};

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(long)]
    assets: Option<SmolStr>,
}

#[derive(Debug, Display, Copy, Clone, Eq, PartialEq, Hash)]
enum GameState {
    Initialize,
    InGame,
    GameOver,
}

#[derive(Debug, Display, Copy, Clone, Eq, PartialEq, Hash)]
enum SoundEffect {
    Notification,
    Laser,
    Thruster,
    ShipExplode,
    AsteroidExplode,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum GeneralTexture {
    Laser,
    Flame,
    Spaceship,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct AsteroidTexture(u8);

#[derive(Debug, Copy, Clone, Deref, Into, From, AsRef)]
struct AsteroidTextureCount(u8);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum Animation {
    BigExplosion,
}

fn main() {
    let args = Args::from_args();
    let assets_path = args.assets.clone();
    let asteroid_textures = asteroid_texture_paths(&assets_path);

    App::new()
        .insert_resource(args)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(WindowDescriptor {
            vsync: true,
            resizable: true,
            width: constants::WIN_WIDTH,
            height: constants::WIN_HEIGHT,
            title: module_path!().into(),
            ..WindowDescriptor::default()
        })
        //
        // set the starting state & general systems
        .add_plugins(DefaultPlugins)
        // .add_plugin(bevy::diagnostic::LogDiagnosticsPlugin::default())
        // .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        .add_state(GameState::Initialize)
        .add_startup_system(initialize_game)
        .add_plugin(BoundsPlugin)
        //
        // graphics / sound effects
        .add_plugin(AudioAssetMapPlugin::<SoundEffect>::default())
        .insert_resource(
            AudioPaths::from_files([
                (SoundEffect::Notification, "sounds/notification.wav"),
                (SoundEffect::AsteroidExplode, "sounds/asteroid_explode.wav"),
                (SoundEffect::Laser, "sounds/laser.wav"),
                (SoundEffect::ShipExplode, "sounds/ship_explode.wav"),
                (SoundEffect::Thruster, "sounds/thruster.wav"),
            ])
            .with_base_path(assets_path.clone()),
        )
        .add_plugin(SoundEffectsPlugin::<SoundEffect>::new().with_volumes([
            (SoundEffect::Notification, 1.0),
            (
                SoundEffect::AsteroidExplode,
                AUDIO_EXPLOSION_ASTEROID_VOLUME,
            ),
            (SoundEffect::Laser, AUDIO_LASER_VOLUME),
            (SoundEffect::ShipExplode, AUDIO_EXPLOSION_SHIP_VOLUME),
            (SoundEffect::Thruster, AUDIO_THRUSTER_VOLUME),
        ]))
        .add_plugin(TextureAssetMapPlugin::<GeneralTexture>::default())
        .insert_resource(
            TexturePaths::from_files([
                (GeneralTexture::Laser, "gfx/laser.png"),
                (GeneralTexture::Spaceship, "gfx/spaceship.png"),
                (GeneralTexture::Flame, "gfx/flame.png"),
            ])
            .with_base_path(assets_path.clone()),
        )
        .add_plugin(TextureAssetMapPlugin::<AsteroidTexture>::default())
        .insert_resource(AsteroidTextureCount(asteroid_textures.len() as u8))
        .insert_resource(TexturePaths::from_files(asteroid_textures))
        .add_plugin(AnimationEffectPlugin::<Animation>::default())
        .add_plugin(AtlasAssetMapPlugin::<Animation>::default())
        .insert_resource(
            TextureAtlasPaths::from_files([(
                Animation::BigExplosion,
                "gfx/explosion.png",
                AtlasDefinition::Grid {
                    columns: 9,
                    rows: 9,
                },
            )])
            .with_base_path(assets_path),
        )
        //
        // game plugins
        .add_system_set(SystemSet::on_update(GameState::Initialize).with_system(wait_for_resources))
        .add_plugin(FadeDespawnPlugin)
        .add_plugin(MovementPlugin)
        .add_plugin(ScoreBoardPlugin)
        .add_plugin(LaserPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(AsteroidPlugin)
        .add_plugin(HitTestPlugin)
        .add_plugin(GameOverPlugin)
        .run();
}

fn asteroid_texture_paths(assets_path: &Option<SmolStr>) -> Vec<(AsteroidTexture, SmolStr)> {
    (0u8..10u8)
        .into_iter()
        .map(|i| (i, assets_path))
        .filter_map(|(i, base_path)| {
            let mut pb = if let Some(ref base_path) = base_path {
                PathBuf::from(base_path.as_str())
            } else {
                let mut pb =
                    PathBuf::from(std::env::current_dir().expect("unable to get current_dir"));
                pb.push("assets");
                pb
            };
            pb.push(format!("gfx/asteroid_{}.png", i));
            match pb.exists() {
                true => Some(pb.display().to_string().into()),
                false => None,
            }
        })
        .enumerate()
        .map(|(i, path)| (AsteroidTexture(i as u8), path))
        .collect()
}

fn initialize_game(mut commands: Commands) {
    log::info!("initializing game");
    // Spawns the camera
    commands
        .spawn()
        .insert_bundle(OrthographicCameraBundle::new_2d())
        .insert(Transform::from_xyz(0.0, 0.0, 1000.0));
}

fn wait_for_resources(
    mut state: ResMut<State<GameState>>,
    t1: Res<TextureAssetMap<GeneralTexture>>,
    t2: Res<TextureAssetMap<AsteroidTexture>>,
    t3: Res<AtlasAssetMap<Animation>>,
    a1: Res<AudioAssetMap<SoundEffect>>,
) {
    if t1.ready() && t2.ready() && t3.ready() && a1.ready() {
        state
            .set(GameState::InGame)
            .expect("unable to transition into the InGame state");
    }
}
