use bevy::{ecs::schedule::ShouldRun, log, prelude::*};
use smol_str::SmolStr;
use std::path::PathBuf;

pub type Size = UVec2;

#[derive(Debug)]
pub struct TextureAtlasAssetsPlugin<KEY> {
    _marker: std::marker::PhantomData<KEY>,
}

#[derive(Debug, Clone)]
pub enum AtlasDef {
    Grid { columns: usize, rows: usize },
}

/// Insert as a resource to make the [TextureAssetsPlugin] load textures and collect sizes during startup.
#[derive(Debug, Clone)]
pub struct TextureAtlasPaths<KEY> {
    base_path: Option<SmolStr>,
    keys_and_paths: Vec<(KEY, SmolStr, AtlasDef)>,
}

#[derive(Debug, Clone)]
enum TextureAtlasInfo<KEY> {
    Loading {
        key: KEY,
        texture: Handle<Texture>,
        columns_rows: AtlasDef,
    },
    Loaded(TextureAtlasAssetInfo<KEY>),
}

/// Resouce for keeping track of a number of textures.
#[derive(Debug, Clone)]
pub struct TextureAtlasAssets<KEY>(Vec<TextureAtlasInfo<KEY>>);

/// Information on a tracked texture. Can be retrieved through the [TextureAssets] resource,
/// or received as an event.
#[derive(Debug, Clone)]
pub struct TextureAtlasAssetInfo<KEY> {
    pub key: KEY,
    pub size: Vec2,
    pub texture: Handle<Texture>,
    pub atlas: Handle<TextureAtlas>,
    pub columns_rows: AtlasDef,
}

impl<KEY> Default for TextureAtlasAssetsPlugin<KEY> {
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<KEY> TextureAtlasPaths<KEY> {
    #[allow(dead_code)]
    pub fn from_files<TP, T>(paths: T) -> Self
    where
        TP: Into<SmolStr>,
        T: IntoIterator<Item = (KEY, TP, AtlasDef)>,
    {
        Self {
            base_path: None,
            keys_and_paths: paths
                .into_iter()
                .map(|(key, value, columns_rows)| (key, value.into(), columns_rows))
                .collect(),
        }
    }

    #[allow(dead_code)]
    pub fn from_path_and_files<P, TP, T>(base_path: Option<P>, paths: T) -> Self
    where
        P: Into<SmolStr>,
        TP: Into<SmolStr>,
        T: IntoIterator<Item = (KEY, TP, AtlasDef)>,
    {
        Self {
            base_path: base_path.map(|p| p.into()),
            keys_and_paths: paths
                .into_iter()
                .map(|(key, value, columns_rows)| (key, value.into(), columns_rows))
                .collect(),
        }
    }
}

impl<KEY> Default for TextureAtlasAssets<KEY> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<KEY> TextureAtlasAssets<KEY>
where
    KEY: Copy + Eq + Sync + Send,
{
    pub fn with_texture_paths(
        texture_paths: &mut TextureAtlasPaths<KEY>,
        asset_server: &AssetServer,
    ) -> Self {
        Self(
            texture_paths
                .keys_and_paths
                .iter()
                .map(|(key, asset_path, columns_rows)| {
                    if let Some(ref mut asset_base_path) = texture_paths.base_path {
                        let mut pathbuf = PathBuf::from(asset_base_path.as_str());
                        pathbuf.push(asset_path.as_str());
                        let handle = asset_server.load(pathbuf.as_path());
                        TextureAtlasInfo::Loading {
                            key: *key,
                            texture: handle,
                            columns_rows: columns_rows.clone(),
                        }
                    } else {
                        let handle = asset_server.load(asset_path.as_str());
                        TextureAtlasInfo::Loading {
                            key: *key,
                            texture: handle,
                            columns_rows: columns_rows.clone(),
                        }
                    }
                })
                .collect(),
        )
    }

    #[allow(dead_code)]
    pub fn ready(&self) -> bool {
        self.0.iter().all(|info| match info {
            TextureAtlasInfo::Loading { .. } => false,
            TextureAtlasInfo::Loaded(..) => true,
        })
    }

    #[allow(dead_code)]
    pub fn get(&self, key: KEY) -> Option<&TextureAtlasAssetInfo<KEY>> {
        self.0.iter().find_map(|info| match info {
            TextureAtlasInfo::Loaded(info @ TextureAtlasAssetInfo { key: k, .. }) if *k == key => {
                Some(info)
            }
            _ => None,
        })
    }
}

impl<KEY> Plugin for TextureAtlasAssetsPlugin<KEY>
where
    KEY: 'static + core::fmt::Debug + Copy + Eq + Sync + Send,
{
    #[allow(dead_code)]
    fn build(&self, app: &mut App) {
        app.add_event::<TextureAtlasAssetInfo<KEY>>()
            .add_startup_system(load_texture_atlas_assets::<KEY>)
            .add_system(monitor_texture_atlas_assets::<KEY>);
    }
}

/// [RunCriteria] detecting when all textures for a key has been loaded.
#[allow(dead_code)]
pub fn texture_atlas_are_loaded<KEY: 'static + Copy + Eq + Sync + Send>(
    assets: Res<TextureAtlasAssets<KEY>>,
) -> ShouldRun {
    match assets.ready() {
        true => ShouldRun::Yes,
        false => ShouldRun::No,
    }
}

/// [RunCriteria] for systems that should run while still loading textures.
#[allow(dead_code)]
pub fn texture_atlas_are_loading<KEY: 'static + Copy + Eq + Sync + Send>(
    assets: Res<TextureAtlasAssets<KEY>>,
) -> ShouldRun {
    match assets.ready() {
        true => ShouldRun::No,
        false => ShouldRun::Yes,
    }
}

pub fn load_texture_atlas_assets<KEY>(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    texture_paths: Option<ResMut<TextureAtlasPaths<KEY>>>,
    texture_assets: Option<Res<TextureAtlasAssets<KEY>>>,
) where
    KEY: 'static + Copy + Eq + Sync + Send,
{
    if let Some(mut texture_paths) = texture_paths {
        commands.insert_resource(TextureAtlasAssets::with_texture_paths(
            &mut texture_paths,
            &asset_server,
        ));
    } else if texture_assets.is_none() {
        commands.insert_resource(TextureAtlasAssets::<KEY>::default());
    }
}

pub fn monitor_texture_atlas_assets<KEY>(
    mut texture_event: EventReader<AssetEvent<Texture>>,
    mut assets: ResMut<TextureAtlasAssets<KEY>>,
    mut events: EventWriter<TextureAtlasAssetInfo<KEY>>,
    textures: Res<Assets<Texture>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) where
    KEY: 'static + core::fmt::Debug + Copy + Eq + Sync + Send,
{
    for ev in texture_event.iter() {
        match ev {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                if let Some((key, columns_rows, texture_info)) =
                    assets.0.iter_mut().find_map(|i| match i {
                        TextureAtlasInfo::Loading {
                            key,
                            columns_rows,
                            texture: th,
                        } if *th == *handle => Some((*key, columns_rows.clone(), i)),
                        TextureAtlasInfo::Loaded(TextureAtlasAssetInfo {
                            key,
                            columns_rows,
                            texture: th,
                            ..
                        }) if *th == *handle => Some((*key, columns_rows.clone(), i)),
                        _ => None,
                    })
                {
                    if let Some(texture) = textures.get(handle.clone()) {
                        let texture_size = Size::new(texture.size.width, texture.size.height);
                        let (texture_atlas, tile_size) = match columns_rows {
                            AtlasDef::Grid { columns, rows } => {
                                let tile_size = Vec2::new(
                                    texture_size.x as f32 / columns as f32,
                                    texture_size.y as f32 / rows as f32,
                                );
                                (
                                    TextureAtlas::from_grid(
                                        handle.clone(),
                                        tile_size,
                                        columns,
                                        rows,
                                    ),
                                    tile_size,
                                )
                            }
                        };

                        let texture_atlas_handle = texture_atlases.add(texture_atlas);

                        let texture_asset_info = TextureAtlasAssetInfo {
                            key,
                            size: tile_size,
                            texture: textures.get_handle(handle),
                            atlas: texture_atlas_handle,
                            columns_rows,
                        };
                        *texture_info = TextureAtlasInfo::Loaded(texture_asset_info.clone());
                        log::info!(?key, ?texture_size, texture_handle=?texture_asset_info.texture, atlas_handle=?texture_asset_info.atlas, "texture atlas loaded");
                        events.send(texture_asset_info)
                    }
                }
            }
            AssetEvent::Removed { handle } => {
                if let Some(key) = assets.0.iter().find_map(|i| match i {
                    TextureAtlasInfo::Loading {
                        key, texture: th, ..
                    } if *th == *handle => Some(*key),
                    TextureAtlasInfo::Loaded(TextureAtlasAssetInfo {
                        key, texture: th, ..
                    }) if *th == *handle => Some(*key),
                    _ => None,
                }) {
                    log::warn!(?key, ?handle, "texture removed");
                }
            }
        }
    }
}
