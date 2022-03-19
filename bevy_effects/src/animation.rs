use bevy::{
    asset::{AssetPath, AssetPathId},
    ecs::system::EntityCommands,
    prelude::*,
    utils::HashMap,
};

/// Descripe all the initial settings for an `TextureAtlasSprite`
/// to be spawned as a Sprite animation.
#[derive(Debug, Clone)]
pub struct SpriteAnimation {
    pub position: Vec3,
    pub rotation: Quat,
    pub flip_x: bool,
    pub flip_y: bool,
    pub fps: f32,
    pub tint: Color,
    pub size: Option<Vec2>,
}

#[derive(Debug, Default)]
pub struct TextureAtlasMap {
    map: HashMap<AssetPathId, Handle<TextureAtlas>>,
}

/// Event raised whenever a `TextureAtlasSprite` animation loops around
#[derive(Debug, Clone, Copy)]
pub struct SpriteAnimationEvent(pub Entity);

/// Plugin for handling `TextureAtlasSprite` animations
#[derive(Debug, Default)]
pub struct SpriteAnimationPlugin;

pub trait SpawnSpriteAnimation<'w, 's> {
    fn spawn_sprite_animation<'a>(
        &'a mut self,
        texture_atlas_handle: &Handle<TextureAtlas>,
        sprite_animation: SpriteAnimation,
    ) -> EntityCommands<'w, 's, 'a>;
}

impl Default for SpriteAnimation {
    fn default() -> Self {
        Self {
            position: Default::default(),
            rotation: Default::default(),
            flip_x: false,
            flip_y: false,
            fps: 30.0,
            tint: Color::WHITE,
            size: None,
        }
    }
}

impl<'w, 's> SpawnSpriteAnimation<'w, 's> for Commands<'w, 's> {
    fn spawn_sprite_animation<'a>(
        &'a mut self,
        // TODO: handle can be part of the SpriteAnimation struct
        texture_atlas_handle: &Handle<TextureAtlas>,
        sprite_animation: SpriteAnimation,
    ) -> EntityCommands<'w, 's, 'a> {
        let mut x = self.spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle.clone(),
            sprite: TextureAtlasSprite {
                color: sprite_animation.tint,
                flip_x: sprite_animation.flip_x,
                flip_y: sprite_animation.flip_y,
                custom_size: sprite_animation.size,
                ..Default::default()
            },
            transform: Transform {
                translation: sprite_animation.position,
                rotation: sprite_animation.rotation,
                ..Default::default()
            },
            ..Default::default()
        });
        x.insert(Timer::from_seconds(1. / sprite_animation.fps, true));
        x
    }
}

impl TextureAtlasMap {
    pub fn get<'a, T: Into<AssetPath<'a>>>(&self, key: T) -> Option<&Handle<TextureAtlas>> {
        self.map.get(&key.into().get_id())
    }

    pub fn insert<'a, T: Into<AssetPath<'a>>>(
        &'a mut self,
        key: T,
        texture_atlas: TextureAtlas,
        texture_atlases: &mut Assets<TextureAtlas>,
    ) {
        self.map
            .insert(key.into().into(), texture_atlases.add(texture_atlas));
    }
}

impl Plugin for SpriteAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpriteAnimationEvent>()
            .add_system(update_sprite_animation_system);
    }
}

fn update_sprite_animation_system(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        Entity,
        &mut Timer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
    mut evt: EventWriter<SpriteAnimationEvent>,
) {
    for (entity, mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = (sprite.index + 1) % texture_atlas.textures.len();

            if sprite.index == 0 {
                evt.send(SpriteAnimationEvent(entity));
            }
        }
    }
}
