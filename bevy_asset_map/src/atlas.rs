use bevy::{ecs::schedule::ShouldRun, log, prelude::*};
use std::{marker::PhantomData, path::PathBuf};

pub type Size = UVec2;

pub struct AtlasAssetMapPlugin<KEY> {
    _marker: PhantomData<KEY>,
}

impl<KEY> Default for AtlasAssetMapPlugin<KEY> {
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<KEY> Plugin for AtlasAssetMapPlugin<KEY>
where
    KEY: 'static + core::fmt::Debug + Clone + Eq + Sync + Send,
{
    fn build(&self, app: &mut App) {
        app.add_event::<AtlasAssetInfo<KEY>>();
        app.add_system_set_to_stage(
            CoreStage::Update,
            SystemSet::new().with_system(monitor_atlas_assets::<KEY>),
        );
    }
}

/// Resource for keeping track of a number of textures.
#[derive(Debug)]
pub struct AtlasAssetMap<KEY>(Vec<AtlasMapEntry<KEY>>);

/// Information on a tracked texture. Can be retrieved through the [AtlasAssetMap] resource,
/// or received as an event.
#[derive(Debug, Clone)]
pub struct AtlasAssetInfo<KEY> {
    pub key: KEY,
    pub atlas: Handle<TextureAtlas>,
    pub texture: Handle<Image>,
    pub tile_size: Vec2,
    pub definition: AtlasDefinition,
}

/// Insert as a resource to make the [AtlasAssetMapPlugin] load/create textures and collect sizes during startup.
#[derive(Debug, Clone)]
pub struct TextureAtlasPaths<KEY> {
    keys_and_paths: Vec<(KEY, String, AtlasDefinition)>,
    base_path: Option<String>,
}

#[derive(Debug, Clone)]
pub enum AtlasDefinition {
    Grid { columns: usize, rows: usize },
}

#[derive(Debug, Clone)]
enum AtlasMapEntry<KEY> {
    Loading {
        key: KEY,
        texture: Handle<Image>,
        definition: AtlasDefinition,
    },
    Loaded(AtlasAssetInfo<KEY>),
}

impl<KEY> TextureAtlasPaths<KEY> {
    #[allow(dead_code)]
    #[must_use]
    pub fn from_files<T, TP>(paths: T) -> Self
    where
        T: IntoIterator<Item = (KEY, TP, AtlasDefinition)>,
        TP: Into<String>,
    {
        Self {
            keys_and_paths: paths
                .into_iter()
                .map(|(key, value, columns_rows)| (key, value.into(), columns_rows))
                .collect(),
            base_path: None,
        }
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn with_base_path<P>(mut self, base_path: P) -> Self
    where
        P: Into<String>,
    {
        self.base_path = Some(base_path.into());
        self
    }
}

impl<KEY> AtlasAssetMap<KEY>
where
    KEY: Clone + Eq + Sync + Send,
{
    #[must_use]
    pub fn with_texture_paths(
        texture_paths: &TextureAtlasPaths<KEY>,
        asset_server: &AssetServer,
    ) -> Self {
        Self(
            texture_paths
                .keys_and_paths
                .iter()
                .map(|(key, asset_path, columns_rows)| {
                    if let Some(ref asset_base_path) = texture_paths.base_path {
                        let mut pathbuf = PathBuf::from(asset_base_path.as_str());
                        pathbuf.push(asset_path.as_str());
                        let handle = asset_server.load(pathbuf.as_path());
                        AtlasMapEntry::Loading {
                            key: key.clone(),
                            texture: handle,
                            definition: columns_rows.clone(),
                        }
                    } else {
                        let handle = asset_server.load(asset_path.as_str());
                        AtlasMapEntry::Loading {
                            key: key.clone(),
                            texture: handle,
                            definition: columns_rows.clone(),
                        }
                    }
                })
                .collect(),
        )
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[allow(dead_code)]
    pub fn ready(&self) -> bool {
        self.0.iter().all(|entry| match entry {
            AtlasMapEntry::Loading { .. } => false,
            AtlasMapEntry::Loaded(..) => true,
        })
    }

    #[allow(dead_code)]
    pub fn get(&self, key: &KEY) -> Option<&AtlasAssetInfo<KEY>> {
        self.0.iter().find_map(|entry| match entry {
            AtlasMapEntry::Loaded(info @ AtlasAssetInfo { key: k, .. }) if *k == *key => Some(info),
            _ => None,
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = &AtlasAssetInfo<KEY>> {
        self.0.iter().filter_map(|e| match e {
            AtlasMapEntry::Loaded(info) => Some(info),
            _ => None,
        })
    }
}

pub fn monitor_atlas_assets<KEY>(
    mut texture_events: EventReader<AssetEvent<Image>>,
    mut atlas_info_event: EventWriter<AtlasAssetInfo<KEY>>,
    atlas_asset_map: Option<ResMut<AtlasAssetMap<KEY>>>,
    texture_assets: Res<Assets<Image>>,
    mut texture_atlas_assets: ResMut<Assets<TextureAtlas>>,
) where
    KEY: 'static + core::fmt::Debug + Clone + Send + Sync,
{
    if let Some(mut atlas_asset_map) = atlas_asset_map {
        for ev in texture_events.iter() {
            match ev {
                AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                    update_atlas_map(
                        &mut atlas_asset_map,
                        handle,
                        &texture_assets,
                        &mut texture_atlas_assets,
                        &mut atlas_info_event,
                    )
                }
                AssetEvent::Removed { handle } => {
                    warn_removed_atlas_texture(&atlas_asset_map, handle)
                }
            }
        }
    }
}

fn update_atlas_map<KEY>(
    atlas_asset_map: &mut AtlasAssetMap<KEY>,
    texture_handle: &Handle<Image>,
    texture_assets: &Assets<Image>,
    texture_atlas_assets: &mut Assets<TextureAtlas>,
    atlas_info_event: &mut EventWriter<AtlasAssetInfo<KEY>>,
) where
    KEY: 'static + core::fmt::Debug + Clone + Send + Sync,
{
    if let Some((key, definition, texture_info)) =
        atlas_asset_map.0.iter_mut().find_map(|i| match i {
            AtlasMapEntry::Loading {
                key,
                definition,
                texture,
            }
            | AtlasMapEntry::Loaded(AtlasAssetInfo {
                key,
                definition,
                texture,
                ..
            }) if *texture == *texture_handle => Some((key.clone(), definition.clone(), i)),
            _ => None,
        })
    {
        let texture = texture_assets.get_handle(texture_handle);
        let texture_size = {
            let texture = texture_assets
                .get(texture_handle)
                .expect("texture not found though just updated");
            Size::new(
                texture.texture_descriptor.size.width,
                texture.texture_descriptor.size.height,
            )
        };
        let (tile_size, atlas) = match definition {
            AtlasDefinition::Grid { columns, rows } => {
                let tile_size = Vec2::new(
                    texture_size.x as f32 / columns as f32,
                    texture_size.y as f32 / rows as f32,
                );
                (
                    tile_size,
                    texture_atlas_assets.add(TextureAtlas::from_grid(
                        texture_handle.clone(),
                        tile_size,
                        columns,
                        rows,
                    )),
                )
            }
        };

        log::info!(?key, ?texture_size, ?tile_size, texture_handle=?texture, atlas_handle=?atlas, "texture atlas loaded");
        let texture_asset_info = AtlasAssetInfo {
            key,
            tile_size,
            texture,
            atlas,
            definition,
        };
        *texture_info = AtlasMapEntry::Loaded(texture_asset_info.clone());
        atlas_info_event.send(texture_asset_info)
    }
}

fn warn_removed_atlas_texture<KEY>(
    atlas_asset_map: &AtlasAssetMap<KEY>,
    texture_handle: &Handle<Image>,
) where
    KEY: 'static + core::fmt::Debug,
{
    if let Some(key) = atlas_asset_map.0.iter().find_map(|i| match i {
        AtlasMapEntry::Loading { key, texture, .. }
        | AtlasMapEntry::Loaded(AtlasAssetInfo { key, texture, .. })
            if *texture == *texture_handle =>
        {
            Some(key)
        }
        _ => None,
    }) {
        log::warn!(?key, ?texture_handle, "atlas texture removed");
    }
}

/// [RunCriteria] detecting when all atlas textures for a key has been loaded.
#[allow(dead_code)]
pub fn atlas_are_loaded<KEY: 'static + Clone + Eq + Send + Sync>(
    atlas_asset_map: Res<AtlasAssetMap<KEY>>,
) -> ShouldRun {
    match atlas_asset_map.ready() {
        true => ShouldRun::Yes,
        false => ShouldRun::No,
    }
}

/// [RunCriteria] for systems that should run while still loading atlas textures.
#[allow(dead_code)]
pub fn atlas_are_loading<KEY: 'static + Clone + Eq + Send + Sync>(
    atlas_asset_map: Res<AtlasAssetMap<KEY>>,
) -> ShouldRun {
    match atlas_asset_map.ready() {
        true => ShouldRun::No,
        false => ShouldRun::Yes,
    }
}
