use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

use bevy::{
    asset::{AssetPath, AssetPathId},
    prelude::*,
    utils::HashMap,
};
use bevy_kira_audio::{Audio, AudioChannel, AudioPlugin, AudioSource};

/// Plugin for playing SoundEffects by sending events.
///
/// `KEY` must be an enumeration type identifying the usable sounds.
/// Each sound will be played in its own channel, so overlapping sounds
/// are fully supported.
pub struct SoundEffectsPlugin<KEY> {
    _marker: PhantomData<KEY>,
}

/// Enumeration of events that may be sent to control SoundEffects.
#[derive(Debug)]
pub enum SfxCmdEvent<KEY> {
    Play(PlaySfx<KEY>),
    Loop(LoopSfx<KEY>),
    Stop(StopSfx<KEY>),
    SetPan(SetPanSfx<KEY>),
    SetVol(SetVolSfx<KEY>),
}

#[derive(Debug)]
pub struct SoundEffectSetting {
    channel: AudioChannel,
    pub panning: f32,
    pub volume: f32,
}

#[derive(Debug, Default)]
pub struct SoundEffectSettings(HashMap<AssetPathId, SoundEffectSetting>);

impl<KEY> Default for SoundEffectsPlugin<KEY>
where
    KEY: 'static + Send + Sync + Clone + Into<AssetPath<'static>>,
{
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<KEY> Plugin for SoundEffectsPlugin<KEY>
where
    KEY: 'static + Send + Sync + Clone + Into<AssetPath<'static>>,
{
    fn build(&self, app: &mut App) {
        app.add_event::<SfxCmdEvent<KEY>>()
            .add_plugin(AudioPlugin)
            .add_system_set(SystemSet::new().with_system(play_sound_effect_on_event_system::<KEY>));
    }
}

macro_rules! impl_evt_from {
    ($struct:ident, $enum:ident) => {
        impl<KEY> From<$struct<KEY>> for SfxCmdEvent<KEY>
        where
            KEY: 'static + Send + Sync + Clone + Into<AssetPath<'static>>,
        {
            fn from(x: $struct<KEY>) -> Self {
                SfxCmdEvent::$enum(x)
            }
        }
    };
}

impl_evt_from!(PlaySfx, Play);
impl_evt_from!(LoopSfx, Loop);
impl_evt_from!(StopSfx, Stop);
impl_evt_from!(SetPanSfx, SetPan);
impl_evt_from!(SetVolSfx, SetVol);

#[derive(Debug)]
pub struct PlaySfx<KEY> {
    key: KEY,
    panning: Option<f32>,
    volume: Option<f32>,
}
impl<KEY> PlaySfx<KEY>
where
    KEY: 'static + Send + Sync + Clone + Into<AssetPath<'static>>,
{
    #[allow(dead_code)]
    pub fn new(key: KEY) -> Self {
        Self {
            key,
            volume: None,
            panning: None,
        }
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = Some(volume.clamp(0., 1.));
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn with_panning(mut self, panning: f32) -> Self {
        self.panning = Some(panning.clamp(0., 1.));
        self
    }

    fn execute(
        &self,
        asset_server: &AssetServer,
        settings: &mut SoundEffectSettings,
        audio: &Audio,
    ) {
        let asset_path_id = self.key.clone().into().get_id();
        let settings = settings.get_mut_or_insert(asset_path_id);
        set_volume_and_panning(&self.volume, &self.panning, audio, settings);
        audio.play_in_channel(
            asset_server.load::<AudioSource, KEY>(self.key.clone()),
            &settings.channel,
        );
    }
}

#[derive(Debug)]
pub struct LoopSfx<KEY> {
    key: KEY,
    panning: Option<f32>,
    volume: Option<f32>,
}
impl<KEY> LoopSfx<KEY>
where
    KEY: 'static + Send + Sync + Clone + Into<AssetPath<'static>>,
{
    #[allow(dead_code)]
    #[must_use]
    pub fn new(key: KEY) -> Self {
        Self {
            key,
            volume: None,
            panning: None,
        }
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = Some(volume.clamp(0., 1.));
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn with_panning(mut self, panning: f32) -> Self {
        self.panning = Some(panning.clamp(0., 1.));
        self
    }

    fn execute(
        &self,
        asset_server: &AssetServer,
        settings: &mut SoundEffectSettings,
        audio: &Audio,
    ) {
        let asset_path_id = self.key.clone().into().get_id();
        let settings = settings.get_mut_or_insert(asset_path_id);
        set_volume_and_panning(&self.volume, &self.panning, audio, settings);
        audio.play_looped_in_channel(
            asset_server.load::<AudioSource, KEY>(self.key.clone()),
            &settings.channel,
        );
    }
}

#[derive(Debug)]
pub struct StopSfx<KEY> {
    key: KEY,
}
impl<KEY> StopSfx<KEY>
where
    KEY: 'static + Send + Sync + Clone + Into<AssetPath<'static>>,
{
    #[allow(dead_code)]
    pub fn new(key: KEY) -> Self {
        Self { key }
    }

    fn execute(&self, settings: &mut SoundEffectSettings, audio: &Audio) {
        let asset_path_id = self.key.clone().into().get_id();
        let settings = settings.get_mut_or_insert(asset_path_id);
        audio.stop_channel(&settings.channel);
    }
}

#[derive(Debug)]
pub struct SetPanSfx<KEY> {
    key: KEY,
    panning: f32,
}
impl<KEY> SetPanSfx<KEY>
where
    KEY: 'static + Send + Sync + Clone + Into<AssetPath<'static>>,
{
    #[allow(dead_code)]
    pub fn new(key: KEY, panning: f32) -> Self {
        Self {
            key,
            panning: panning.clamp(0., 1.),
        }
    }

    fn execute(&self, settings: &mut SoundEffectSettings, audio: &Audio) {
        let asset_path_id = self.key.clone().into().get_id();
        let settings = settings.get_mut_or_insert(asset_path_id);
        settings.panning = self.panning;
        audio.set_panning_in_channel(self.panning, &settings.channel);
    }
}

#[derive(Debug)]
pub struct SetVolSfx<KEY> {
    key: KEY,
    volume: f32,
}
impl<KEY> SetVolSfx<KEY>
where
    KEY: 'static + Send + Sync + Clone + Into<AssetPath<'static>>,
{
    #[allow(dead_code)]
    pub fn new(key: KEY, volume: f32) -> Self {
        Self {
            key,
            volume: volume.clamp(0., 1.),
        }
    }

    fn execute(&self, settings: &mut SoundEffectSettings, audio: &Audio) {
        let asset_path_id = self.key.clone().into().get_id();
        let settings = settings.get_mut_or_insert(asset_path_id);
        settings.volume = self.volume;
        audio.set_volume_in_channel(self.volume, &settings.channel);
    }
}

impl SoundEffectSettings {
    pub fn get_mut_or_insert(&mut self, asset_path_id: AssetPathId) -> &mut SoundEffectSetting {
        let asset_path_id_hash = {
            let mut hasher = DefaultHasher::new();
            asset_path_id.hash(&mut hasher);
            hasher.finish()
        };
        let settings = self
            .0
            .entry(asset_path_id)
            .or_insert_with(|| SoundEffectSetting {
                channel: AudioChannel::new(asset_path_id_hash.to_string()),
                volume: 1.0,
                panning: 0.5,
            });
        settings
    }
}

pub fn play_sound_effect_on_event_system<KEY>(
    mut events: EventReader<SfxCmdEvent<KEY>>,
    mut settings: ResMut<SoundEffectSettings>,
    asset_server: ResMut<AssetServer>,
    audio: Res<Audio>,
) where
    KEY: 'static + Send + Sync + Clone + Into<AssetPath<'static>>,
{
    for cmd in events.iter() {
        match cmd {
            SfxCmdEvent::Play(evt) => evt.execute(&asset_server, &mut settings, &audio),
            SfxCmdEvent::Loop(evt) => evt.execute(&asset_server, &mut settings, &audio),
            SfxCmdEvent::Stop(evt) => evt.execute(&mut settings, &audio),
            SfxCmdEvent::SetPan(evt) => evt.execute(&mut settings, &audio),
            SfxCmdEvent::SetVol(evt) => evt.execute(&mut settings, &audio),
        }
    }
}

fn set_volume_and_panning(
    volume: &Option<f32>,
    panning: &Option<f32>,
    audio: &Audio,
    setting: &SoundEffectSetting,
) {
    audio.set_volume_in_channel(
        if let Some(volume) = volume {
            *volume
        } else {
            setting.volume
        },
        &setting.channel,
    );

    audio.set_panning_in_channel(
        if let Some(panning) = panning {
            *panning
        } else {
            setting.panning
        },
        &setting.channel,
    );
}
