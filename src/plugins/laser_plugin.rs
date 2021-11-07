use bevy::{log, math::vec3, prelude::*};
use bevy_kira_audio::Audio;
use derive_more::{Constructor, Deref};
use std::{f32::consts::PI, time::Duration};

use crate::{
    constants::*,
    plugins::{DelayedFadeDespawn, Despawn, Player, ShadowController, Velocity},
    resources::{AudioAssets, Bounds, TextureAssets},
    AudioChannels, GameState, GeneralTexture, Sounds,
};

pub struct LaserPlugin;

#[derive(Debug, Clone, Copy)]
pub struct FireLaserEvent;

#[derive(Debug, Clone, Copy, Deref, Constructor)]
pub struct LaserSpentEvent(Entity);

#[derive(Component, Debug, Reflect)]
pub struct Laser;

impl Plugin for LaserPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FireLaserEvent>()
            .add_event::<LaserSpentEvent>();

        app.register_type::<Laser>();

        app.add_system_set(
            SystemSet::on_update(GameState::InGame)
                .with_system(play_sound_on_fire.system())
                .with_system(fire_laser.system())
                .with_system(spend_laser.system()),
        );
    }
}

fn spend_laser(
    mut events: EventReader<LaserSpentEvent>,
    query: Query<Entity, With<Laser>>,
    mut commands: Commands,
) {
    for &laser in events.iter().map(|e| e as &Entity) {
        if query.iter().any(|b| b == laser) {
            commands.entity(laser).remove::<Laser>().insert(Despawn);
        }
    }
}

fn play_sound_on_fire(
    mut events: EventReader<FireLaserEvent>,
    channels: Res<AudioChannels>,
    audio_assets: Res<AudioAssets<Sounds>>,
    audio: Res<Audio>,
) {
    for _ in events.iter() {
        audio.play_in_channel(
            audio_assets
                .get(Sounds::Laser)
                .expect("missing laser sound"),
            &channels.laser,
        );
    }
}

fn fire_laser(
    mut commands: Commands,
    mut events: EventReader<FireLaserEvent>,
    mut material_assets: ResMut<Assets<ColorMaterial>>,
    player_query: Query<(&Transform, &Velocity), (With<Player>, With<ShadowController>)>,
    textures: Res<TextureAssets<GeneralTexture>>,
) {
    for _ in events.iter() {
        let (
            &Transform {
                translation: position,
                rotation: orientation,
                ..
            },
            &Velocity(player_velocity),
        ) = player_query.get_single().expect("missing player");

        let laser_texture = textures
            .get(GeneralTexture::Laser)
            .expect("no texture for laser");
        let (size, scale) = {
            let scale = LASER_MAX_SIZE / laser_texture.size.max_element() as f32;
            (
                Vec2::new(laser_texture.size.x as f32, laser_texture.size.y as f32) * scale,
                scale,
            )
        };

        let position = position
            + orientation.mul_vec3(Vec3::new(
                0.,
                LASER_PLAYER_RELATIVE_Y,
                LASER_PLAYER_RELATIVE_Z,
            ));

        let velocity = {
            let velocity = orientation.mul_vec3(vec3(0., LASER_SPEED, 0.)).truncate();
            Velocity::from(velocity + player_velocity.project_onto(Vec2::from(velocity)))
        };

        let laser_id = commands
            .spawn_bundle(SpriteBundle {
                material: material_assets.add(laser_texture.texture.clone().into()),
                transform: Transform {
                    translation: position,
                    rotation: Quat::from_rotation_z(PI / 2.).mul_quat(orientation),
                    scale: Vec2::splat(scale).extend(1.),
                },
                ..SpriteBundle::default()
            })
            .insert(Laser)
            .insert(velocity)
            .insert(Bounds::from_pos_and_size(position.truncate(), size))
            .insert(
                DelayedFadeDespawn::new(
                    Duration::from_secs_f32(LASER_LIFETIME_SECONDS),
                    Duration::from_secs_f32(LASER_FADEOUT_SECONDS),
                )
                .before_fading(|cmds| {
                    cmds.remove::<Laser>();
                }),
            )
            .id();
        log::debug!(buller=?laser_id, "spawned laser");
    }
}
