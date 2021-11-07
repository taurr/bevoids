use bevy::{ecs::schedule::ShouldRun, log, prelude::*};
use bevy_kira_audio::AudioSource;
use smol_str::SmolStr;
use std::path::PathBuf;

/// Bevy plugin for loading a number of audio files.
#[derive(Debug)]
pub struct AudioAssetsPlugin<KEY> {
    _marker: core::marker::PhantomData<KEY>,
}

/// Insert as a resource to make the [AudioAssetsPlugin] load audio files during startup.
#[derive(Debug, Clone)]
pub struct AudioPaths<KEY> {
    base_path: Option<SmolStr>,
    keys_and_paths: Vec<(KEY, SmolStr)>,
}

#[derive(Debug, Clone)]
enum AudioInfo<KEY> {
    Loading {
        key: KEY,
        handle: Handle<AudioSource>,
    },
    Loaded {
        key: KEY,
        handle: Handle<AudioSource>,
    },
}

#[derive(Debug, Clone)]
pub struct AudioAssets<KEY> {
    info: Vec<AudioInfo<KEY>>,
}

impl<KEY> Default for AudioAssetsPlugin<KEY> {
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
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

impl<KEY> Default for AudioAssets<KEY> {
    fn default() -> Self {
        Self {
            info: Default::default(),
        }
    }
}

impl<KEY> AudioAssets<KEY>
where
    KEY: Copy + Eq + Sync + Send,
{
    pub fn with_audio_paths(audio_paths: &mut AudioPaths<KEY>, asset_server: &AssetServer) -> Self {
        Self {
            info: audio_paths
                .keys_and_paths
                .iter()
                .map(|(key, asset_path)| {
                    if let Some(ref mut asset_base_path) = audio_paths.base_path {
                        let mut pathbuf = PathBuf::from(asset_base_path.as_str());
                        pathbuf.push(asset_path.as_str());
                        let asset_handle = asset_server.load(pathbuf.as_path());
                        AudioInfo::Loading {
                            key: *key,
                            handle: asset_handle,
                        }
                    } else {
                        let asset_handle = asset_server.load(asset_path.as_str());
                        AudioInfo::Loading {
                            key: *key,
                            handle: asset_handle,
                        }
                    }
                })
                .collect(),
        }
    }

    #[allow(dead_code)]
    pub fn ready(&self) -> bool {
        self.info.iter().all(|info| match info {
            AudioInfo::Loading { .. } => false,
            AudioInfo::Loaded { .. } => true,
        })
    }

    #[allow(dead_code)]
    pub fn get(&self, key: KEY) -> Option<Handle<AudioSource>> {
        self.info.iter().find_map(|info| match info {
            AudioInfo::Loaded { key: k, handle, .. } if *k == key => Some(handle.clone()),
            AudioInfo::Loading { key: k, handle, .. } if *k == key => Some(handle.clone()),
            _ => None,
        })
    }
}

impl<KEY> Plugin for AudioAssetsPlugin<KEY>
where
    KEY: 'static + core::fmt::Debug + Copy + Eq + Sync + Send,
{
    #[allow(dead_code)]
    fn build(&self, app: &mut App) {
        app.add_startup_system(load_audio_assets::<KEY>)
            .add_system(monitor_audio_assets::<KEY>);
    }
}

/// [RunCriteria] detecting when all AudioSource files for a key has been loaded.
#[allow(dead_code)]
pub fn audio_has_loaded<KEY: 'static + Copy + Eq + Sync + Send>(
    assets: Res<AudioAssets<KEY>>,
) -> ShouldRun {
    match assets.ready() {
        true => ShouldRun::Yes,
        false => ShouldRun::No,
    }
}

/// [RunCriteria] for systems that should run while still loading audio sources.
#[allow(dead_code)]
pub fn audio_is_loading<KEY: 'static + Copy + Eq + Sync + Send>(
    assets: Res<AudioAssets<KEY>>,
) -> ShouldRun {
    match assets.ready() {
        true => ShouldRun::No,
        false => ShouldRun::Yes,
    }
}

pub fn load_audio_assets<KEY>(
    mut commands: Commands,
    //mut asset_storage: ResMut<Assets<AudioSource>>,
    asset_server: Res<AssetServer>,
    audio_paths: Option<ResMut<AudioPaths<KEY>>>,
    audio_assets: Option<Res<AudioAssets<KEY>>>,
) where
    KEY: 'static + Copy + Eq + Sync + Send,
{
    if let Some(mut audio_paths) = audio_paths {
        commands.insert_resource(AudioAssets::with_audio_paths(
            &mut audio_paths,
            &asset_server,
        ));
    } else if audio_assets.is_none() {
        commands.insert_resource(AudioAssets::<KEY>::default());
    }
}

pub fn monitor_audio_assets<KEY>(
    mut audio_event: EventReader<AssetEvent<AudioSource>>,
    mut audio_assets: ResMut<AudioAssets<KEY>>,
    asset_storage: Res<Assets<AudioSource>>,
) where
    KEY: 'static + core::fmt::Debug + Copy + Eq + Sync + Send,
{
    for ev in audio_event.iter() {
        match ev {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                if let Some((key, audio_info)) =
                    audio_assets.info.iter_mut().find_map(|i| match i {
                        AudioInfo::Loading { key, handle: h } if *h == *handle => Some((*key, i)),
                        AudioInfo::Loaded { key, handle: h, .. } if *h == *handle => {
                            Some((*key, i))
                        }
                        _ => None,
                    })
                {
                    if let Some(_audio) = asset_storage.get(handle.clone()) {
                        log::info!(?key, ?handle, "audio loaded");
                        *audio_info = AudioInfo::Loaded {
                            key,
                            handle: asset_storage.get_handle(handle),
                        };
                    }
                }
            }
            AssetEvent::Removed { handle } => {
                log::warn!(?handle, "audio removed");
            }
        }
    }
}
