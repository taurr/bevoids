use bevy::{ecs::schedule::ShouldRun, log, prelude::*};
use smol_str::SmolStr;
use std::path::PathBuf;

pub type Size = UVec2;

/// Bevy plugin for loading a number of textures and keep track of their sizes through the [TextureAssetMap] resource.
#[derive(Debug)]
pub struct TextureAssetMapPlugin<KEY>(std::marker::PhantomData<KEY>);

/// Resouce for keeping track of a number of textures.
#[derive(Debug)]
pub struct TextureAssetMap<KEY>(Vec<TextureMapEntry<KEY>>);

/// Information on a tracked texture. Can be retrieved through the [TextureAssetMap] resource,
/// or received as an event.
#[derive(Debug, Clone)]
pub struct TextureAssetInfo<KEY> {
    pub key: KEY,
    pub texture: Handle<Texture>,
    pub size: Size,
}

/// Insert as a resource to make the [TextureAssetMapPlugin] load textures and collect sizes during startup.
#[derive(Debug, Clone)]
pub struct TexturePaths<KEY> {
    keys_and_paths: Vec<(KEY, SmolStr)>,
    base_path: Option<SmolStr>,
}

#[derive(Debug, Clone)]
enum TextureMapEntry<KEY> {
    Loading { key: KEY, texture: Handle<Texture> },
    Loaded(TextureAssetInfo<KEY>),
}

impl<KEY> Default for TextureAssetMapPlugin<KEY> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<KEY> Plugin for TextureAssetMapPlugin<KEY>
where
    KEY: 'static + core::fmt::Debug + Clone + Eq + Send + Sync,
{
    #[allow(dead_code)]
    fn build(&self, app: &mut App) {
        app.add_event::<TextureAssetInfo<KEY>>()
            .add_startup_system(load_texture_assets::<KEY>)
            .add_system(monitor_texture_assets::<KEY>);
    }
}

impl<KEY> TexturePaths<KEY> {
    #[allow(dead_code)]
    pub fn from_files<TP: Into<SmolStr>, T: IntoIterator<Item = (KEY, TP)>>(paths: T) -> Self {
        Self {
            keys_and_paths: paths
                .into_iter()
                .map(|(key, value)| (key, value.into()))
                .collect(),
            base_path: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_base_path<P>(mut self, base_path: Option<P>) -> Self
    where
        P: Into<SmolStr>,
    {
        if let Some(base_path) = base_path {
            self.base_path = Some(base_path.into());
        }
        self
    }
}

impl<KEY> Default for TextureAssetMap<KEY> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<KEY> TextureAssetMap<KEY>
where
    KEY: Clone + Eq + Send + Sync,
{
    pub fn with_texture_paths(
        texture_paths: &TexturePaths<KEY>,
        asset_server: &AssetServer,
    ) -> Self {
        Self(
            texture_paths
                .keys_and_paths
                .iter()
                .map(|(key, asset_path)| {
                    if let Some(ref asset_base_path) = texture_paths.base_path {
                        let mut pathbuf = PathBuf::from(asset_base_path.as_str());
                        pathbuf.push(asset_path.as_str());
                        let texture = asset_server.load(pathbuf.as_path());
                        TextureMapEntry::Loading {
                            key: key.clone(),
                            texture,
                        }
                    } else {
                        let texture = asset_server.load(asset_path.as_str());
                        TextureMapEntry::Loading {
                            key: key.clone(),
                            texture,
                        }
                    }
                })
                .collect(),
        )
    }

    #[allow(dead_code)]
    pub fn ready(&self) -> bool {
        self.0.iter().all(|info| match info {
            TextureMapEntry::Loading { .. } => false,
            TextureMapEntry::Loaded(..) => true,
        })
    }

    #[allow(dead_code)]
    pub fn get(&self, key: KEY) -> Option<&TextureAssetInfo<KEY>> {
        self.0.iter().find_map(|info| match info {
            TextureMapEntry::Loaded(info @ TextureAssetInfo { key: k, .. }) if *k == key => {
                Some(info)
            }
            _ => None,
        })
    }
}

pub fn load_texture_assets<KEY>(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    texture_paths: Option<Res<TexturePaths<KEY>>>,
    texture_asset_map: Option<Res<TextureAssetMap<KEY>>>,
) where
    KEY: 'static + Clone + Eq + Send + Sync,
{
    if let Some(texture_paths) = texture_paths {
        commands.insert_resource(TextureAssetMap::with_texture_paths(
            &texture_paths,
            &asset_server,
        ));
    } else if texture_asset_map.is_none() {
        commands.insert_resource(TextureAssetMap::<KEY>::default());
    }
}

pub fn monitor_texture_assets<KEY>(
    mut texture_events: EventReader<AssetEvent<Texture>>,
    mut texture_info_event: EventWriter<TextureAssetInfo<KEY>>,
    mut texture_asset_map: ResMut<TextureAssetMap<KEY>>,
    texture_assets: Res<Assets<Texture>>,
) where
    KEY: 'static + core::fmt::Debug + Clone + Eq + Send + Sync,
{
    for ev in texture_events.iter() {
        match ev {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => update_texture_map(
                &mut texture_asset_map,
                &texture_assets,
                handle,
                &mut texture_info_event,
            ),
            AssetEvent::Removed { handle } => {
                warn_removed_texture(&texture_asset_map, handle);
            }
        }
    }
}

fn update_texture_map<KEY>(
    texture_asset_map: &mut TextureAssetMap<KEY>,
    texture_assets: &Assets<Texture>,
    texture_handle: &Handle<Texture>,
    texture_info_event: &mut EventWriter<TextureAssetInfo<KEY>>,
) where
    KEY: 'static + core::fmt::Debug + Clone + Eq + Send + Sync,
{
    if let Some((key, texture_info)) = texture_asset_map.0.iter_mut().find_map(|i| match i {
        TextureMapEntry::Loading { key, texture, .. }
        | TextureMapEntry::Loaded(TextureAssetInfo { key, texture, .. })
            if *texture == *texture_handle =>
        {
            Some((key.clone(), i))
        }
        _ => None,
    }) {
        let texture_asset_info = {
            let texture = texture_assets
                .get(texture_handle)
                .expect("texture not found though just updated");
            let size = Size::new(texture.size.width, texture.size.height);
            log::info!(?key, ?size, texture_handle=?texture_handle, "texture loaded");
            TextureAssetInfo {
                key,
                texture: texture_assets.get_handle(texture_handle),
                size,
            }
        };
        *texture_info = TextureMapEntry::Loaded(texture_asset_info.clone());
        texture_info_event.send(texture_asset_info)
    }
}

fn warn_removed_texture<KEY>(
    texture_asset_map: &TextureAssetMap<KEY>,
    texture_handle: &Handle<Texture>,
) where
    KEY: 'static + core::fmt::Debug,
{
    if let Some(key) = texture_asset_map.0.iter().find_map(|i| match i {
        TextureMapEntry::Loading { key, texture, .. }
        | TextureMapEntry::Loaded(TextureAssetInfo { key, texture, .. })
            if *texture == *texture_handle =>
        {
            Some(key)
        }
        _ => None,
    }) {
        log::warn!(?key, ?texture_handle, "texture removed");
    }
}

/// [RunCriteria] detecting when all textures for a key has been loaded.
#[allow(dead_code)]
pub fn textures_are_loaded<KEY: 'static + Copy + Eq + Sync + Send>(
    texture_asset_map: Res<TextureAssetMap<KEY>>,
) -> ShouldRun {
    match texture_asset_map.ready() {
        true => ShouldRun::Yes,
        false => ShouldRun::No,
    }
}

/// [RunCriteria] for systems that should run while still loading textures.
#[allow(dead_code)]
pub fn textures_are_loading<KEY: 'static + Copy + Eq + Sync + Send>(
    texture_asset_map: Res<TextureAssetMap<KEY>>,
) -> ShouldRun {
    match texture_asset_map.ready() {
        true => ShouldRun::No,
        false => ShouldRun::Yes,
    }
}