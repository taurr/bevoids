#![allow(clippy::complexity)]
use bevy::{log, prelude::*, sprite::SpriteSettings};
use bevy_kira_audio::*;
use constants::{
    AUDIO_EXPLOSION_ASTEROID_VOLUME, AUDIO_EXPLOSION_SHIP_VOLUME, AUDIO_LASER_VOLUME,
    AUDIO_THRUSTER_VOLUME,
};
use derive_more::{Deref, Display, Into};
use plugins::Despawn;
use resources::TextureAtlasAssets;
use smol_str::SmolStr;
use std::path::PathBuf;
use structopt::StructOpt;

use crate::{
    plugins::{
        AsteroidPlugin, FadeDespawnPlugin, GameOverPlugin, HitTestPlugin, LaserPlugin,
        MovementPlugin, PlayerPlugin, ScoreBoardPlugin,
    },
    resources::{
        AtlasDef, AudioAssets, AudioAssetsPlugin, AudioPaths, Bounds, BoundsPlugin, TextureAssets,
        TextureAssetsPlugin, TextureAtlasAssetsPlugin, TextureAtlasPaths, TexturePaths,
    },
};

mod constants;
mod plugins;
mod resources;
mod text;

#[derive(Debug, StructOpt, Clone)]
struct Args {
    #[structopt(long)]
    assets: Option<SmolStr>,
}

// TODO: use modifier to alter turn speed of player
// TODO: menu state: display menu before starting the game

#[derive(Debug, Display, Copy, Clone, Eq, PartialEq, Hash)]
enum GameState {
    Initialize,
    InGame,
    GameOver,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Display)]
enum Sounds {
    Notification,
    AsteroidExplode,
    Laser,
    ShipExplode,
    Thruster,
}

#[derive(Debug)]
struct AudioChannels {
    pub laser: AudioChannel,
    pub thruster: AudioChannel,
    pub ship_explode: AudioChannel,
    pub asteroid_explode: AudioChannel,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum GeneralTexture {
    Laser,
    Flame,
    Spaceship,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
struct AsteroidTexture(u8);

#[derive(Debug, Clone, Copy, Deref, Into)]
struct AsteroidTextureCount(u8);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum Animations {
    BigExplosion,
}

fn main() {
    let args = Args::from_args();
    let asteroid_textures: Vec<(AsteroidTexture, String)> = (0u8..10u8)
        .into_iter()
        .map(|i| (i, &args.assets))
        .filter_map(|(i, base_path)| valid_asteroid_texture_path(base_path, i))
        .enumerate()
        .map(|(i, path)| (AsteroidTexture(i as u8), path))
        .collect();

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        // .add_system(animate_sprite_system)
        // .add_plugin(bevy::diagnostic::LogDiagnosticsPlugin::default())
        // .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        //
        // set the starting state & general systems
        .add_state_to_stage(CoreStage::Update, GameState::Initialize)
        .add_state_to_stage(CoreStage::PostUpdate, GameState::Initialize)
        .add_startup_system(initialize.system())
        .add_plugin(BoundsPlugin)
        .add_plugin(AudioAssetsPlugin::<Sounds>::default())
        .insert_resource(AudioPaths::from_path_and_files(
            args.assets.clone(),
            [
                (Sounds::Notification, "sounds/notification.wav"),
                (Sounds::AsteroidExplode, "sounds/asteroid_explode.wav"),
                (Sounds::Laser, "sounds/laser.wav"),
                (Sounds::ShipExplode, "sounds/ship_explode.wav"),
                (Sounds::Thruster, "sounds/thruster.wav"),
            ],
        ))
        .add_plugin(TextureAssetsPlugin::<GeneralTexture>::default())
        .add_plugin(TextureAssetsPlugin::<AsteroidTexture>::default())
        .insert_resource(TexturePaths::from_path_and_files(
            args.assets.clone(),
            [
                (GeneralTexture::Laser, "gfx/laser.png"),
                (GeneralTexture::Spaceship, "gfx/spaceship.png"),
                (GeneralTexture::Flame, "gfx/flame.png"),
            ],
        ))
        .insert_resource(AsteroidTextureCount(asteroid_textures.len() as u8))
        .insert_resource(TexturePaths::from_files(asteroid_textures))
        .add_plugin(TextureAtlasAssetsPlugin::<Animations>::default())
        .insert_resource(TextureAtlasPaths::from_path_and_files(
            args.assets.clone(),
            [(
                Animations::BigExplosion,
                "gfx/explosion.png",
                AtlasDef::Grid {
                    columns: 9,
                    rows: 9,
                },
            )],
        ))
        //
        // game plugins
        .add_system_set(SystemSet::on_update(GameState::Initialize).with_system(wait_for_textures))
        .add_event::<StartSingleAnimation>()
        .add_system(handle_animation_events)
        .add_system(animate_sprite_system)
        .add_plugin(FadeDespawnPlugin)
        .add_plugin(MovementPlugin)
        .add_plugin(ScoreBoardPlugin)
        .add_plugin(LaserPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(AsteroidPlugin)
        .add_plugin(HitTestPlugin)
        .add_plugin(GameOverPlugin)
        //
        // resources
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
        .insert_resource(SpriteSettings {
            frustum_culling_enabled: true,
        })
        .run();
}

fn valid_asteroid_texture_path(base_path: &Option<SmolStr>, i: u8) -> Option<String> {
    let mut pb = if let Some(ref base_path) = base_path {
        PathBuf::from(base_path.as_str())
    } else {
        let mut pb = PathBuf::from(std::env::current_dir().expect("unable to get current_dir"));
        pb.push("assets");
        pb
    };
    pb.push(format!("gfx/asteroid_{}.png", i));
    match pb.exists() {
        true => Some(pb.display().to_string()),
        false => None,
    }
}

fn initialize(mut commands: Commands) {
    log::info!("initializing game");
    // Spawns the camera
    commands
        .spawn()
        .insert_bundle(OrthographicCameraBundle::new_2d())
        .insert(Transform::from_xyz(0.0, 0.0, 1000.0));
}

struct StartSingleAnimation {
    key: Animations,
    size: f32,
    position: Vec3,
}

#[derive(Component, Debug, Copy, Clone)]
struct LoopLimit(u32);

fn handle_animation_events(
    mut events: EventReader<StartSingleAnimation>,
    mut commands: Commands,
    texture_atlases: Res<TextureAtlasAssets<Animations>>,
) {
    for ev in events.iter() {
        let atlas_info = texture_atlases.get(ev.key).unwrap();
        let atlas_handle = atlas_info.atlas.clone();
        let scale = ev.size / atlas_info.size.max_element();

        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: atlas_handle,
                transform: Transform::from_translation(ev.position).with_scale(Vec3::splat(scale)),
                ..Default::default()
            })
            .insert(LoopLimit(1))
            // TODO: use constant
            .insert(Timer::from_seconds(1. / 30., true));
    }
}

fn animate_sprite_system(
    mut commands: Commands,
    time: Res<Time>,
    atlas_assets: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        Entity,
        &mut Timer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
        Option<&mut LoopLimit>,
    )>,
) {
    for (entity, mut timer, mut sprite, texture_atlas_handle, limit) in query.iter_mut() {
        if timer.tick(time.delta()).finished() {
            let texture_atlas = atlas_assets.get(texture_atlas_handle).unwrap();
            sprite.index = ((sprite.index as usize + 1) % texture_atlas.textures.len()) as u32;

            if sprite.index != 0 {
                continue;
            }

            if let Some(mut limit) = limit {
                if limit.0 > 1 {
                    limit.0 -= 1;
                } else {
                    commands.entity(entity).insert(Despawn);
                }
            }
        } else if let Some(limit) = limit {
            if limit.0 == 0 {
                commands.entity(entity).insert(Despawn);
            }
        }
    }
}

fn wait_for_textures(
    mut commands: Commands,
    mut state: ResMut<State<GameState>>,
    audio: Res<Audio>,
    t1: Res<TextureAssets<GeneralTexture>>,
    t2: Res<TextureAssets<AsteroidTexture>>,
    a1: Res<AudioAssets<Sounds>>,
) {
    if t1.ready() && t2.ready() && a1.ready() {
        // Create channels and set volume ahead of time
        let channels = AudioChannels {
            laser: AudioChannel::new(Sounds::Laser.to_string()),
            thruster: AudioChannel::new(Sounds::Thruster.to_string()),
            ship_explode: AudioChannel::new(Sounds::ShipExplode.to_string()),
            asteroid_explode: AudioChannel::new(Sounds::AsteroidExplode.to_string()),
        };
        audio.set_volume_in_channel(AUDIO_LASER_VOLUME, &channels.laser);
        audio.set_volume_in_channel(AUDIO_THRUSTER_VOLUME, &channels.thruster);
        audio.set_volume_in_channel(AUDIO_EXPLOSION_SHIP_VOLUME, &channels.ship_explode);
        audio.set_volume_in_channel(AUDIO_EXPLOSION_ASTEROID_VOLUME, &channels.asteroid_explode);

        audio.play(a1.get(Sounds::Notification).unwrap());

        commands.insert_resource(channels);

        state
            .set(GameState::InGame)
            .expect("unable to transition into the InGame state");
    }
}
