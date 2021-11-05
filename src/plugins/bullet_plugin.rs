use bevy::{log, math::vec3, prelude::*};
use bevy_kira_audio::{Audio, AudioChannel};
use derive_more::{Constructor, Deref};
use std::{f32::consts::PI, time::Duration};

use crate::{
    assets::LoadRelative,
    constants::*,
    plugins::{
        DelayedFadeDespawn, Despawn, Orientation, Player, ShadowController, Textures, Velocity,
    },
    Args, Bounds, GameState,
};

pub struct BulletPlugin;

#[derive(Debug, Clone, Copy)]
pub struct FireBulletEvent;

#[derive(Debug, Clone, Copy, Deref, Constructor)]
pub struct BulletSpentEvent(Entity);

#[derive(Component, Debug, Reflect)]
pub struct Bullet;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FireBulletEvent>()
            .add_event::<BulletSpentEvent>();

        app.register_type::<Bullet>();

        app.add_system_set(
            SystemSet::on_update(GameState::InGame)
                .with_system(play_sound_on_fire.system())
                .with_system(fire_bullet.system())
                .with_system(spend_bullet.system()),
        );
    }
}

fn spend_bullet(
    mut events: EventReader<BulletSpentEvent>,
    query: Query<Entity, With<Bullet>>,
    mut commands: Commands,
) {
    for &bullet in events.iter().map(|e| e as &Entity) {
        if query.iter().any(|b| b == bullet) {
            commands.entity(bullet).remove::<Bullet>().insert(Despawn);
        }
    }
}

fn play_sound_on_fire(
    mut events: EventReader<FireBulletEvent>,
    args: Res<Args>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
) {
    for _ in events.iter() {
        let audio_channel = AudioChannel::new(AUDIO_CHANNEL_LASER.into());
        audio.play_in_channel(
            asset_server
                .load_relative(&AUDIO_LASER, &*args)
                .expect("missing laser sound"),
            &audio_channel,
        );
        audio.set_volume_in_channel(AUDIO_LASER_VOLUME, &audio_channel);
    }
}

fn fire_bullet(
    mut events: EventReader<FireBulletEvent>,
    player_query: Query<
        (&Transform, &Orientation, &Velocity),
        (With<Player>, With<ShadowController>),
    >,
    mut commands: Commands,
    textures: Res<Textures>,
    mut material_assets: ResMut<Assets<ColorMaterial>>,
) {
    let (
        &Transform {
            translation: position,
            ..
        },
        &Orientation(orientation),
        &Velocity(player_velocity),
    ) = player_query.get_single().expect("missing player");

    let (size, scale) = {
        let texture_size = textures
            .get_size(&textures.shot)
            .expect("missing size for laser texture");
        let scale = BULLET_MAX_SIZE / texture_size.max_element();
        (texture_size * scale, scale)
    };

    let position = position
        + orientation.mul_vec3(Vec3::new(
            0.,
            BULLET_PLAYER_RELATIVE_Y,
            BULLET_PLAYER_RELATIVE_Z,
        ));

    let velocity = {
        let velocity = orientation.mul_vec3(vec3(0., BULLET_SPEED, 0.)).truncate();
        Velocity::from(velocity + player_velocity.project_onto(Vec2::from(velocity)))
    };

    for _ in events.iter() {
        let bullet_id = commands
            .spawn_bundle(SpriteBundle {
                material: material_assets.add(ColorMaterial::texture(textures.shot.clone())),
                transform: Transform {
                    translation: position,
                    rotation: Quat::from_rotation_z(PI / 2.).mul_quat(orientation),
                    scale: Vec2::splat(scale).extend(1.),
                },
                ..SpriteBundle::default()
            })
            .insert(Bullet)
            .insert(velocity)
            .insert(Bounds::from_pos_and_size(position.truncate(), size))
            .insert(
                DelayedFadeDespawn::new(
                    Duration::from_secs_f32(BULLET_LIFETIME_SECONDS),
                    Duration::from_secs_f32(BULLET_FADEOUT_SECONDS),
                )
                .before_fading(|cmds| {
                    cmds.remove::<Bullet>();
                }),
            )
            .id();
        log::debug!(buller=?bullet_id, "spawned bullet");
    }
}
