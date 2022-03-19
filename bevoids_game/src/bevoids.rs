use bevoids_assets::{BackgroundAsset, EnumCount, SoundAsset};
use bevy::{core::FixedTimestep, ecs::{schedule::ShouldRun}, log, prelude::*};
use bevy_effects::{
    animation::{SpriteAnimationPlugin, SpriteAnimationEvent},
    despawn::{DespawnPlugin, FadeDespawn, FadeIn},
    sound::SoundEffectsPlugin,
};
use bevy_egui::{egui, EguiContext, EguiPlugin};
#[cfg(feature = "inspector")]
use bevy_inspector_egui::WorldInspectorPlugin;
use derive_more::Display;
use rand::Rng;

use crate::{
    bevoids::{
        highscore::{load_highscores, update_score_system, AddScoreEvent, Score},
        settings::Settings,
    },
    bounds::{GfxBounds, WinBoundsPlugin},
};

mod asteroids;
mod highscore;
mod hit_test;
mod laser;
mod movement;
mod player;
mod resources;
pub mod settings;
mod ui;

use {asteroids::*, hit_test::*, laser::*, movement::*, player::*, resources::*, ui::*};

#[derive(Debug, Display, Copy, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    MainMenu,
    HighScoreMenu,
    Playing,
    Paused,
    GameOver,
    NewHighScore,
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

        #[cfg(feature = "inspector")]
        app.add_plugin(WorldInspectorPlugin::new());

        // misc
        app.add_plugin(EguiPlugin)
            .add_plugin(DespawnPlugin::with_run_criteria(run_criteria_if_not_paused))
            .add_plugin(SoundEffectsPlugin::<SoundAsset>::default())
            .add_plugin(SpriteAnimationPlugin::default())
            .add_plugin(WinBoundsPlugin)
            .add_startup_system(egui_defaults_system)
            .add_system(capture_cursor_when_playing_system)
            .add_system(esc_to_pause_unpause_system)
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_criteria_if_not_paused)
                    .with_system(wrapping_linear_movement_system)
                    .with_system(non_wrapping_linear_movement_system),
            )
            .add_system(move_shadow_system)
            .add_system(spawn_asteroid_event_system)
            .add_system(asteroid_explosion_system);

        // introduce the state to its relevant stages
        app.insert_resource(State::new(GameState::MainMenu))
            .add_system_set_to_stage(CoreStage::PreUpdate, State::<GameState>::get_driver())
            .add_system_set_to_stage(CoreStage::Update, State::<GameState>::get_driver())
            .add_system_set_to_stage(CoreStage::PostUpdate, State::<GameState>::get_driver());

        setup_initialize(app);
        setup_mainmenu(app);
        setup_highscore(app);
        setup_playing(app);
        setup_paused(app);
        setup_gameover(app);
        setup_new_highscore(app);
    }
}

fn setup_initialize(app: &mut App) {
    app.add_startup_system(load_highscores)
        .add_startup_system(define_animations)
        .add_startup_system(change_background_system)
        .add_startup_system(spawn_menu_asteroids_system)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(10.0))
                .with_system(change_background_system),
        )
        .add_system_to_stage(CoreStage::PostUpdate, despawn_effect);
}

fn setup_mainmenu(app: &mut App) {
    let state = GameState::MainMenu;
    app.add_system_set(SystemSet::on_enter(state).with_system(spawn_menu_asteroids_system))
        .add_system_set(SystemSet::on_update(state).with_system(display_main_menu_system));
}

fn setup_highscore(app: &mut App) {
    let state = GameState::HighScoreMenu;
    app.add_system_set(SystemSet::on_enter(state).with_system(spawn_menu_asteroids_system))
        .add_system_set(SystemSet::on_update(state).with_system(display_highscore_menu_system));
}

fn setup_playing(app: &mut App) {
    let state = GameState::Playing;

    app.add_system_set(
        SystemSet::on_enter(state)
            .with_system(despawn_menu_asteroids_system)
            .with_system(prep_playingfield_system)
            .with_system(spawn_player_system)
            .with_system(spawn_asteroid_spawner_system),
    )
    .add_system_set(
        SystemSet::on_update(state)
            .with_system(display_playing_ui_system)
            .with_system(asteroid_spawner_system)
            .with_system(player_controls_system.label("input"))
            .with_system(laser_fired_system.after("input"))
            .with_system(laser_vs_asteroid_system.label("hittest"))
            .with_system(player_vs_asteroid_system.label("hittest"))
            .with_system(player_dead_system.after("hittest"))
            .with_system(update_score_system.after("hittest"))
            .with_system(shot_asteroid_system.after("hittest")),
    )
    .add_system_set(
        SystemSet::on_exit(state)
            .with_system(stop_thruster_sound_system)
            .with_system(despawn_asteroid_spawner_system),
    );
}

fn setup_paused(app: &mut App) {
    let state = GameState::Paused;
    app.add_system_set(SystemSet::on_update(state).with_system(display_paused_menu_system));
    // TODO: pause/unpause all timers for TextureAtlasSprite
}

fn setup_gameover(app: &mut App) {
    let state = GameState::GameOver;
    app.add_system_set(SystemSet::on_update(state).with_system(display_gameover_menu_system))
        .add_system_set(SystemSet::on_exit(state).with_system(clear_playingfield_system));
}

fn setup_new_highscore(app: &mut App) {
    let state = GameState::NewHighScore;
    app.add_system_set(SystemSet::on_update(state).with_system(display_new_highscore_menu_system))
        .add_system_set(SystemSet::on_exit(state).with_system(clear_playingfield_system));
}

fn esc_to_pause_unpause_system(
    mut kb: ResMut<Input<KeyCode>>,
    mut state: ResMut<State<GameState>>,
) {
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

fn run_criteria_if_not_paused(state: Res<State<GameState>>) -> ShouldRun {
    match state.current() {
        GameState::Paused => ShouldRun::No,
        _ => ShouldRun::Yes,
    }
}

fn capture_cursor_when_playing_system(state: Res<State<GameState>>, mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    let capture = match state.current() {
        GameState::Paused => false,
        GameState::Playing => true,
        _ => false,
    };
    window.set_cursor_lock_mode(capture);
    window.set_cursor_visibility(!capture);
}

fn egui_defaults_system(mut egui_context: ResMut<EguiContext>) {
    let ctx = egui_context.ctx_mut();
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
    fonts.family_and_size.insert(
        egui::TextStyle::Monospace,
        (egui::FontFamily::Monospace, 24.0),
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

fn spawn_menu_asteroids_system(
    settings: Res<Settings>,
    mut spawn_event: EventWriter<SpawnAsteroidEvent>,
    mut background_asteroids_query: Query<Entity, With<BackgroundAsteroid>>,
) {
    if background_asteroids_query.iter_mut().next().is_none() {
        let mut rng = rand::thread_rng();
        for _ in 0..settings.general.asteroids_in_start_menu {
            spawn_event.send(SpawnAsteroidEvent::new(
                rng.gen_range(settings.asteroid.size_min..settings.asteroid.size_max),
                None,
                true,
            ));
        }
    }
}

fn despawn_menu_asteroids_system(
    mut commands: Commands,
    mut background_asteroids_query: Query<Entity, With<BackgroundAsteroid>>,
) {
    // hide background asteroids
    for e in background_asteroids_query.iter_mut() {
        commands.entity(e).despawn_recursive();
    }
}

#[derive(Debug, Component)]
pub(crate) struct Background;

fn change_background_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    background_query: Query<Entity, With<Background>>,
    win_bounds: Res<GfxBounds>,
    settings: Res<Settings>,
) {
    let mut rng = rand::thread_rng();
    let bg = BackgroundAsset::iter()
        .nth(rng.gen_range(0..BackgroundAsset::COUNT - 1))
        .unwrap();

    if let Some(entity) = background_query.iter().next() {
        commands
            .entity(entity)
            .insert(FadeDespawn::new(settings.general.background_fade));
    }

    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load(bg),
            sprite: Sprite {
                custom_size: Some(win_bounds.size()),
                ..Default::default()
            },
            ..SpriteBundle::default()
        })
        .insert(Background)
        .insert(FadeIn::from(settings.general.background_fade));
}

fn prep_playingfield_system(mut commands: Commands) {
    // clear asteroid counter
    commands.insert_resource(AsteroidCounter::default());

    // reset score
    commands.insert_resource(Score::default());
}

fn clear_playingfield_system(
    mut commands: Commands,
    player_query: Query<Entity, With<Player>>,
    asteroids_query: Query<Entity, With<Asteroid>>,
) {
    player_query
        .iter()
        .for_each(|e| commands.entity(e).despawn_recursive());
    asteroids_query
        .iter()
        .for_each(|e| commands.entity(e).despawn_recursive());
}

#[derive(Debug,Default,Clone,Component)]
struct Effect;

fn despawn_effect(
    mut commands: Commands,
    mut anim_loop_event: EventReader<SpriteAnimationEvent>,
) {
    for SpriteAnimationEvent(entity) in anim_loop_event.iter() {
        commands.entity(*entity).despawn();
    }
}
