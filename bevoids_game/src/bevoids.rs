use bevy::{ecs::schedule::ShouldRun, log, prelude::*};
use bevy_asset_map::{BoundsPlugin, FontAssetMapPlugin, TextureAssetMapPlugin};
use bevy_effects::{
    animation::AnimationEffectPlugin, despawn::DespawnPlugin, sound::SoundEffectsPlugin,
};
use bevy_egui::{egui, EguiContext, EguiPlugin};
#[cfg(feature = "inspector")]
use bevy_inspector_egui::WorldInspectorPlugin;
use derive_more::{AsRef, Deref, Display, From, Into};

use crate::bevoids::{
    menu::{set_menu_background, start_menu},
    paused::display_paused_menu,
};

mod asteroids;
mod gameover;
mod hit_test;
mod laser;
mod menu;
mod movement;
mod paused;
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

        #[cfg(feature = "inspector")]
        app.add_plugin(WorldInspectorPlugin::new());

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
            .add_plugin(BoundsPlugin)
            .add_startup_system(set_egui_defaults.system())
            .add_system(capture_cursor_when_playing.system())
            .add_system(esc_to_pause_unpause.system())
            .add_system(
                wrapping_linear_movement
                    .system()
                    .chain(move_shadow.system())
                    .with_run_criteria(run_if_not_paused.system()),
            )
            .add_system(
                non_wrapping_linear_movement
                    .system()
                    .with_run_criteria(run_if_not_paused.system()),
            )
            .add_system(handle_spawn_asteroid.system())
            .add_system(handle_asteroid_explosion.system());

        // introduce the state to its relevant stages
        app.insert_resource(State::new(GameState::Initialize))
            .add_system_set_to_stage(CoreStage::PreUpdate, State::<GameState>::get_driver())
            .add_system_set_to_stage(CoreStage::Update, State::<GameState>::get_driver())
            .add_system_set_to_stage(CoreStage::PostUpdate, State::<GameState>::get_driver());

        setup_initialize(app);
        setup_menu(app);
        setup_playing(app);
        setup_paused(app);
        setup_gameover(app);
    }
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

fn setup_menu(app: &mut AppBuilder) {
    let state = GameState::Menu;

    app.add_plugin(EguiPlugin)
        .add_system_set(SystemSet::on_enter(state).with_system(set_menu_background.system()))
        .add_system_set(
            SystemSet::on_update(state)
                .with_system(restart_on_enter.system())
                .with_system(start_menu.system()),
        );
}

fn setup_playing(app: &mut AppBuilder) {
    let state = GameState::Playing;

    app.add_system_set(
        SystemSet::on_enter(state)
            .with_system(spawn_player.system())
            .with_system(spawn_asteroid_spawner.system())
            .with_system(reset_scoreboard.system()),
    )
    .add_system_set_to_stage(
        CoreStage::Update,
        SystemSet::on_update(state)
            .with_system(display_playing_scoreboard.system())
            .with_system(asteroid_spawner.system())
            .with_system(player_controls.system().label("input"))
            .with_system(handle_fire_laser.system().after("input"))
            .with_system(hittest_shot_vs_asteroid.system().label("hittest"))
            .with_system(hittest_player_vs_asteroid.system().label("hittest"))
            .with_system(handle_player_dead.system().after("hittest"))
            .with_system(update_scoreboard.system().after("hittest"))
            .with_system(handle_shot_asteroids.system().after("hittest")),
    )
    .add_system_set(
        SystemSet::on_exit(state)
            .with_system(stop_thruster_sounds.system())
            .with_system(despawn_asteroid_spawner.system()),
    );
}

fn setup_paused(app: &mut AppBuilder) {
    let state = GameState::Paused;
    app.add_system_set_to_stage(
        CoreStage::Update,
        SystemSet::on_update(state).with_system(display_paused_menu.system()),
    );
}

fn setup_gameover(app: &mut AppBuilder) {
    let state = GameState::GameOver;

    app.add_system_set_to_stage(
        CoreStage::Update,
        SystemSet::on_update(state)
            .with_system(restart_on_enter.system())
            .with_system(display_gameover_menu.system()),
    );
}

fn esc_to_pause_unpause(mut kb: ResMut<Input<KeyCode>>, mut state: ResMut<State<GameState>>) {
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

fn run_if_not_paused(state: Res<State<GameState>>) -> ShouldRun {
    match state.current() {
        GameState::Paused => ShouldRun::No,
        _ => ShouldRun::Yes,
    }
}

fn capture_cursor_when_playing(state: Res<State<GameState>>, mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    let capture = match state.current() {
        GameState::Paused => false,
        GameState::Playing => true,
        _ => false,
    };
    window.set_cursor_lock_mode(capture);
    window.set_cursor_visibility(!capture);
}

fn set_egui_defaults(egui_context: Res<EguiContext>) {
    let ctx = egui_context.ctx();
    let mut fonts = egui::FontDefinitions::default();
    fonts.family_and_size.insert(
        egui::TextStyle::Button,
        (egui::FontFamily::Proportional, 24.0),
    );
    fonts.family_and_size.insert(
        egui::TextStyle::Heading,
        (egui::FontFamily::Proportional, 36.0),
    );
    fonts.family_and_size.insert(
        egui::TextStyle::Body,
        (egui::FontFamily::Proportional, 24.0),
    );
    fonts.family_and_size.insert(
        egui::TextStyle::Small,
        (egui::FontFamily::Proportional, 14.0),
    );
    ctx.set_fonts(fonts);

    let mut visuals = egui::Visuals::dark();
    visuals.button_frame = true;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::none();
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_black_alpha(160);
    ctx.set_visuals(visuals);

    //let mut style: egui::Style = (*ctx.style()).clone();
    //style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    //ctx.set_style(style);
}
