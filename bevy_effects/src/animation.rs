use bevy::prelude::*;
use bevy_asset_map::AtlasAssetMap;

pub struct AnimationEffectPlugin<KEY>(core::marker::PhantomData<KEY>);

#[derive(Debug, Clone)]
pub struct AnimationEffectEvent<KEY> {
    pub key: KEY,
    pub size: f32,
    pub position: Vec3,
    pub fps: f32,
}

impl<KEY> Default for AnimationEffectPlugin<KEY>
where
    KEY: 'static + Clone + Eq + Send + Sync,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<KEY> AnimationEffectPlugin<KEY>
where
    KEY: 'static + Clone + Eq + Send + Sync,
{
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self(Default::default())
    }
}

impl<KEY> Plugin for AnimationEffectPlugin<KEY>
where
    KEY: 'static + Clone + Eq + Send + Sync,
{
    fn build(&self, app: &mut App) {
        app.add_event::<AnimationEffectEvent<KEY>>().add_system_set(
            SystemSet::new()
                .with_system(start_animation_effect::<KEY>)
                .with_system(update_animate_effect::<KEY>),
        );
    }
}

#[derive(Component, Debug)]
struct AnimEffect;

fn start_animation_effect<KEY>(
    mut commands: Commands,
    mut animation_events: EventReader<AnimationEffectEvent<KEY>>,
    atlas_asset_map: Res<AtlasAssetMap<KEY>>,
) where
    KEY: 'static + Clone + Eq + Send + Sync,
{
    for AnimationEffectEvent {
        key,
        size,
        position,
        fps,
    } in animation_events.iter()
    {
        let atlas_info = atlas_asset_map.get(key).expect("texture atlas not present");
        let texture_atlas = atlas_info.atlas.clone();
        let scale = size / atlas_info.tile_size.max_element();
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas,
                transform: Transform::from_translation(*position).with_scale(Vec3::splat(scale)),
                ..Default::default()
            })
            .insert(AnimEffect)
            .insert(Timer::from_seconds(1. / fps, true));
    }
}

fn update_animate_effect<KEY>(
    mut commands: Commands,
    time: Res<Time>,
    atlas_assets: Res<Assets<TextureAtlas>>,
    mut query: Query<
        (
            Entity,
            &mut Timer,
            &mut TextureAtlasSprite,
            &Handle<TextureAtlas>,
        ),
        With<AnimEffect>,
    >,
) where
    KEY: 'static + Clone + Eq + Send + Sync,
{
    for (entity, mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
        if timer.tick(time.delta()).finished() {
            let texture_atlas = atlas_assets
                .get(texture_atlas_handle)
                .expect("texture atlas not found");
            sprite.index = ((sprite.index as usize + 1) % texture_atlas.textures.len()) as u32;

            if sprite.index == 0 {
                commands.entity(entity).despawn();
            }
        }
    }
}