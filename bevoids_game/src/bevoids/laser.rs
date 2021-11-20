use bevy::{log, prelude::*};
use bevy_asset_map::{GfxBounds, TextureAssetMap};
use bevy_effects::{
    despawn::DelayedFadeDespawn,
    sound::{PlaySfx, SfxCmdEvent},
};
use std::{f32::consts::PI, time::Duration};

use super::{
    movement::{ShadowController, Velocity},
    player::Player,
    settings::Settings,
    GeneralTexture, SoundEffect,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct FireLaserEvent;

#[derive(Debug)]
pub(crate) struct Laser;

pub(crate) fn handle_fire_laser(
    mut commands: Commands,
    mut events: EventReader<FireLaserEvent>,
    mut sfx_event: EventWriter<SfxCmdEvent<SoundEffect>>,
    mut material_assets: ResMut<Assets<ColorMaterial>>,
    player_query: Query<(&Transform, &Velocity), (With<Player>, With<ShadowController>)>,
    textures: Res<TextureAssetMap<GeneralTexture>>,
    bounds: Res<GfxBounds>,
    settings: Res<Settings>,
) {
    for _ in events.iter() {
        let (
            &Transform {
                translation: position,
                rotation: orientation,
                ..
            },
            &Velocity(player_velocity),
        ) = player_query.iter().next().expect("missing player");

        let laser_texture = textures
            .get(GeneralTexture::Laser)
            .expect("no texture for laser");
        let (size, scale) = {
            let scale = settings.laser.size / laser_texture.size.max_element() as f32;
            (
                Vec2::new(laser_texture.size.x as f32, laser_texture.size.y as f32) * scale,
                scale,
            )
        };

        let position =
            position + orientation.mul_vec3(Vec3::new(0., settings.player.gun_ypos, -1.));

        let velocity = orientation.mul_vec3(Vec3::Y).truncate();
        let velocity = velocity
            * (player_velocity.length() * player_velocity.angle_between(velocity).cos()
                + settings.laser.speed);

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
            .insert(Velocity::from(velocity))
            .insert(GfxBounds::from_pos_and_size(position.truncate(), size))
            .insert(DelayedFadeDespawn::new(
                Duration::from_millis(settings.laser.lifetime_miliseconds),
                Duration::from_millis(settings.laser.fadeout_miliseconds),
            ))
            .id();

        let panning = (position.x + bounds.width() / 2.) / bounds.width();
        sfx_event.send(
            PlaySfx::new(SoundEffect::Laser)
                .with_panning(panning)
                .into(),
        );
        log::trace!(buller=?laser_id, "spawned laser");
    }
}
