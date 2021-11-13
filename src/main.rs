#![allow(clippy::complexity)]

// TODO: menu state: display menu before starting the game
// TODO: split general functionality into own crate
// TODO: tests in bevy?
// TODO: can we get event once a sound stops when not playing in a loop / whenever the loop resets?

use bevy::{log, prelude::*};
use derive_more::Display;
use std::path::PathBuf;
use structopt::StructOpt;
use walkdir::WalkDir;

mod effects;
mod plugins;
mod resources;
mod settings;
mod text;

use crate::{
    effects::{AnimationEffectPlugin, SoundEffectsPlugin},
    plugins::{
        AsteroidPlugin, FadeDespawnPlugin, GameOverPlugin, HitTestPlugin, LaserPlugin,
        MovementPlugin, PlayerPlugin, ScoreBoardPlugin,
    },
    resources::{
        AtlasAssetMap, AtlasAssetMapPlugin, AtlasDefinition, AudioAssetMap, AudioAssetMapPlugin,
        AudioPaths, BoundsPlugin, FontAssetMap, FontAssetMapPlugin, FontPaths, GfxBounds,
        TextureAssetMap, TextureAssetMapPlugin, TextureAtlasPaths, TexturePaths,
    },
};

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
struct AsteroidTexture(usize);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct BackgroundTexture(usize);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum Animation {
    BigExplosion,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum Fonts {
    ScoreBoard,
}

fn main() {
    let mut args = Args::from_args();
    let assets_path = args.assets.unwrap_or_else(|| {
        let mut pb = PathBuf::from(std::env::current_dir().unwrap());
        pb.push("assets");
        pb.display().to_string()
    });
    args.assets = Some(assets_path.clone());

    let settings = {
        let mut pb = PathBuf::from(&assets_path);
        pb.push("settings.toml");
        std::fs::read_to_string(pb).expect("unable to read settings")
    };
    let settings: settings::Settings =
        toml::from_str(&settings).expect("unable to parse settings file");

    let asteroid_textures = asteroid_texture_paths(&assets_path);
    let background_textures = background_texture_paths(&assets_path);

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(WindowDescriptor {
            vsync: true,
            resizable: false,
            width: settings.window.width as f32,
            height: settings.window.height as f32,
            title: module_path!().into(),
            ..WindowDescriptor::default()
        })
        .insert_resource(args)
        .add_plugin(BoundsPlugin)
        //
        // set the starting state & general systems
        .add_plugins(DefaultPlugins)
        // .add_plugin(bevy::diagnostic::LogDiagnosticsPlugin::default())
        // .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        .add_state(GameState::Initialize)
        .add_startup_system(initialize_game)
        // fonts
        .add_plugin(FontAssetMapPlugin::<Fonts>::default())
        .insert_resource(
            FontPaths::from_files([(Fonts::ScoreBoard, "fonts/FiraMono-Medium.ttf")])
                .with_base_path(assets_path.clone()),
        )
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
                settings.volume.asteroid_explosion,
            ),
            (SoundEffect::Laser, settings.volume.laser),
            (SoundEffect::ShipExplode, settings.volume.ship_explosion),
            (SoundEffect::Thruster, settings.volume.thruster),
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
        .insert_resource(TexturePaths::from_files(asteroid_textures))
        .add_plugin(TextureAssetMapPlugin::<BackgroundTexture>::default())
        .insert_resource(TexturePaths::from_files(background_textures))
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
        .insert_resource(settings)
        .run();
}

fn asteroid_texture_paths(assets_path: &str) -> Vec<(AsteroidTexture, String)> {
    let mut pb = PathBuf::from(assets_path);
    pb.push("gfx/asteroids");

    WalkDir::new(pb)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .enumerate()
        .map(|(i, e)| (AsteroidTexture(i), e.path().display().to_string()))
        .collect()
}

fn background_texture_paths(assets_path: &str) -> Vec<(BackgroundTexture, String)> {
    let mut pb = PathBuf::from(assets_path);
    pb.push("gfx/backgrounds");

    WalkDir::new(pb)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .enumerate()
        .map(|(i, e)| (BackgroundTexture(i), e.path().display().to_string()))
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
    tex1: Res<TextureAssetMap<GeneralTexture>>,
    tex2: Res<TextureAssetMap<AsteroidTexture>>,
    tex3: Res<TextureAssetMap<BackgroundTexture>>,
    anim1: Res<AtlasAssetMap<Animation>>,
    audio1: Res<AudioAssetMap<SoundEffect>>,
    fonts: Res<FontAssetMap<Fonts>>,
) {
    if tex1.ready()
        && tex2.ready()
        && tex3.ready()
        && anim1.ready()
        && audio1.ready()
        && fonts.ready()
    {
        state
            .set(GameState::InGame)
            .expect("unable to transition into the InGame state");
    }
}
