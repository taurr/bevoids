use bevy::{ecs::schedule::ShouldRun, log, prelude::*};
use bevy_asset_map::{BoundsPlugin, FontAssetMapPlugin, TextureAssetMapPlugin};
use bevy_effects::{
    animation::AnimationEffectPlugin, despawn::DespawnPlugin, sound::SoundEffectsPlugin,
};
use bevy_egui::{EguiContext, EguiPlugin, egui};
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
    Menu,
    Playing,
    Paused,
    GameOver,
}

#[derive(Debug, Default)]
pub struct Bevoids {}

impl Plugin for Bevoids {
    fn build(&self, app: &mut AppBuilder) {
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

        // misc
        app.add_plugin(DespawnPlugin::with_run_criteria(run_if_not_paused.system()))
            .add_plugin(AnimationEffectPlugin::<AnimationAtlas>::with_run_criteria(
                run_if_not_paused.system(),
            ))
            .add_plugin(SoundEffectsPlugin::<SoundEffect>::default())
            .add_plugin(FontAssetMapPlugin::<GameFont>::default())
            .add_plugin(TextureAssetMapPlugin::<GeneralTexture>::default())
            .add_plugin(TextureAssetMapPlugin::<AsteroidTexture>::default())
            .add_plugin(TextureAssetMapPlugin::<BackgroundTexture>::default())
            .add_plugin(BoundsPlugin);

        // introduce the state to its relevant stages
        app.insert_resource(State::new(GameState::Initialize))
            .add_system_set_to_stage(CoreStage::PreUpdate, State::<GameState>::get_driver())
            .add_system_set_to_stage(CoreStage::Update, State::<GameState>::get_driver())
            .add_system_set_to_stage(CoreStage::PostUpdate, State::<GameState>::get_driver());

        // all states
        app.add_system_set_to_stage(
            CoreStage::Update,
            SystemSet::new().with_system(pause_control.system()),
        );

        setup_initialize(app);
        setup_menu(app);
        setup_playing(app);
        setup_paused(app);
        setup_gameover(app);
    }
}

fn setup_menu(app: &mut AppBuilder) {
    let state = GameState::Menu;

    app.add_plugin(EguiPlugin)
        .add_system_set(SystemSet::on_update(state).with_system(start_menu.system()))
        .add_system_set(SystemSet::on_update(state).with_system(restart_on_enter.system()))
        .add_system_set(SystemSet::on_exit(state).with_system(stop_menu.system()));
}

fn setup_initialize(app: &mut AppBuilder) {
    let state = GameState::Initialize;

    app.add_startup_system(load_gamefont.system())
        .add_startup_system(load_general_textures.system())
        .add_startup_system(load_asteroid_textures.system())
        .add_startup_system(load_background_textures.system())
        .add_startup_system(load_animations.system())
        .add_startup_system(load_audio.system())
        .add_system_set(SystemSet::on_update(state).with_system(wait_for_resources.system()));
}

fn setup_playing(app: &mut AppBuilder) {
    let state = GameState::Playing;

    app.add_system_set(
        SystemSet::on_enter(state)
            .with_system(spawn_player.system())
            .with_system(spawn_asteroid_spawner.system())
            .with_system(setup_ingame_scoreboard.system()),
    )
    .add_system_set_to_stage(
        CoreStage::Update,
        SystemSet::on_update(state)
            .with_system(player_controls.system().label("input"))
            .with_system(asteroid_spawner.system().label("spawner"))
            .with_system(handle_fire_laser.system().after("input"))
            .with_system(handle_player_dead.system())
            .with_system(handle_shot_asteroids.system())
            .with_system(update_scoreboard.system())
            .with_system(hittest_shot_vs_asteroid.system())
            .with_system(hittest_player_vs_asteroid.system())
            //
            .with_system(wrapping_linear_movement.system())
            .with_system(non_wrapping_linear_movement.system())
            .with_system(move_shadow.system())
            .with_system(handle_spawn_asteroid.system().after("spawner"))
            .with_system(handle_asteroid_explosion.system()),
    )
    .add_system_set(SystemSet::on_exit(state).with_system(stop_thruster_sounds.system()));
}

fn setup_paused(_app: &mut AppBuilder) {
    let _state = GameState::Paused;
}

fn setup_gameover(app: &mut AppBuilder) {
    let state = GameState::GameOver;

    app.add_system_set(
        SystemSet::on_enter(state)
            .with_system(despawn_asteroid_spawner.system())
            .with_system(setup_gameover_scoreboard.system())
            .with_system(init_gameover_texts.system()),
    )
    .add_system_set_to_stage(
        CoreStage::Update,
        SystemSet::on_update(state)
            .with_system(restart_on_enter.system())
            .with_system(wrapping_linear_movement.system())
            .with_system(non_wrapping_linear_movement.system())
            .with_system(move_shadow.system())
            .with_system(handle_asteroid_explosion.system()),
    )
    .add_system_set(SystemSet::on_exit(state).with_system(remove_gameover_texts.system()));
}

fn pause_control(mut kb: ResMut<Input<KeyCode>>, mut state: ResMut<State<GameState>>) {
    // NOTE: triggering state changes from input, requires us to reset manually
    // See https://github.com/bevyengine/bevy/issues/1700
    if kb.just_pressed(KeyCode::Escape) {
        match state.current() {
            GameState::Playing => {
                state.push(GameState::Paused).unwrap();
                kb.reset(KeyCode::Escape);
            }
            GameState::Paused => {
                state.pop().unwrap();
                kb.reset(KeyCode::Escape);
            }
            _ => {}
        }
    }
}

fn run_if_not_paused(mode: Res<State<GameState>>) -> ShouldRun {
    match mode.current() {
        GameState::Paused => ShouldRun::No,
        _ => ShouldRun::Yes,
    }
}

fn start_menu(egui_context: Res<EguiContext>) {
    egui::Window::new("Hello").show(egui_context.ctx(), |ui| {
        ui.label("world");
    });
}

fn stop_menu(_egui_context: Res<EguiContext>) {}
