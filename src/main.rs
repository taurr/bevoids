#![allow(clippy::complexity)]

use std::f32::consts::PI;

use asset_helper::RelativeAssetLoader;
use asteroid_plugin::AsteroidPlugin;
use bevy::{
    //diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    log,
    prelude::*,
    sprite::SpriteSettings,
    utils::HashMap,
};
use derive_more::{Deref, DerefMut, Display, From, Into};
use hit_test::HitTestPlugin;
use parry2d::{
    bounding_volume::{BoundingSphere, AABB},
    math::Point,
};
use rand::Rng;
use structopt::StructOpt;
use thiserror::Error;

use crate::{
    fade_plugin::FadePlugin, movement_plugin::MovementPlugin, player_plugin::PlayerPlugin,
};

mod asset_helper;
mod asteroid_plugin;
mod constants;
mod fade_plugin;
mod hit_test;
mod movement_plugin;
mod player_plugin;

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(long)]
    assets: Option<String>,
}

#[derive(Debug, Display, Copy, Clone, Eq, PartialEq, Hash)]
enum GameState {
    Initialize,
    InGame,
}

struct Textures {
    pub spaceship: Handle<Texture>,
    pub flame: Handle<Texture>,
    pub shot: Handle<Texture>,
    pub asteroids: Vec<Handle<Texture>>,
    sizes: HashMap<Handle<Texture>, Vec2>,
}

#[derive(Debug, Error, Copy, Clone)]
enum AsteroidMaterialError {
    #[error("no asteroid materials available")]
    NoMaterialsAvailable,
}

struct AsteroidMaterials {
    sizes_by_material: HashMap<Handle<ColorMaterial>, Vec2>,
    materials: Vec<Handle<ColorMaterial>>,
}

#[derive(Debug, Component, Copy, Clone)]
struct Bounds {
    aabb: AABB,
    min_sphere: BoundingSphere,
}

#[derive(Debug, Component, Display)]
struct Despawn;

// TODO: scoring
// TODO: respawn player / lives / levels

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        //.add_plugin(LogDiagnosticsPlugin::default())
        //.add_plugin(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(SpriteSettings {
            frustum_culling_enabled: true,
        })
        // general systems
        .add_system_to_stage(CoreStage::PostUpdate, despawn.system())
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
        .add_plugin(HitTestPlugin)
        // add resources that are always available
        .insert_resource(Args::from_args())
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(WindowDescriptor {
            width: constants::WIN_WIDTH,
            height: constants::WIN_HEIGHT,
            ..Default::default()
        })
        .run();
}

fn despawn(mut commands: Commands, query: Query<Entity, With<Despawn>>) {
    for entity in query.iter() {
        log::debug!(?entity, "despawning");
        commands.entity(entity).despawn_recursive();
    }
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
    window.set_vsync(false);
    window.set_title(module_path!().into());

    commands.insert_resource(Bounds::from_window(window));

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
                log::trace!(texture=?handle.id, "texture created/modified");
                textures.capture_size(handle, &texture_assets);
                if textures.has_size_for_all() {
                    log::trace!("generating asteroid materials");
                    commands.insert_resource(AsteroidMaterials::from_textures(
                        &textures,
                        constants::ASTEROIDS_MAX_ACTIVE,
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

impl AsteroidMaterials {
    fn from_textures(
        textures: &Textures,
        max_asteroid_sprites: usize,
        material_assets: &mut Assets<ColorMaterial>,
    ) -> Self {
        let mut sizes_by_material = HashMap::default();
        let mut materials = Vec::new();

        let mut rng = rand::thread_rng();

        // pre-generate materials
        for _ in 0..max_asteroid_sprites {
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

    pub fn push(&mut self, material: Handle<ColorMaterial>) {
        if self.sizes_by_material.get(&material).is_some() {
            let mut rng = rand::thread_rng();
            self.materials
                .insert(rng.gen_range(0..=self.materials.len()), material);
        }
    }
}

fn min_aabb_radius(aabb: &AABB) -> f32 {
    let ext = aabb.half_extents();
    f32::cos(PI / 4.0) * f32::max(ext.x, ext.y)
}

impl Bounds {
    pub fn from_window(window: &Window) -> Self {
        Bounds::from_pos_and_size(Vec2::ZERO, Vec2::from((window.width(), window.height())))
    }

    pub fn from_pos_and_size(position: Vec2, size: Vec2) -> Bounds {
        let (w, h) = (size.x / 2., size.y / 2.);
        let aabb = {
            AABB::new(
                Point::from([position.x - w, position.y - h]),
                Point::from([position.x + w, position.y + h]),
            )
        };
        let min_sphere = BoundingSphere::new(aabb.center(), min_aabb_radius(&aabb));
        Self { aabb, min_sphere }
    }

    pub fn size(&self) -> Vec2 {
        let extents = self.aabb.extents();
        Vec2::from((extents.x, extents.y))
    }

    pub fn width(&self) -> f32 {
        self.aabb.extents().x
    }

    pub fn height(&self) -> f32 {
        self.aabb.extents().y
    }

    pub fn set_center(&mut self, position: &Vec2) {
        self.aabb = AABB::from_half_extents(
            Point::from([position.x, position.y]),
            self.aabb.half_extents(),
        );
        self.min_sphere = BoundingSphere::new(self.aabb.center(), min_aabb_radius(&self.aabb));
    }

    pub fn as_aabb(&self) -> AABB {
        self.aabb
    }

    pub fn as_min_sphere(&self) -> BoundingSphere {
        self.min_sphere
    }
}
