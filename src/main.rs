#![allow(clippy::complexity)]

use crate::{
    asteroid_plugin::split_asteroid,
    fade_plugin::{FadePlugin, Fadeout},
    movement_plugin::MovementPlugin,
    player_plugin::PlayerPlugin,
};
use asset_helper::RelativeAssetLoader;
use asteroid_plugin::{Asteroid, AsteroidPlugin};
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    log,
    prelude::*,
    sprite::{collide_aabb::collide, SpriteSettings},
    utils::HashMap,
};
use fade_plugin::Fadein;
use movement_plugin::{ShadowController, ShadowOf, Velocity};
use player_plugin::Bullet;
use rand::Rng;
use std::f32::consts::PI;
use structopt::StructOpt;

mod asset_helper;
mod asteroid_plugin;
mod fade_plugin;
mod movement_plugin;
mod player_plugin;

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(short, long)]
    assets: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum GameState {
    Initialize,
    InGame,
}

const WIN_WIDTH: f32 = 1024.;
const WIN_HEIGHT: f32 = 800.;

const ASTEROIDS_LEVEL_SPAWN: usize = 10;
const ASTEROIDS_PLAYER_SPAWN_DISTANCE: f32 = 200.;
const ASTEROIDS_MAX_ACTIVE: usize = 500;
const ASTEROID_Z_MIN: f32 = 100.;
const ASTEROID_Z_MAX: f32 = 200.;
const ASTEROID_MIN_SIZE: f32 = 20.;
const ASTEROID_MAX_SIZE: f32 = 150.;
const ASTEROID_MIN_SPEED: f32 = 25.;
const ASTEROID_MAX_SPEED: f32 = 125.;
const ASTEROID_FADEIN_SECONDS: f32 = 2.;
const ASTEROID_FADEOUT_BULLET_SECONDS: f32 = 0.15;
const ASTEROID_FADEOUT_PLAYER_SECONDS: f32 = 1.0;

const BULLET_RELATIVE_Z: f32 = -1.;
const BULLET_RELATIVE_Y: f32 = 20.;
const BULLET_MAX_SIZE: f32 = 25.;
const BULLET_SPEED: f32 = 500.;
const BULLET_LIFETIME_SECONDS: f32 = 1.5;
const BULLET_FADEOUT_SECONDS: f32 = 0.25;

const FLAME_RELATIVE_Z: f32 = -10.;
const FLAME_RELATIVE_Y: f32 = -32.;
const FLAME_WIDTH: f32 = 15.;
const FLAME_OPACITY: f32 = 1.;

const PLAYER_Z: f32 = 900.;
const PLAYER_MAX_SIZE: f32 = 50.;
const PLAYER_ACCELLERATION: f32 = 250.;
const PLAYER_DECCELLERATION: f32 = 100.;
const PLAYER_START_SPEED: f32 = 50.;
const PLAYER_MAX_SPEED: f32 = 800.;
const PLAYER_FADEOUT_SECONDS: f32 = 0.5;
const PLAYER_TURN_SPEED: f32 = 2. * PI;

// TODO: scoring
// TODO: respawn player / lives / levels
// TODO: player hit asteroid

pub struct Textures {
    pub spaceship: Handle<Texture>,
    pub flame: Handle<Texture>,
    pub shot: Handle<Texture>,
    pub asteroids: Vec<Handle<Texture>>,
    sizes: HashMap<Handle<Texture>, Vec2>,
}
impl Textures {
    pub fn from_path(asset_server: &AssetServer, assets_path: &Option<String>) -> Self {
        Self {
            spaceship: asset_server.load_relative(assets_path, "spaceship.png"),
            flame: asset_server.load_relative(assets_path, "flame.png"),
            shot: asset_server.load_relative(assets_path, "laser.png"),
            asteroids: (1..20)
                .map(|n| asset_server.attempt_relative(assets_path, &format!("asteroid_{}.png", n)))
                .filter_map(|x| x)
                .collect(),
            sizes: Default::default(),
        }
    }

    pub fn has_size_for_all(&self) -> bool {
        3 + self.asteroids.len() == self.sizes.len()
    }

    pub fn capture_size(&mut self, handle: &Handle<Texture>, texture_assets: &Assets<Texture>) {
        let texture = texture_assets.get(handle).unwrap();
        let size = Vec2::new(texture.size.width as f32, texture.size.height as f32);
        self.sizes.insert(handle.clone(), size);
    }

    pub fn get_size(&self, handle: &Handle<Texture>) -> Option<Vec2> {
        self.sizes.get(handle).map(Vec2::clone)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum AsteroidMaterialError {
    #[error("no asteroid materials available")]
    NoMaterialsAvailable,
}
pub struct AsteroidMaterials {
    sizes_by_material: HashMap<Handle<ColorMaterial>, Vec2>,
    materials: Vec<Handle<ColorMaterial>>,
}
impl AsteroidMaterials {
    fn from_textures(
        textures: &Textures,
        size: usize,
        material_assets: &mut Assets<ColorMaterial>,
    ) -> Self {
        let mut sizes_by_material = HashMap::default();
        let mut materials = Vec::new();

        let mut rng = rand::thread_rng();
        for _ in 0..size {
            let random_texture = textures
                .asteroids
                .get(rng.gen_range(0..textures.asteroids.len()))
                .unwrap()
                .clone();
            let size = textures.sizes.get(&random_texture).unwrap().clone();
            let color_material = material_assets.add(ColorMaterial {
                color: Color::WHITE,
                texture: Some(random_texture),
            });
            materials.push(color_material.clone());
            sizes_by_material.insert(color_material, size);
        }

        Self {
            sizes_by_material,
            materials,
        }
    }

    pub fn pop(&mut self) -> Result<(Handle<ColorMaterial>, Vec2), AsteroidMaterialError> {
        self.materials
            .pop()
            .map(|material| {
                let size = self.sizes_by_material.get(&material).unwrap().clone();
                (material, size)
            })
            .ok_or(AsteroidMaterialError::NoMaterialsAvailable)
    }
}

#[derive(Debug, Component, Copy, Clone)]
pub struct SpriteSize(pub Vec2);

#[derive(Debug, Component, Copy, Clone)]
pub struct WinSize(pub Vec2);
impl WinSize {
    fn from_window(window: &Window) -> Self {
        Self(Vec2::new(window.width(), window.height()))
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(SpriteSettings {
            frustum_culling_enabled: true,
        })
        // set the starting state
        .add_state(GameState::Initialize)
        .add_system_set(SystemSet::on_enter(GameState::Initialize).with_system(initialize.system()))
        .add_system_set(
            SystemSet::on_update(GameState::Initialize).with_system(collect_textures.system()),
        )
        // plugins
        .add_plugin(PlayerPlugin)
        .add_plugin(AsteroidPlugin)
        .add_plugin(MovementPlugin)
        .add_plugin(FadePlugin)
        // add resources that are always available
        .insert_resource(Args::from_args())
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(WindowDescriptor {
            width: WIN_WIDTH,
            height: WIN_HEIGHT,
            ..Default::default()
        })
        .add_system_set(
            SystemSet::on_update(GameState::InGame).with_system(shot_hit_asteroid.system()),
        )
        //.with_system(asteroid_hit_player.system())
        .run();
}

fn initialize(
    args: Res<Args>,
    mut commands: Commands,
    mut windows: ResMut<Windows>,
    asset_server: Res<AssetServer>,
) {
    log::info!("initializing game");
    let window = windows.get_primary_mut().unwrap();
    window.set_resizable(false);
    window.set_vsync(true);
    window.set_title(module_path!().into());

    commands.insert_resource(WinSize::from_window(window));

    // Spawns the camera
    commands
        .spawn()
        .insert_bundle(OrthographicCameraBundle::new_2d())
        //.insert(Timer::from_seconds(1.0, true))
        .insert(Transform::from_xyz(0.0, 0.0, 1000.0));

    commands.insert_resource(Textures::from_path(&asset_server, &args.assets));
}

fn collect_textures(
    mut commands: Commands,
    mut texture_event: EventReader<AssetEvent<Texture>>,
    mut textures: ResMut<Textures>,
    texture_assets: Res<Assets<Texture>>,
    mut material_assets: ResMut<Assets<ColorMaterial>>,
    mut state: ResMut<State<GameState>>,
) {
    for ev in texture_event.iter() {
        match ev {
            AssetEvent::Created { handle } => {
                log::trace!("texture created/modified {:?}", handle.id);
                textures.capture_size(handle, &texture_assets);
                if textures.has_size_for_all() {
                    log::trace!("generating asteroid materials");
                    commands.insert_resource(AsteroidMaterials::from_textures(
                        &textures,
                        ASTEROIDS_MAX_ACTIVE,
                        &mut material_assets,
                    ));

                    log::info!("starting game");
                    state.set(GameState::InGame).unwrap();
                }
            }
            AssetEvent::Modified { handle: _ } => {}
            AssetEvent::Removed { handle: _ } => {}
        }
    }
}

fn shot_hit_asteroid(
    bullet_query: Query<(Entity, &Transform, &SpriteSize), With<Bullet>>,
    asteroids_query: Query<
        (
            Entity,
            &Transform,
            &SpriteSize,
            Option<&Velocity>,
            Option<&ShadowOf>,
        ),
        (With<Asteroid>, Without<Fadeout>),
    >,
    controller_query: Query<
        (Entity, Option<&Velocity>),
        (With<Asteroid>, With<ShadowController>, Without<Fadeout>),
    >,
    shadows_query: Query<(Entity, &ShadowOf), With<Asteroid>>,
    win_size: Res<WinSize>,
    mut commands: Commands,
    mut materials: ResMut<AsteroidMaterials>,
) {
    let mut spent_bullets = vec![];
    let mut asteroids_hit = vec![];

    'bullet: for (bullet_entity, bullet_transform, bullet_size) in bullet_query.iter() {
        for (asteroid_entity, asteroid_transform, asteroid_size, velocity, shadowof) in
            asteroids_query.iter()
        {
            let (controller, velocity) = shadowof
                .and_then(|shadowof| {
                    controller_query
                        .iter()
                        .find(|(controller, _)| controller == &shadowof.0)
                })
                .unwrap_or((asteroid_entity, velocity));

            if asteroids_hit.contains(&controller) {
                continue;
            }

            if velocity.is_none() {
                log::warn!(
                    "no velocity on controller - it's likely a double hit on a stopped asteroid"
                );
                continue;
            }
            let velocity = velocity.unwrap();

            // TODO: take bullet orientation into account for collision check!
            if collide(
                bullet_transform.translation,
                bullet_size.0,
                asteroid_transform.translation,
                asteroid_size.0,
            )
            .is_some()
            {
                log::debug!("bullet hits asteroid");
                asteroids_hit.push(controller);
                spent_bullets.push(bullet_entity);

                split_asteroid(
                    &asteroid_size.0,
                    &asteroid_transform.translation,
                    velocity,
                    &win_size,
                    &mut materials,
                    &mut commands,
                );

                continue 'bullet;
            }
        }
    }

    for bullet in spent_bullets {
        commands.entity(bullet).despawn();
    }

    for asteroid in asteroids_hit {
        // TODO: once in a blue moon, we try to insert Fadeout on an entity that has been removed - why?

        // removing Asteroid component stops us from finding the asteroid again
        // removing the Velocity stops the asteroid movement
        // removing any FadeIn as it kinda conflicts with the FadeOut
        // adding FadeOut fades the asteroids, and despawns when done!

        // do remember the "shadows" as well as the controller
        for entity in shadows_query
            .iter()
            .filter(|(_, shadowof)| asteroid == shadowof.0)
            .map(|(entity, _)| entity)
        {
            commands
                .entity(entity)
                .remove_bundle::<(Asteroid, Fadein)>()
                .insert(Fadeout::from_secs_f32(ASTEROID_FADEOUT_BULLET_SECONDS));
        }
        commands
            .entity(asteroid)
            .remove_bundle::<(Asteroid, Velocity, Fadein)>()
            .insert(Fadeout::from_secs_f32(ASTEROID_FADEOUT_BULLET_SECONDS));
    }
}

/*
fn asteroid_hit_player(
    mut commands: Commands,
    asteroid_query: Query<
        (Entity, &Transform, &ScaledSize),
        (With<Asteroid>, Without<Fadeout>, Without<Fadein>),
    >,
    player_query: Query<(Entity, &Transform, &ScaledSize), With<Player>>,
) {
    let mut removed = vec![];

    for (asteroid_entity, asteroid_transform, asteroid_size) in asteroid_query.iter() {
        for (player_entity, player_transform, player_size) in player_query.iter() {
            if collide(
                asteroid_transform.translation,
                **asteroid_size,
                player_transform.translation,
                **player_size,
            )
            .is_some()
            {
                if !removed.contains(&asteroid_entity) {
                    removed.push(asteroid_entity);
                    commands
                        .entity(asteroid_entity)
                        .remove::<Asteroid>()
                        .remove::<Velocity>()
                        .insert(Fadeout::from_secs_f32(ASTEROID_FADEOUT_PLAYER_SECONDS));
                }
                if !removed.contains(&player_entity) {
                    commands
                        .entity(player_entity)
                        .remove::<Player>()
                        .remove::<Velocity>()
                        .insert(Fadeout::from_secs_f32(PLAYER_FADEOUT_SECONDS));
                    removed.push(player_entity);
                }
            }
        }
    }
}
*/
