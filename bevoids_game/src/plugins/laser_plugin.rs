use bevy::{log, math::vec3, prelude::*};
use bevy_asset_map::{GfxBounds, TextureAssetMap};
use bevy_effects::sound::{PlaySfx, SfxCmdEvent};
use derive_more::{Constructor, Deref};
use std::{f32::consts::PI, time::Duration};

use crate::{
    plugins::{DelayedFadeDespawn, Despawn, Player, ShadowController, Velocity},
    settings::Settings,
    GameState, GeneralTexture, SoundEffect,
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
                .with_system(fire_laser)
                .with_system(spend_laser),
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

fn fire_laser(
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
        ) = player_query.get_single().expect("missing player");

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

        let velocity = {
            let velocity = orientation
                .mul_vec3(vec3(0., settings.laser.speed, 0.))
                .truncate();
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
