use bevy::prelude::*;
use bevy_asset_map::AudioAssetMap;
use bevy_kira_audio::{Audio, AudioChannel, AudioPlugin};
use itertools::*;

#[derive(Clone)]
pub struct SoundEffectsPlugin<KEY> {
    default_volume: Option<Vec<(KEY, f32)>>,
    default_panning: Option<Vec<(KEY, f32)>>,
}

impl<KEY> Default for SoundEffectsPlugin<KEY>
where
    KEY: 'static + core::fmt::Debug + Clone + Send + Sync,
{
    fn default() -> Self {
        Self {
            default_volume: None,
            default_panning: None,
        }
    }
}

impl<KEY> SoundEffectsPlugin<KEY>
where
    KEY: 'static + core::fmt::Debug + Clone + Send + Sync,
{
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            default_volume: None,
            default_panning: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_volumes<I: IntoIterator<Item = (KEY, f32)>>(mut self, itt: I) -> Self {
        self.default_volume = Some(
            itt.into_iter()
                .map(|(key, x)| (key, x.clamp(0.0, 1.0)))
                .collect(),
        );
        self
    }

    #[allow(dead_code)]
    pub fn with_panning<I: IntoIterator<Item = (KEY, f32)>>(mut self, itt: I) -> Self {
        self.default_panning = Some(
            itt.into_iter()
                .map(|(key, x)| (key, x.clamp(0.0, 1.0)))
                .collect(),
        );
        self
    }
}

#[derive(Debug, Clone)]
pub enum SfxCmdEvent<KEY> {
    Play(PlaySfx<KEY>),
    Loop(LoopSfx<KEY>),
    Stop(StopSfx<KEY>),
    Pan(SetPanSfx<KEY>),
    Vol(SetVolSfx<KEY>),
}

impl<KEY> From<PlaySfx<KEY>> for SfxCmdEvent<KEY> {
    fn from(x: PlaySfx<KEY>) -> Self {
        SfxCmdEvent::Play(x)
    }
}

impl<KEY> From<LoopSfx<KEY>> for SfxCmdEvent<KEY> {
    fn from(x: LoopSfx<KEY>) -> Self {
        SfxCmdEvent::Loop(x)
    }
}

impl<KEY> From<StopSfx<KEY>> for SfxCmdEvent<KEY> {
    fn from(x: StopSfx<KEY>) -> Self {
        SfxCmdEvent::Stop(x)
    }
}

impl<KEY> From<SetPanSfx<KEY>> for SfxCmdEvent<KEY> {
    fn from(x: SetPanSfx<KEY>) -> Self {
        SfxCmdEvent::Pan(x)
    }
}

impl<KEY> From<SetVolSfx<KEY>> for SfxCmdEvent<KEY> {
    fn from(x: SetVolSfx<KEY>) -> Self {
        SfxCmdEvent::Vol(x)
    }
}

#[derive(Debug, Clone)]
pub struct PlaySfx<KEY> {
    key: KEY,
    panning: Option<f32>,
    volume: Option<f32>,
}
impl<KEY> PlaySfx<KEY> {
    #[allow(dead_code)]
    pub fn new(key: KEY) -> Self {
        Self {
            key,
            volume: None,
            panning: None,
        }
    }
    #[allow(dead_code)]
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = Some(volume.clamp(0., 1.));
        self
    }
    #[allow(dead_code)]
    pub fn with_panning(mut self, panning: f32) -> Self {
        self.panning = Some(panning.clamp(0., 1.));
        self
    }
}

#[derive(Debug, Clone)]
pub struct LoopSfx<KEY> {
    key: KEY,
    panning: Option<f32>,
    volume: Option<f32>,
}
impl<KEY> LoopSfx<KEY> {
    #[allow(dead_code)]
    pub fn new(key: KEY) -> Self {
        Self {
            key,
            volume: None,
            panning: None,
        }
    }
    #[allow(dead_code)]
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = Some(volume.clamp(0., 1.));
        self
    }
    #[allow(dead_code)]
    pub fn with_panning(mut self, panning: f32) -> Self {
        self.panning = Some(panning.clamp(0., 1.));
        self
    }
}

#[derive(Debug, Clone)]
pub struct StopSfx<KEY> {
    key: KEY,
}
impl<KEY> StopSfx<KEY> {
    #[allow(dead_code)]
    pub fn new(key: KEY) -> Self {
        Self { key }
    }
}

#[derive(Debug, Clone)]
pub struct SetPanSfx<KEY> {
    key: KEY,
    panning: f32,
}
impl<KEY> SetPanSfx<KEY> {
    #[allow(dead_code)]
    pub fn new(key: KEY, panning: f32) -> Self {
        Self {
            key,
            panning: panning.clamp(0., 1.),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SetVolSfx<KEY> {
    key: KEY,
    volume: f32,
}
impl<KEY> SetVolSfx<KEY> {
    #[allow(dead_code)]
    pub fn new(key: KEY, volume: f32) -> Self {
        Self {
            key,
            volume: volume.clamp(0., 1.),
        }
    }
}

impl<KEY> Plugin for SoundEffectsPlugin<KEY>
where
    KEY: 'static + Clone + Eq + core::hash::Hash + Send + Sync + ToString,
{
    fn build(&self, app: &mut App) {
        app.add_plugin(AudioPlugin)
            .add_event::<SfxCmdEvent<KEY>>()
            .insert_resource(self.clone())
            .add_startup_system(sound_effects_startup_system::<KEY>)
            .add_system_set(SystemSet::new().with_system(play_sound_effect_on_event::<KEY>));
    }
}

fn sound_effects_startup_system<KEY>(
    mut commands: Commands,
    plugin: ResMut<SoundEffectsPlugin<KEY>>,
    audio: Res<Audio>,
) where
    KEY: 'static + Clone + Eq + core::hash::Hash + Send + Sync + ToString,
{
    commands.remove_resource::<SoundEffectsPlugin<KEY>>();

    let resource = SoundEffectChannels(
        plugin
            .default_volume
            .iter()
            .flatten()
            .map(|(key, _)| key.clone())
            .chain(
                plugin
                    .default_panning
                    .iter()
                    .flatten()
                    .map(|(key, _)| key.clone()),
            )
            .unique()
            .map(|key| {
                (
                    key.clone(),
                    SoundEffectSetting {
                        channel: AudioChannel::new(key.to_string()),
                        default_volume: plugin
                            .default_volume
                            .iter()
                            .flatten()
                            .find(|(k, _)| *k == key)
                            .map_or(1.0, |(_, v)| *v),
                        default_panning: plugin
                            .default_panning
                            .iter()
                            .flatten()
                            .find(|(k, _)| *k == key)
                            .map_or(0.5, |(_, v)| *v),
                    },
                )
            })
            .collect(),
    );
    for (_, setting) in resource.0.iter() {
        audio.set_volume_in_channel(setting.default_volume, &setting.channel);
        audio.set_panning_in_channel(setting.default_panning, &setting.channel);
    }
    commands.insert_resource(resource);
}

#[derive(Debug, Clone)]
struct SoundEffectSetting {
    channel: AudioChannel,
    default_panning: f32,
    default_volume: f32,
}

struct SoundEffectChannels<KEY>(Vec<(KEY, SoundEffectSetting)>);

fn play_sound_effect_on_event<KEY>(
    mut cmd_events: EventReader<SfxCmdEvent<KEY>>,
    mut channels: ResMut<SoundEffectChannels<KEY>>,
    audio_asset_map: Res<AudioAssetMap<KEY>>,
    audio: Res<Audio>,
) where
    KEY: 'static + Clone + Eq + Send + Sync + ToString,
{
    for cmd in cmd_events.iter() {
        match cmd {
            SfxCmdEvent::Play(PlaySfx {
                key,
                volume,
                panning,
            }) => {
                execute_sfx(key, &mut channels, |setting| {
                    set_volume_and_panning(volume, panning, &audio, setting);
                    audio.play_in_channel(
                        audio_asset_map.get(key).expect("missing sound"),
                        &setting.channel,
                    );
                });
            }
            SfxCmdEvent::Loop(LoopSfx {
                key,
                volume,
                panning,
            }) => {
                execute_sfx(key, &mut channels, |setting| {
                    set_volume_and_panning(volume, panning, &audio, setting);
                    audio.play_looped_in_channel(
                        audio_asset_map.get(key).expect("missing sound"),
                        &setting.channel,
                    );
                });
            }
            SfxCmdEvent::Stop(StopSfx { key }) => {
                execute_sfx(key, &mut channels, |setting| {
                    audio.stop_channel(&setting.channel);
                });
            }
            SfxCmdEvent::Pan(SetPanSfx { key, panning }) => {
                execute_sfx(key, &mut channels, |setting| {
                    set_volume_and_panning(&None, &Some(*panning), &audio, setting);
                });
            }
            SfxCmdEvent::Vol(SetVolSfx { key, volume }) => {
                execute_sfx(key, &mut channels, |setting| {
                    set_volume_and_panning(&Some(*volume), &None, &audio, setting);
                });
            }
        }
    }
}

fn execute_sfx<KEY, T>(key: &KEY, channels: &mut SoundEffectChannels<KEY>, x: T)
where
    T: FnOnce(&SoundEffectSetting),
    KEY: 'static + Clone + Eq + Send + Sync + ToString,
{
    let setting = channels.0.iter().find(|(k, _)| *key == *k).map(|(_, s)| s);
    if let Some(setting) = setting {
        x(setting);
    } else {
        let setting = SoundEffectSetting {
            channel: AudioChannel::new(key.to_string()),
            default_panning: 0.5,
            default_volume: 1.0,
        };
        x(&setting);
        channels.0.push((key.clone(), setting));
    };
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
            setting.default_volume
        },
        &setting.channel,
    );

    audio.set_panning_in_channel(
        if let Some(panning) = panning {
            *panning
        } else {
            setting.default_panning
        },
        &setting.channel,
    );
}
