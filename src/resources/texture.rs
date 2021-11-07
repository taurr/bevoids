use bevy::{ecs::schedule::ShouldRun, log, prelude::*};
use smol_str::SmolStr;
use std::path::PathBuf;

pub type Size = UVec2;

/// Bevy plugin for loading a number of textures and keep track of their sizes through the [TextureAssets] resource.
#[derive(Debug)]
pub struct TextureAssetsPlugin<KEY> {
    _marker: std::marker::PhantomData<KEY>,
}

/// Insert as a resource to make the [TextureAssetsPlugin] load textures and collect sizes during startup.
#[derive(Debug, Clone)]
pub struct TexturePaths<KEY> {
    base_path: Option<SmolStr>,
    keys_and_paths: Vec<(KEY, SmolStr)>,
}

#[derive(Debug, Clone)]
enum TextureInfo<KEY> {
    Loading { key: KEY, texture: Handle<Texture> },
    Loaded(TextureAssetInfo<KEY>),
}

/// Resouce for keeping track of a number of textures.
#[derive(Debug, Clone)]
pub struct TextureAssets<KEY>(Vec<TextureInfo<KEY>>);

/// Information on a tracked texture. Can be retrieved through the [TextureAssets] resource,
/// or received as an event.
#[derive(Debug, Clone)]
pub struct TextureAssetInfo<KEY> {
    pub key: KEY,
    pub texture: Handle<Texture>,
    pub size: Size,
}

impl<KEY> Default for TextureAssetsPlugin<KEY> {
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<KEY> TexturePaths<KEY> {
    #[allow(dead_code)]
    pub fn from_files<TP: Into<SmolStr>, T: IntoIterator<Item = (KEY, TP)>>(paths: T) -> Self {
        Self {
            base_path: None,
            keys_and_paths: paths
                .into_iter()
                .map(|(key, value)| (key, value.into()))
                .collect(),
        }
    }

    #[allow(dead_code)]
    pub fn from_path_and_files<P, TP, T>(base_path: Option<P>, paths: T) -> Self
    where
        P: Into<SmolStr>,
        TP: Into<SmolStr>,
        T: IntoIterator<Item = (KEY, TP)>,
    {
        Self {
            base_path: base_path.map(|p| p.into()),
            keys_and_paths: paths
                .into_iter()
                .map(|(key, value)| (key, value.into()))
                .collect(),
        }
    }
}

impl<KEY> Default for TextureAssets<KEY> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<KEY> TextureAssets<KEY>
where
    KEY: Copy + Eq + Sync + Send,
{
    pub fn with_texture_paths(
        texture_paths: &mut TexturePaths<KEY>,
        asset_server: &AssetServer,
    ) -> Self {
        Self(
            texture_paths
                .keys_and_paths
                .iter()
                .map(|(key, asset_path)| {
                    if let Some(ref mut asset_base_path) = texture_paths.base_path {
                        let mut pathbuf = PathBuf::from(asset_base_path.as_str());
                        pathbuf.push(asset_path.as_str());
                        let texture = asset_server.load(pathbuf.as_path());
                        TextureInfo::Loading { key: *key, texture }
                    } else {
                        let texture = asset_server.load(asset_path.as_str());
                        TextureInfo::Loading { key: *key, texture }
                    }
                })
                .collect(),
        )
    }

    #[allow(dead_code)]
    pub fn ready(&self) -> bool {
        self.0.iter().all(|info| match info {
            TextureInfo::Loading { .. } => false,
            TextureInfo::Loaded(..) => true,
        })
    }

    #[allow(dead_code)]
    pub fn get(&self, key: KEY) -> Option<&TextureAssetInfo<KEY>> {
        self.0.iter().find_map(|info| match info {
            TextureInfo::Loaded(info @ TextureAssetInfo { key: k, .. }) if *k == key => Some(info),
            _ => None,
        })
    }
}

impl<KEY> Plugin for TextureAssetsPlugin<KEY>
where
    KEY: 'static + core::fmt::Debug + Copy + Eq + Sync + Send,
{
    #[allow(dead_code)]
    fn build(&self, app: &mut App) {
        app.add_event::<TextureAssetInfo<KEY>>()
            .add_startup_system(load_texture_assets::<KEY>)
            .add_system(monitor_texture_assets::<KEY>);
    }
}

/// [RunCriteria] detecting when all textures for a key has been loaded.
#[allow(dead_code)]
pub fn textures_are_loaded<KEY: 'static + Copy + Eq + Sync + Send>(
    assets: Res<TextureAssets<KEY>>,
) -> ShouldRun {
    match assets.ready() {
        true => ShouldRun::Yes,
        false => ShouldRun::No,
    }
}

/// [RunCriteria] for systems that should run while still loading textures.
#[allow(dead_code)]
pub fn textures_are_loading<KEY: 'static + Copy + Eq + Sync + Send>(
    assets: Res<TextureAssets<KEY>>,
) -> ShouldRun {
    match assets.ready() {
        true => ShouldRun::No,
        false => ShouldRun::Yes,
    }
}

pub fn load_texture_assets<KEY>(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    texture_paths: Option<ResMut<TexturePaths<KEY>>>,
    texture_assets: Option<Res<TextureAssets<KEY>>>,
) where
    KEY: 'static + Copy + Eq + Sync + Send,
{
    if let Some(mut texture_paths) = texture_paths {
        commands.insert_resource(TextureAssets::with_texture_paths(
            &mut texture_paths,
            &asset_server,
        ));
    } else if texture_assets.is_none() {
        commands.insert_resource(TextureAssets::<KEY>::default());
    }
}

pub fn monitor_texture_assets<KEY>(
    mut texture_event: EventReader<AssetEvent<Texture>>,
    mut assets: ResMut<TextureAssets<KEY>>,
    mut events: EventWriter<TextureAssetInfo<KEY>>,
    textures: Res<Assets<Texture>>,
) where
    KEY: 'static + core::fmt::Debug + Copy + Eq + Sync + Send,
{
    for ev in texture_event.iter() {
        match ev {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                if let Some((key, texture_info)) = assets.0.iter_mut().find_map(|i| match i {
                    TextureInfo::Loading {
                        key, texture: th, ..
                    } if *th == *handle => Some((*key, i)),
                    TextureInfo::Loaded(TextureAssetInfo {
                        key, texture: th, ..
                    }) if *th == *handle => Some((*key, i)),
                    _ => None,
                }) {
                    if let Some(texture) = textures.get(handle.clone()) {
                        let size = Size::new(texture.size.width, texture.size.height);
                        let texture_asset_info = TextureAssetInfo {
                            key,
                            texture: textures.get_handle(handle),
                            size,
                        };
                        *texture_info = TextureInfo::Loaded(texture_asset_info.clone());
                        log::info!(?key, ?size, texture_handle=?texture_asset_info.texture, "texture loaded");
                        events.send(texture_asset_info)
                    }
                }
            }
            AssetEvent::Removed { handle } => {
                if let Some(key) = assets.0.iter().find_map(|i| match i {
                    TextureInfo::Loading {
                        key, texture: th, ..
                    } if *th == *handle => Some(*key),
                    TextureInfo::Loaded(TextureAssetInfo {
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
