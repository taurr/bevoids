use bevy::{log, prelude::*};
use bevy_asset_map::{
    monitor_atlas_assets, monitor_audio_assets, monitor_font_assets, monitor_texture_assets,
    AtlasAssetInfo, BoundsPlugin, TextureAssetInfo,
};
use bevy_effects::{
    animation::AnimationEffectPlugin,
    despawn::DespawnPlugin,
    sound::{play_sound_effect_on_event, SfxCmdEvent},
};
use derive_more::{AsRef, Deref, Display, From, Into};

mod asteroids;
mod gameover;
mod hit_test;
mod laser;
mod movement;
mod player;
mod resources;
mod scoreboard;
pub mod settings;

use {
    asteroids::*, gameover::*, hit_test::*, laser::*, movement::*, player::*, resources::*,
    scoreboard::*,
};

#[derive(Debug, Display, Clone, Eq, PartialEq, AsRef, Deref, From, Into)]
pub struct AssetPath(String);

#[derive(Debug, Display, Copy, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    Initialize,
    InGame,
    GameOver,
}

#[derive(Debug, Default)]
pub struct Bevoids {}

impl Plugin for Bevoids {
    fn build(&self, app: &mut App) {
        log::trace!("setting up systems");

        // events
        app.add_event::<EnterWindowEvent>()
            .add_event::<ExitWindowEvent>()
            .add_event::<PlayerDeadEvent>()
            .add_event::<SpawnAsteroidEvent>()
            .add_event::<AsteroidShotEvent>()
            .add_event::<AsteroidExplosionEvent>()
            .add_event::<FireLaserEvent>()
            .add_event::<AddScoreEvent>();

        // movement
        app.add_system_set(
            SystemSet::new()
                .with_system(wrapping_linear_movement)
                .with_system(non_wrapping_linear_movement)
                .with_system(move_shadow),
        );

        // sound
        app.add_plugin(bevy_kira_audio::AudioPlugin)
            .add_plugin(AnimationEffectPlugin::<AnimationAtlas>::new())
            .add_system(play_sound_effect_on_event::<SoundEffect>);

        // misc needed plugins
        app.add_plugin(DespawnPlugin).add_plugin(BoundsPlugin);

        // introduce the state to its relevant stages
        let state = GameState::Initialize;
        app.add_state_to_stage(CoreStage::PreUpdate, state)
            .add_state_to_stage(CoreStage::Update, state)
            .add_state_to_stage(CoreStage::PostUpdate, state);

        setup_initialize(app);
        setup_ingame(app);
        setup_gameover(app);
    }
}

fn setup_initialize(app: &mut App) {
    log::trace!("systems specific for initialize");

    app.add_event::<AtlasAssetInfo<AnimationAtlas>>()
        .add_event::<TextureAssetInfo<GeneralTexture>>()
        .add_event::<TextureAssetInfo<AsteroidTexture>>()
        .add_event::<TextureAssetInfo<BackgroundTexture>>()
        .add_event::<SfxCmdEvent<SoundEffect>>();

    app.add_startup_system_set(
        SystemSet::new()
            .with_system(load_gamefont)
            .with_system(load_general_textures)
            .with_system(load_asteroid_textures)
            .with_system(load_background_textures)
            .with_system(load_animations)
            .with_system(load_audio),
    );

    app.add_system_set(
        SystemSet::on_update(GameState::Initialize)
            .with_system(monitor_font_assets::<GameFont>)
            .with_system(monitor_texture_assets::<GeneralTexture>)
            .with_system(monitor_texture_assets::<AsteroidTexture>)
            .with_system(monitor_texture_assets::<BackgroundTexture>)
            .with_system(monitor_atlas_assets::<AnimationAtlas>)
            .with_system(monitor_audio_assets::<SoundEffect>)
            .with_system(wait_for_resources),
    );
}

fn setup_ingame(app: &mut App) {
    app.add_system_set(
        SystemSet::on_enter(GameState::InGame)
            .with_system(spawn_player)
            .with_system(spawn_asteroid_spawner)
            .with_system(setup_ingame_scoreboard),
    )
    .add_system_set_to_stage(
        CoreStage::Update,
        SystemSet::on_update(GameState::InGame)
            .with_system(player_controls)
            .with_system(asteroid_spawner)
            .with_system(handle_fire_laser)
            .with_system(handle_player_dead)
            .with_system(handle_shot_asteroids)
            .with_system(hittest_shot_vs_asteroid)
            .with_system(hittest_player_vs_asteroid)
            .with_system(update_scoreboard),
    )
    .add_system_set_to_stage(
        CoreStage::PostUpdate,
        SystemSet::on_update(GameState::InGame)
            .with_system(handle_spawn_asteroid)
            .with_system(handle_asteroid_explosion),
    )
    .add_system_set(
        SystemSet::on_exit(GameState::InGame)
            .with_system(despawn_asteroid_spawner)
            .with_system(stop_thruster_sounds),
    );
}

fn setup_gameover(app: &mut App) {
    app.add_system_set(
        SystemSet::on_enter(GameState::GameOver)
            .with_system(setup_gameover_scoreboard)
            .with_system(init_gameover_texts),
    )
    .add_system_set(SystemSet::on_update(GameState::GameOver).with_system(restart_on_enter))
    .add_system_set(
        SystemSet::on_exit(GameState::GameOver).with_system(remove_texts_on_exit_gameover),
    );
}
