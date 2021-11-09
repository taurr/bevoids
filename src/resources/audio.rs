use bevy::{ecs::schedule::ShouldRun, log, prelude::*};
use bevy_kira_audio::AudioSource;
use smol_str::SmolStr;
use std::path::PathBuf;

/// Bevy plugin for loading a number of audio files.
#[derive(Debug)]
pub struct AudioAssetMapPlugin<KEY>(core::marker::PhantomData<KEY>);

#[derive(Debug)]
pub struct AudioAssetMap<KEY>(Vec<AudioMapEntry<KEY>>);

/// Insert as a resource to make the [AudioAssetMapPlugin] load audio files during startup.
#[derive(Debug, Clone)]
pub struct AudioPaths<KEY> {
    keys_and_paths: Vec<(KEY, SmolStr)>,
    base_path: Option<SmolStr>,
}

#[derive(Debug, Clone)]
enum AudioMapEntry<KEY> {
    Loading {
        key: KEY,
        handle: Handle<AudioSource>,
    },
    Loaded {
        key: KEY,
        handle: Handle<AudioSource>,
    },
}

impl<KEY> Default for AudioAssetMapPlugin<KEY> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<KEY> Plugin for AudioAssetMapPlugin<KEY>
where
    KEY: 'static + core::fmt::Debug + Copy + Eq + Sync + Send,
{
    #[allow(dead_code)]
    fn build(&self, app: &mut App) {
        app.add_startup_system(load_audio_assets::<KEY>)
            .add_system(monitor_audio_assets::<KEY>);
    }
}

impl<KEY> Default for AudioAssetMap<KEY> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<KEY> AudioPaths<KEY> {
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
    pub fn with_base_path<P: Into<SmolStr>>(mut self, base_path: Option<P>) -> Self {
        if let Some(base_path) = base_path {
            self.base_path = Some(base_path.into());
        }
        self
    }
}

impl<KEY> AudioAssetMap<KEY>
where
    KEY: Clone + Eq + Send + Sync,
{
    pub fn with_audio_paths(audio_paths: &AudioPaths<KEY>, asset_server: &AssetServer) -> Self {
        Self(
            audio_paths
                .keys_and_paths
                .iter()
                .map(|(key, asset_path)| {
                    if let Some(ref asset_base_path) = audio_paths.base_path {
                        let mut pathbuf = PathBuf::from(asset_base_path.as_str());
                        pathbuf.push(asset_path.as_str());
                        AudioMapEntry::Loading {
                            key: key.clone(),
                            handle: asset_server.load(pathbuf.as_path()),
                        }
                    } else {
                        AudioMapEntry::Loading {
                            key: key.clone(),
                            handle: asset_server.load(asset_path.as_str()),
                        }
                    }
                })
                .collect(),
        )
    }

    #[allow(dead_code)]
    pub fn ready(&self) -> bool {
        self.0.iter().all(|info| match info {
            AudioMapEntry::Loading { .. } => false,
            AudioMapEntry::Loaded { .. } => true,
        })
    }

    #[allow(dead_code)]
    pub fn get(&self, key: &KEY) -> Option<Handle<AudioSource>> {
        self.0.iter().find_map(|info| match info {
            AudioMapEntry::Loaded { key: k, handle, .. }
            | AudioMapEntry::Loading { key: k, handle, .. }
                if *k == *key =>
            {
                Some(handle.clone())
            }
            _ => None,
        })
    }
}

pub fn load_audio_assets<KEY>(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    audio_paths: Option<Res<AudioPaths<KEY>>>,
    audio_asset_map: Option<Res<AudioAssetMap<KEY>>>,
) where
    KEY: 'static + Clone + Eq + Send + Sync,
{
    if let Some(audio_paths) = audio_paths {
        commands.insert_resource(AudioAssetMap::with_audio_paths(&audio_paths, &asset_server));
    } else if audio_asset_map.is_none() {
        commands.insert_resource(AudioAssetMap::<KEY>::default());
    }
}

pub fn monitor_audio_assets<KEY>(
    mut audio_event: EventReader<AssetEvent<AudioSource>>,
    mut audio_asset_map: ResMut<AudioAssetMap<KEY>>,
    audio_assets: Res<Assets<AudioSource>>,
) where
    KEY: 'static + core::fmt::Debug + Clone + Eq + Send + Sync,
{
    for ev in audio_event.iter() {
        match ev {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                update_audio_map(&mut audio_asset_map, handle, &audio_assets);
            }
            AssetEvent::Removed { handle } => warn_removed_audio(&audio_asset_map, handle),
        }
    }
}

fn update_audio_map<KEY>(
    audio_asset_map: &mut AudioAssetMap<KEY>,
    handle: &Handle<AudioSource>,
    audio_assets: &Assets<AudioSource>,
) where
    KEY: 'static + core::fmt::Debug + Clone + Eq + Send + Sync,
{
    if let Some((key, audio_info)) = audio_asset_map.0.iter_mut().find_map(|i| match i {
        AudioMapEntry::Loading { key, handle: h }
        | AudioMapEntry::Loaded { key, handle: h, .. }
            if *h == *handle =>
        {
            Some((key.clone(), i))
        }
        _ => None,
    }) {
        log::info!(?key, ?handle, "audio loaded");
        *audio_info = AudioMapEntry::Loaded {
            key,
            handle: audio_assets.get_handle(handle),
        };
    }
}

fn warn_removed_audio<KEY>(audio_asset_map: &AudioAssetMap<KEY>, audio_handle: &Handle<AudioSource>)
where
    KEY: 'static + core::fmt::Debug,
{
    if let Some(key) = audio_asset_map.0.iter().find_map(|i| match i {
        AudioMapEntry::Loading { key, handle } | AudioMapEntry::Loaded { key, handle }
            if *handle == *audio_handle =>
        {
            Some(key)
        }
        _ => None,
    }) {
        log::warn!(?key, ?audio_handle, "audio removed");
    }
}

/// [RunCriteria] detecting when all AudioSource files for a key has been loaded.
#[allow(dead_code)]
pub fn audio_has_loaded<KEY: 'static + Copy + Eq + Sync + Send>(
    audio_asset_map: Res<AudioAssetMap<KEY>>,
) -> ShouldRun {
    match audio_asset_map.ready() {
        true => ShouldRun::Yes,
        false => ShouldRun::No,
    }
}

/// [RunCriteria] for systems that should run while still loading audio sources.
#[allow(dead_code)]
pub fn audio_is_loading<KEY: 'static + Copy + Eq + Sync + Send>(
    audio_asset_map: Res<AudioAssetMap<KEY>>,
) -> ShouldRun {
    match audio_asset_map.ready() {
        true => ShouldRun::No,
        false => ShouldRun::Yes,
    }
}
