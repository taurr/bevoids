use bevoids_assets::BevoidsAssets;
use bevy::{log, prelude::*};
use bevy_asset_map::{
    AtlasAssetMap, AtlasDefinition, AudioAssetMap, AudioPaths, TextureAssetMap, TextureAtlasPaths,
    TexturePaths,
};
use bevy_effects::sound::set_audio_channel_defaults;
use bevy_kira_audio::Audio;
use derive_more::Display;

use super::{settings::Settings, GameState};

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

pub(crate) fn load_general_textures(mut commands: Commands, asset_server: Res<AssetServer>) {
    log::debug!("loading textures");

    commands.insert_resource(TextureAssetMap::with_texture_paths(
        &TexturePaths::from_files([
            (GeneralTexture::Laser, BevoidsAssets::GfxLaser.path().display().to_string()),
            (GeneralTexture::Spaceship, BevoidsAssets::GfxSpaceship.path().display().to_string()),
            (GeneralTexture::Flame, BevoidsAssets::GfxFlame.path().display().to_string()),
            (GeneralTexture::Trophy, BevoidsAssets::GfxTrophy.path().display().to_string()),
        ]),
        &asset_server,
    ));
}

pub(crate) fn load_asteroid_textures(mut commands: Commands, asset_server: Res<AssetServer>) {
    log::debug!("loading asteroids");
    let assets = vec![
        BevoidsAssets::GfxAsteroids1.path().display().to_string(),
        BevoidsAssets::GfxAsteroids2.path().display().to_string(),
        BevoidsAssets::GfxAsteroids3.path().display().to_string(),
        BevoidsAssets::GfxAsteroids4.path().display().to_string(),
        BevoidsAssets::GfxAsteroids5.path().display().to_string(),
        BevoidsAssets::GfxAsteroids6.path().display().to_string(),
        BevoidsAssets::GfxAsteroids7.path().display().to_string(),
    ];

    commands.insert_resource(TextureAssetMap::with_texture_paths(
        &TexturePaths::from_files(
            assets
                .into_iter()
                .enumerate()
                .map(|(i, e)| (AsteroidTexture(i), e)),
        ),
        &asset_server,
    ));
}

pub(crate) fn load_background_textures(mut commands: Commands, asset_server: Res<AssetServer>) {
    log::debug!("loading backgrounds");
    let assets = vec![
        BevoidsAssets::GfxBackgrounds1.path().display().to_string(),
        BevoidsAssets::GfxBackgrounds2.path().display().to_string(),
        BevoidsAssets::GfxBackgrounds3.path().display().to_string(),
        BevoidsAssets::GfxBackgrounds4.path().display().to_string(),
        BevoidsAssets::GfxBackgrounds5.path().display().to_string(),
        BevoidsAssets::GfxBackgrounds6.path().display().to_string(),
        BevoidsAssets::GfxBackgrounds7.path().display().to_string(),
        BevoidsAssets::GfxBackgrounds8.path().display().to_string(),
        BevoidsAssets::GfxBackgrounds9.path().display().to_string(),
        BevoidsAssets::GfxBackgrounds10.path().display().to_string(),
        BevoidsAssets::GfxBackgrounds11.path().display().to_string(),
        BevoidsAssets::GfxBackgrounds12.path().display().to_string(),
    ];

    commands.insert_resource(TextureAssetMap::with_texture_paths(
        &TexturePaths::from_files(
            assets
                .into_iter()
                .enumerate()
                .map(|(i, e)| (BackgroundTexture(i), e)),
        ),
        &asset_server,
    ));
}

pub(crate) fn load_animations(mut commands: Commands, asset_server: Res<AssetServer>) {
    log::debug!("loading animations");
    commands.insert_resource(AtlasAssetMap::with_texture_paths(
        &TextureAtlasPaths::from_files([(
            AnimationAtlas::BigExplosion,
            BevoidsAssets::GfxExplosion.path().display().to_string(),
            AtlasDefinition::Grid {
                columns: 9,
                rows: 9,
            },
        )]),
        &asset_server,
    ));
}

pub(crate) fn load_audio(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    settings: Res<Settings>,
) {
    log::debug!("loading audio");
    commands.insert_resource(AudioAssetMap::with_audio_paths(
        &AudioPaths::from_files([
            (SoundEffect::Notification, BevoidsAssets::SoundNotification.path().display().to_string()),
            (SoundEffect::AsteroidExplode, BevoidsAssets::SoundAsteroidExplode.path().display().to_string()),
            (SoundEffect::Laser, BevoidsAssets::SoundLaser.path().display().to_string()),
            (SoundEffect::ShipExplode, BevoidsAssets::SoundShipExplode.path().display().to_string()),
            (SoundEffect::Thruster, BevoidsAssets::SoundThruster.path().display().to_string()),
        ]),
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
