use bevy::{log, prelude::*};
use bevy_asset_map::{
    AtlasAssetMap, AtlasDefinition, AudioAssetMap, AudioPaths, TextureAssetMap, TextureAtlasPaths,
    TexturePaths,
};
use bevy_effects::sound::set_audio_channel_defaults;
use bevy_kira_audio::Audio;
use derive_more::Display;
use std::path::PathBuf;
use walkdir::WalkDir;

use super::{settings::Settings, AssetPath, GameState};

#[derive(Debug, Display, Copy, Clone, Eq, PartialEq, Hash)]
pub(crate) enum SoundEffect {
    Notification,
    Laser,
    Thruster,
    ShipExplode,
    AsteroidExplode,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub(crate) enum GeneralTexture {
    Laser,
    Flame,
    Spaceship,
    Trophy,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub(crate) enum AnimationAtlas {
    BigExplosion,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub(crate) struct AsteroidTexture(pub usize);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub(crate) struct BackgroundTexture(pub usize);

pub(crate) fn load_general_textures(
    mut commands: Commands,
    assets_path: Res<AssetPath>,
    asset_server: Res<AssetServer>,
) {
    log::debug!("loading textures");

    commands.insert_resource(TextureAssetMap::with_texture_paths(
        &TexturePaths::from_files([
            (GeneralTexture::Laser, "gfx/laser.png"),
            (GeneralTexture::Spaceship, "gfx/spaceship.png"),
            (GeneralTexture::Flame, "gfx/flame.png"),
            (GeneralTexture::Trophy, "gfx/trophy.png"),
        ])
        .with_base_path(assets_path.clone()),
        &asset_server,
    ));
}

pub(crate) fn load_asteroid_textures(
    mut commands: Commands,
    assets_path: Res<AssetPath>,
    asset_server: Res<AssetServer>,
) {
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

    log::debug!("loading asteroids");

    commands.insert_resource(TextureAssetMap::with_texture_paths(
        &TexturePaths::from_files(asteroid_texture_paths(&assets_path)),
        &asset_server,
    ));
}

pub(crate) fn load_background_textures(
    mut commands: Commands,
    assets_path: Res<AssetPath>,
    asset_server: Res<AssetServer>,
) {
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

    log::debug!("loading backgrounds");

    commands.insert_resource(TextureAssetMap::with_texture_paths(
        &TexturePaths::from_files(background_texture_paths(&assets_path)),
        &asset_server,
    ));
}

pub(crate) fn load_animations(
    mut commands: Commands,
    assets_path: Res<AssetPath>,
    asset_server: Res<AssetServer>,
) {
    log::debug!("loading animations");
    commands.insert_resource(AtlasAssetMap::with_texture_paths(
        &TextureAtlasPaths::from_files([(
            AnimationAtlas::BigExplosion,
            "gfx/explosion.png",
            AtlasDefinition::Grid {
                columns: 9,
                rows: 9,
            },
        )])
        .with_base_path(assets_path.clone()),
        &asset_server,
    ));
}

pub(crate) fn load_audio(
    mut commands: Commands,
    assets_path: Res<AssetPath>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    settings: Res<Settings>,
) {
    log::debug!("loading audio");
    commands.insert_resource(AudioAssetMap::with_audio_paths(
        &AudioPaths::from_files([
            (SoundEffect::Notification, "sounds/notification.wav"),
            (SoundEffect::AsteroidExplode, "sounds/asteroid_explode.wav"),
            (SoundEffect::Laser, "sounds/laser.wav"),
            (SoundEffect::ShipExplode, "sounds/ship_explode.wav"),
            (SoundEffect::Thruster, "sounds/thruster.wav"),
        ])
        .with_base_path(assets_path.clone()),
        &asset_server,
    ));
    log::trace!("setting default volume in audio channels");
    set_audio_channel_defaults::<_, _, &[_]>(
        Some([
            (SoundEffect::Notification, 1.0),
            (
                SoundEffect::AsteroidExplode,
                settings.volume.asteroid_explosion,
            ),
            (SoundEffect::Laser, settings.volume.laser),
            (SoundEffect::ShipExplode, settings.volume.ship_explosion),
            (SoundEffect::Thruster, settings.volume.thruster),
        ]),
        None,
        &audio,
        &mut commands,
    );
}

pub(crate) fn wait_for_resources(
    mut state: ResMut<State<GameState>>,
    tex1: Res<TextureAssetMap<GeneralTexture>>,
    tex2: Res<TextureAssetMap<AsteroidTexture>>,
    tex3: Res<TextureAssetMap<BackgroundTexture>>,
    anim1: Res<AtlasAssetMap<AnimationAtlas>>,
    audio1: Res<AudioAssetMap<SoundEffect>>,
) {
    log::trace!("waiting...");

    if tex1.ready() && tex2.ready() && tex3.ready() && anim1.ready() && audio1.ready() {
        state
            .set(GameState::MainMenu)
            .expect("unable to transition into the InGame state");
    }
}
