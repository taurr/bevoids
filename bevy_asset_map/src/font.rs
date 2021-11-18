use bevy::{ecs::schedule::ShouldRun, log, prelude::*};
use std::path::PathBuf;

#[derive(Debug)]
pub struct FontAssetMap<KEY>(Vec<FontMapEntry<KEY>>);

/// Insert as a resource to make the [FontAssetMapPlugin] load fonts during startup.
#[derive(Debug, Clone)]
pub struct FontPaths<KEY> {
    keys_and_paths: Vec<(KEY, String)>,
    base_path: Option<String>,
}

#[derive(Debug, Clone)]
enum FontMapEntry<KEY> {
    Loading { key: KEY, handle: Handle<Font> },
    Loaded { key: KEY, handle: Handle<Font> },
}

impl<KEY> FontPaths<KEY> {
    #[allow(dead_code)]
    pub fn from_files<TP: Into<String>, T: IntoIterator<Item = (KEY, TP)>>(paths: T) -> Self {
        Self {
            base_path: None,
            keys_and_paths: paths
                .into_iter()
                .map(|(key, value)| (key, value.into()))
                .collect(),
        }
    }

    #[allow(dead_code)]
    pub fn with_base_path<P: Into<String>>(mut self, base_path: P) -> Self {
        self.base_path = Some(base_path.into());
        self
    }
}

impl<KEY> FontAssetMap<KEY>
where
    KEY: Clone + Eq + Send + Sync,
{
    pub fn with_font_paths(font_paths: &FontPaths<KEY>, asset_server: &AssetServer) -> Self {
        Self(
            font_paths
                .keys_and_paths
                .iter()
                .map(|(key, asset_path)| {
                    if let Some(ref asset_base_path) = font_paths.base_path {
                        let mut pathbuf = PathBuf::from(asset_base_path.as_str());
                        pathbuf.push(asset_path.as_str());
                        FontMapEntry::Loading {
                            key: key.clone(),
                            handle: asset_server.load(pathbuf.as_path()),
                        }
                    } else {
                        FontMapEntry::Loading {
                            key: key.clone(),
                            handle: asset_server.load(asset_path.as_str()),
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
        self.0.iter().all(|info| match info {
            FontMapEntry::Loading { .. } => false,
            FontMapEntry::Loaded { .. } => true,
        })
    }

    #[allow(dead_code)]
    pub fn get(&self, key: &KEY) -> Option<Handle<Font>> {
        self.0.iter().find_map(|info| match info {
            FontMapEntry::Loaded { key: k, handle, .. }
            | FontMapEntry::Loading { key: k, handle, .. }
                if *k == *key =>
            {
                Some(handle.clone())
            }
            _ => None,
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = (&KEY, &Handle<Font>)> {
        self.0.iter().filter_map(|e| match e {
            FontMapEntry::Loaded { key, handle } => Some((key, handle)),
            _ => None,
        })
    }
}

pub fn monitor_font_assets<KEY>(
    mut font_event: EventReader<AssetEvent<Font>>,
    font_asset_map: Option<ResMut<FontAssetMap<KEY>>>,
    font_assets: Res<Assets<Font>>,
) where
    KEY: 'static + core::fmt::Debug + Clone + Eq + Send + Sync,
{
    if let Some(mut font_asset_map) = font_asset_map {
        for ev in font_event.iter() {
            match ev {
                AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                    update_font_map(&mut font_asset_map, handle, &font_assets);
                }
                AssetEvent::Removed { handle } => warn_removed_font(&font_asset_map, handle),
            }
        }
    }
}

fn update_font_map<KEY>(
    font_asset_map: &mut FontAssetMap<KEY>,
    handle: &Handle<Font>,
    font_assets: &Assets<Font>,
) where
    KEY: 'static + core::fmt::Debug + Clone + Eq + Send + Sync,
{
    if let Some((key, font_info)) = font_asset_map.0.iter_mut().find_map(|i| match i {
        FontMapEntry::Loading { key, handle: h } | FontMapEntry::Loaded { key, handle: h, .. }
            if *h == *handle =>
        {
            Some((key.clone(), i))
        }
        _ => None,
    }) {
        log::info!(?key, ?handle, "font loaded");
        *font_info = FontMapEntry::Loaded {
            key,
            handle: font_assets.get_handle(handle),
        };
    }
}

fn warn_removed_font<KEY>(font_asset_map: &FontAssetMap<KEY>, font_handle: &Handle<Font>)
where
    KEY: 'static + core::fmt::Debug,
{
    if let Some(key) = font_asset_map.0.iter().find_map(|i| match i {
        FontMapEntry::Loading { key, handle } | FontMapEntry::Loaded { key, handle }
            if *handle == *font_handle =>
        {
            Some(key)
        }
        _ => None,
    }) {
        log::warn!(?key, ?font_handle, "font removed");
    }
}

/// [RunCriteria] detecting when all Font files for a key has been loaded.
#[allow(dead_code)]
pub fn font_has_loaded<KEY: 'static + Copy + Eq + Sync + Send>(
    font_asset_map: Res<FontAssetMap<KEY>>,
) -> ShouldRun {
    match font_asset_map.ready() {
        true => ShouldRun::Yes,
        false => ShouldRun::No,
    }
}

/// [RunCriteria] for systems that should run while still loading fonts.
#[allow(dead_code)]
pub fn font_is_loading<KEY: 'static + Copy + Eq + Sync + Send>(
    font_asset_map: Res<FontAssetMap<KEY>>,
) -> ShouldRun {
    match font_asset_map.ready() {
        true => ShouldRun::No,
        false => ShouldRun::Yes,
    }
}
