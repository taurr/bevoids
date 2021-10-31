use bevy::{
    //diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    log,
    prelude::*,
    utils::HashMap,
};
use rand::Rng;
use thiserror::Error;

use crate::{assets::LoadRelative, constants, Args, GameState};

pub(crate) struct TextureLoaderPlugin;

pub(crate) struct Textures {
    pub spaceship: Handle<Texture>,
    pub flame: Handle<Texture>,
    pub shot: Handle<Texture>,
    pub asteroids: Vec<Handle<Texture>>,
    sizes: HashMap<Handle<Texture>, Vec2>,
}

#[derive(Debug, Error, Copy, Clone)]
pub(crate) enum AsteroidMaterialError {
    #[error("no asteroid materials available")]
    NoMaterialsAvailable,
}

pub(crate) struct AsteroidMaterials {
    sizes_by_material: HashMap<Handle<ColorMaterial>, Vec2>,
    materials: Vec<Handle<ColorMaterial>>,
}

impl Plugin for TextureLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PostStartup, texture_initialize.system())
            .add_system_set(
                SystemSet::on_update(GameState::Initialize).with_system(collect_textures.system()),
            );
    }
}

fn texture_initialize(args: Res<Args>, mut commands: Commands, asset_server: Res<AssetServer>) {
    log::debug!("loading textures");
    commands.insert_resource(Textures::from_args(&asset_server, &args));
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
            AssetEvent::Modified { handle: _ } | AssetEvent::Removed { handle: _ } => {}
        }
    }
}

impl Textures {
    pub fn from_args(asset_server: &AssetServer, args: &Args) -> Self {
        Self {
            spaceship: asset_server
                .load_relative(&"gfx/spaceship.png", args)
                .expect("missing texture"),
            flame: asset_server
                .load_relative(&"gfx/flame.png", args)
                .expect("missing texture"),
            shot: asset_server
                .load_relative(&"gfx/laser.png", args)
                .expect("missing texture"),
            asteroids: (1..20)
                .filter_map(|n| {
                    asset_server
                        .load_relative(&format!("gfx/asteroid_{}.png", n), args)
                        .ok()
                })
                .collect(),
            sizes: HashMap::default(),
        }
    }

    pub fn has_size_for_all(&self) -> bool {
        3 + self.asteroids.len() == self.sizes.len()
    }

    pub fn capture_size(&mut self, handle: &Handle<Texture>, texture_assets: &Assets<Texture>) {
        if let Some(texture) = texture_assets.get(handle) {
            let size = Vec2::new(texture.size.width as f32, texture.size.height as f32);
            self.sizes.insert(handle.clone(), size);
        }
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
            if let Some(random_texture) = textures
                .asteroids
                .get(rng.gen_range(0..textures.asteroids.len()))
                .cloned()
            {
                if let Some(size) = textures.sizes.get(&random_texture).copied() {
                    let color_material = material_assets.add(ColorMaterial {
                        color: Color::WHITE,
                        texture: Some(random_texture),
                    });
                    materials.push(color_material.clone());
                    sizes_by_material.insert(color_material, size);
                }
            }
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
