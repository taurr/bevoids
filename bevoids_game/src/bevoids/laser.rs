use bevoids_assets::{SoundAsset, SpriteAsset};
use bevy::{log, prelude::*};
use bevy_effects::{
    despawn::DelayedFadeDespawn,
    sound::{PlaySfx, SfxCmdEvent},
};
use std::f32::consts::PI;

use crate::bounds::GfxBounds;

use super::{
    movement::{ShadowController, Velocity},
    player::Player,
    settings::Settings,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct FireLaserEvent;

#[derive(Debug, Component)]
pub(crate) struct Laser;

pub(crate) fn laser_fired_system(
    mut commands: Commands,
    mut events: EventReader<FireLaserEvent>,
    mut sfx_event: EventWriter<SfxCmdEvent<SoundAsset>>,
    player_query: Query<(&Transform, &Velocity), (With<Player>, With<ShadowController>)>,
    asset_server: Res<AssetServer>,
    bounds: Res<GfxBounds>,
    settings: Res<Settings>,
) {
    for _ in events.iter() {
        let (
            &Transform {
                translation: player_position,
                rotation: player_orientation,
                ..
            },
            &Velocity(player_velocity),
        ) = player_query.iter().next().expect("missing player");

        let laser_texture = asset_server.load(SpriteAsset::GfxLaser);
        let size = settings.laser.size.into();

        let position = player_position
            + player_orientation.mul_vec3(Vec3::new(0., settings.player.gun_ypos, -1.));

        let velocity = player_orientation.mul_vec3(Vec3::Y).truncate();
        let velocity = velocity
            * (player_velocity.length() * player_velocity.angle_between(velocity).cos()
                + settings.laser.speed);

        let laser_id = commands
            .spawn_bundle(SpriteBundle {
                texture: laser_texture,
                transform: Transform {
                    translation: position,
                    rotation: Quat::from_rotation_z(PI / 2.).mul_quat(player_orientation),
                    ..Default::default()
                },
                sprite: Sprite {
                    custom_size: Some(size),
                    ..Default::default()
                },
                ..SpriteBundle::default()
            })
            .insert(Laser)
            .insert(Velocity::from(velocity))
            .insert(GfxBounds::from_pos_and_size(position.truncate(), size))
            .insert(DelayedFadeDespawn::new(
                settings.laser.lifetime,
                settings.laser.fadeout,
            ))
            .id();

        sfx_event.send(
            PlaySfx::new(SoundAsset::Laser)
                .with_panning((position.x + bounds.width() / 2.) / bounds.width())
                .into(),
        );
        log::trace!(buller=?laser_id, "spawned laser");
    }
}
