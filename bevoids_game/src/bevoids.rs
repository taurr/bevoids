use bevy::{ecs::schedule::ShouldRun, log, prelude::*};
use bevy_asset_map::{BoundsPlugin, GfxBounds, TextureAssetMap, TextureAssetMapPlugin};
use bevy_effects::{
    animation::AnimationEffectPlugin,
    despawn::{DespawnPlugin, FadeDespawn, FadeIn},
    sound::SoundEffectsPlugin,
};
use bevy_egui::{egui, EguiContext, EguiPlugin};
#[cfg(feature = "inspector")]
use bevy_inspector_egui::WorldInspectorPlugin;
use derive_more::Display;
use rand::Rng;
use std::time::Duration;

use crate::bevoids::{
    highscore::{load_highscores, update_score, AddScoreEvent, Score},
    settings::Settings,
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
    Initialize,
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
        app.add_plugin(EguiPlugin)
            .add_plugin(DespawnPlugin::with_run_criteria(run_if_not_paused.system()))
            .add_plugin(AnimationEffectPlugin::<AnimationAtlas>::with_run_criteria(
                run_if_not_paused.system(),
            ))
            .add_plugin(SoundEffectsPlugin::<SoundEffect>::default())
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
        setup_mainmenu(app);
        setup_highscore(app);
        setup_playing(app);
        setup_paused(app);
        setup_gameover(app);
        setup_new_highscore(app);
    }
}

fn setup_initialize(app: &mut AppBuilder) {
    let state = GameState::Initialize;

    app.add_startup_system(load_highscores.system())
        .add_startup_system(load_general_textures.system())
        .add_startup_system(load_asteroid_textures.system())
        .add_startup_system(load_background_textures.system())
        .add_startup_system(load_animations.system())
        .add_startup_system(load_audio.system())
        .add_system_set(SystemSet::on_update(state).with_system(wait_for_resources.system()));
}

fn setup_mainmenu(app: &mut AppBuilder) {
    let state = GameState::MainMenu;

    app //.add_plugin(EguiPlugin)
        .add_system_set(SystemSet::on_enter(state).with_system(set_menu_background.system()))
        .add_system_set(SystemSet::on_update(state).with_system(display_main_menu.system()));
}

fn setup_highscore(app: &mut AppBuilder) {
    let state = GameState::HighScoreMenu;
    app.add_system_set(SystemSet::on_enter(state).with_system(set_menu_background.system()))
        .add_system_set(SystemSet::on_update(state).with_system(display_highscore_menu.system()));
}

fn setup_playing(app: &mut AppBuilder) {
    let state = GameState::Playing;

    app.add_system_set(
        SystemSet::on_enter(state)
            .with_system(prep_playingfield.system())
            .with_system(spawn_player.system())
            .with_system(spawn_asteroid_spawner.system()),
    )
    .add_system_set(
        SystemSet::on_update(state)
            .with_system(display_playing_ui.system())
            .with_system(asteroid_spawner.system())
            .with_system(player_controls.system().label("input"))
            .with_system(handle_fire_laser.system().after("input"))
            .with_system(hittest_shot_vs_asteroid.system().label("hittest"))
            .with_system(hittest_player_vs_asteroid.system().label("hittest"))
            .with_system(handle_player_dead.system().after("hittest"))
            .with_system(update_score.system().after("hittest"))
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
    app.add_system_set(SystemSet::on_update(state).with_system(display_paused_menu.system()));
}

fn setup_gameover(app: &mut AppBuilder) {
    let state = GameState::GameOver;

    app.add_system_set(SystemSet::on_update(state).with_system(display_gameover_menu.system()))
        .add_system_set(SystemSet::on_exit(state).with_system(clear_playingfield.system()));
}

fn setup_new_highscore(app: &mut AppBuilder) {
    let state = GameState::NewHighScore;

    app.add_system_set(
        SystemSet::on_update(state).with_system(display_new_highscore_menu.system()),
    )
    .add_system_set(SystemSet::on_exit(state).with_system(clear_playingfield.system()));
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

#[derive(Debug)]
pub(crate) struct Background;

pub(crate) fn spawn_background(
    background_asset_map: &TextureAssetMap<BackgroundTexture>,
    color_assets: &mut Assets<ColorMaterial>,
    background_query: &Query<Entity, With<Background>>,
    win_bounds: &GfxBounds,
    commands: &mut Commands,
    settings: &Settings,
) {
    let mut rng = rand::thread_rng();

    let bg_texture = background_asset_map
        .get(BackgroundTexture(
            rng.gen_range(0..background_asset_map.len()),
        ))
        .expect("no texture for background");
    let bg_material = color_assets.add(bg_texture.texture.clone().into());
    color_assets.get_mut(&bg_material).unwrap().color.set_a(0.0);
    let bg_size = bg_texture.size;
    let bg_scale = f32::max(
        win_bounds.width() / bg_size.x as f32,
        win_bounds.height() / bg_size.y as f32,
    );

    if let Some(entity) = background_query.iter().next() {
        commands
            .entity(entity)
            .insert(FadeDespawn::new(Duration::from_secs_f32(
                settings.general.background_fade_seconds,
            )));
    }
    commands
        .spawn_bundle(SpriteBundle {
            material: bg_material,
            transform: Transform {
                scale: Vec3::splat(bg_scale),
                ..Default::default()
            },
            ..SpriteBundle::default()
        })
        .insert(Background)
        .insert(FadeIn::from(Duration::from_secs_f32(
            settings.general.background_fade_seconds,
        )));
}

fn set_menu_background(
    mut commands: Commands,
    mut color_assets: ResMut<Assets<ColorMaterial>>,
    background_query: Query<Entity, With<Background>>,
    background_asset_map: Res<TextureAssetMap<BackgroundTexture>>,
    win_bounds: Res<GfxBounds>,
    settings: Res<Settings>,
    mut spawn_event: EventWriter<SpawnAsteroidEvent>,
    mut background_asteroids_query: Query<&mut Visible, With<BackgroundAsteroid>>,
) {
    spawn_background(
        &background_asset_map,
        &mut color_assets,
        &background_query,
        &win_bounds,
        &mut commands,
        &settings,
    );

    if background_asteroids_query.iter_mut().next().is_some() {
        for mut visible in background_asteroids_query.iter_mut() {
            visible.is_visible = true;
        }
    } else {
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

fn prep_playingfield(
    mut commands: Commands,
    mut background_asteroids_query: Query<&mut Visible, With<BackgroundAsteroid>>,
) {
    // hide background asteroids
    for mut visible in background_asteroids_query.iter_mut() {
        visible.is_visible = false;
    }

    // clear asteroid counter
    commands.insert_resource(AsteroidCounter::default());

    // reset score
    commands.insert_resource(Score::default());
}

fn clear_playingfield(
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
