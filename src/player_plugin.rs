use bevy::{core::FixedTimestep, ecs::system::EntityCommands, log, math::vec3, prelude::*};
use derive_more::{Display, From, Into};
use rand::Rng;
use std::{f32::consts::PI, time::Duration};

use crate::{
    fade_plugin::DelayedFadeout,
    movement_plugin::{spawn_shadows_for_display_wrap, ShadowController, Velocity},
    Despawn, GameState, SpriteSize, Textures, WinSize, BULLET_FADEOUT_SECONDS,
    BULLET_LIFETIME_SECONDS, BULLET_MAX_SIZE, BULLET_RELATIVE_Y, BULLET_RELATIVE_Z, BULLET_SPEED,
    FLAME_OPACITY, FLAME_RELATIVE_Y, FLAME_RELATIVE_Z, FLAME_WIDTH, PLAYER_ACCELLERATION,
    PLAYER_DECCELLERATION, PLAYER_MAX_SIZE, PLAYER_MAX_SPEED, PLAYER_START_SPEED,
    PLAYER_TURN_SPEED, PLAYER_Z,
};

pub(crate) struct PlayerPlugin;

#[derive(Debug, Component, Display)]
pub(crate) struct Player;

#[derive(Debug, Component, Display)]
pub(crate) struct Bullet;

#[derive(Debug, Default, Component, Display, From, Into, Copy, Clone)]
pub(crate) struct Orientation(Quat);

#[derive(Debug, Component, Display)]
pub(crate) struct Flame;

pub(crate) fn kill_player(commands: &mut Commands, player: Entity) {
    log::warn!(?player, "player dead");
    commands.entity(player).remove::<Player>().insert(Despawn);
}

pub(crate) fn bullet_spent(commands: &mut Commands, bullet: Entity) {
    log::debug!(?bullet, "bullet spent");
    commands.entity(bullet).despawn();
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(GameState::InGame).with_system(player_spawn.system()),
        );
        app.add_system_set(
            SystemSet::on_update(GameState::InGame).with_system(player_controls.system()),
        );
        app.add_system_set(
            SystemSet::on_update(GameState::InGame)
                .with_run_criteria(FixedTimestep::step(1.0 as f64))
                .with_system(player_stats.system()),
        );
    }
}

fn player_stats(player_query: Query<&Velocity, With<Player>>) {
    for velocity in player_query.iter() {
        log::trace!(speed=velocity.length());
    }
}

fn player_spawn(
    mut commands: Commands,
    win_size: Res<WinSize>,
    textures: Res<Textures>,
    mut material_assets: ResMut<Assets<ColorMaterial>>,
) {
    log::debug!("spawning player");
    let mut rng = rand::thread_rng();

    let position = Vec2::new(
        rng.gen_range(-win_size.0.x / 2.0..win_size.0.x / 2.0),
        rng.gen_range(-win_size.0.y / 2.0..win_size.0.y / 2.0),
    );
    let random_rotation = Quat::from_rotation_z(rng.gen_range(0.0..(2. * PI)));
    let texture_size = textures.get_size(&textures.spaceship).unwrap();
    let scale = PLAYER_MAX_SIZE / texture_size.max_element();
    let velocity = random_rotation.mul_vec3(Vec3::Y) * PLAYER_START_SPEED;
    let material = material_assets.add(textures.spaceship.clone().into());

    let translation = position.extend(PLAYER_Z);
    let sprite_size = SpriteSize(texture_size * scale);
    let id: Entity = commands
        .spawn_bundle(SpriteBundle {
            material: material.clone(),
            transform: Transform {
                translation,
                rotation: random_rotation,
                scale: Vec2::splat(scale).extend(1.),
            },
            ..Default::default()
        })
        .insert(sprite_size)
        .insert(Player)
        .insert(SpriteSize(texture_size * scale))
        .insert(ShadowController)
        .insert(Orientation(random_rotation))
        .insert(Velocity::new(velocity))
        .id();

    log::info!(player=?id, "player spawned");

    spawn_shadows_for_display_wrap(
        id,
        material,
        sprite_size,
        &win_size,
        scale,
        translation,
        &Some(|mut cmds: EntityCommands| {
            cmds.insert(Player);
        }),
        &mut commands,
    );
}

fn player_controls(
    mut commands: Commands,
    kb: Res<Input<KeyCode>>,
    mut player_query: Query<
        (Entity, &mut Velocity, &mut Orientation, &mut Transform),
        With<Player>,
    >,
    flame_query: Query<Entity, With<Flame>>,
    textures: Res<Textures>,
    mut material_assets: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
) {
    for (player, mut player_velocity, mut player_orientation, mut player_transform) in
        player_query.iter_mut()
    {
        // orientation
        if kb.pressed(KeyCode::Left) {
            player_orientation.0 = player_orientation.0.mul_quat(Quat::from_rotation_z(
                PLAYER_TURN_SPEED * time.delta_seconds(),
            ));
        } else if kb.pressed(KeyCode::Right) {
            player_orientation.0 = player_orientation.0.mul_quat(Quat::from_rotation_z(
                -PLAYER_TURN_SPEED * time.delta_seconds(),
            ));
        }
        player_transform.rotation = player_orientation.0;

        if kb.pressed(KeyCode::Up) {
            // accelleration
            let v = player_orientation
                .0
                .mul_vec3(vec3(0., PLAYER_ACCELLERATION, 0.))
                * time.delta_seconds();
            let velocity = **player_velocity + v;
            let capped_velocity = if velocity.length() > PLAYER_MAX_SPEED {
                velocity.normalize() * PLAYER_MAX_SPEED
            } else {
                velocity
            };
            **player_velocity = capped_velocity;
        } else {
            // decellerate
            if player_velocity.length() > 0. {
                let v = player_velocity.normalize()
                    * f32::min(
                        PLAYER_DECCELLERATION * time.delta_seconds(),
                        player_velocity.length(),
                    );
                **player_velocity -= v;
            }
        }
        if kb.just_pressed(KeyCode::Up) {
            log::trace!("accellerate on");
            let flame = spawn_flame(
                &mut commands,
                &textures,
                &mut material_assets,
                &player_transform,
            );
            commands.entity(player).push_children(&[flame]);
        }
        if kb.just_released(KeyCode::Up) {
            log::trace!("accellerate off");
            for flame in flame_query.iter() {
                commands.entity(flame).despawn();
            }
        }

        // fire
        if kb.just_pressed(KeyCode::Space) {
            log::debug!("fire!");
            spawn_bullet(
                &mut commands,
                &textures,
                &mut material_assets,
                &player_transform,
                &player_velocity,
                &player_orientation,
            );
        }
    }
}

fn spawn_bullet(
    commands: &mut Commands,
    textures: &Textures,
    material_assets: &mut Assets<ColorMaterial>,
    player_transform: &Transform,
    player_velocity: &Velocity,
    player_orientation: &Orientation,
) {
    let texture_handle = textures.shot.clone();
    let texture_size = textures.get_size(&texture_handle).unwrap();
    let scale = BULLET_MAX_SIZE / texture_size.max_element();
    let mut bullet_velocity = player_orientation.0.mul_vec3(vec3(0., BULLET_SPEED, 0.));
    bullet_velocity += player_velocity.project_onto(bullet_velocity);

    let id = commands
        .spawn_bundle(SpriteBundle {
            // TODO: ColorMaterial should not be created each time we show the flame
            material: material_assets.add(ColorMaterial::texture(textures.shot.clone())),
            // TODO: Transform should not be created each time we show the flame
            transform: Transform {
                translation: player_transform.translation
                    + player_orientation
                        .0
                        .mul_vec3(vec3(0., BULLET_RELATIVE_Y, BULLET_RELATIVE_Z)),
                rotation: Quat::from_rotation_z(PI / 2.).mul_quat(player_orientation.0),
                scale: Vec2::splat(scale).extend(1.),
            },
            ..Default::default()
        })
        .insert(Bullet)
        .insert(SpriteSize(texture_size * scale))
        .insert(DelayedFadeout::new(
            Duration::from_secs_f32(BULLET_LIFETIME_SECONDS),
            Duration::from_secs_f32(BULLET_FADEOUT_SECONDS),
        ))
        .insert(Velocity::new(bullet_velocity)).id();
    log::debug!(buller=?id, "spawned bullet");
}

fn spawn_flame(
    commands: &mut Commands,
    textures: &Textures,
    material_assets: &mut Assets<ColorMaterial>,
    player_transform: &Transform,
) -> Entity {
    let texture = textures.flame.clone();
    let flame_width = textures.get_size(&texture).unwrap().x;
    let scale = FLAME_WIDTH / flame_width;
    let flame = commands
        .spawn_bundle(SpriteBundle {
            // TODO: ColorMaterial should not be created each time we show the flame
            material: material_assets.add(ColorMaterial::modulated_texture(
                texture,
                *Color::WHITE.clone().set_a(FLAME_OPACITY),
            )),
            // TODO: Transform should not be created each time we show the flame
            transform: Transform {
                translation: Vec2::new(0., FLAME_RELATIVE_Y).extend(FLAME_RELATIVE_Z)
                    / player_transform.scale,
                rotation: Default::default(),
                scale: Vec2::splat(scale / player_transform.scale.x).extend(1.),
            },
            ..Default::default()
        })
        .insert(Flame)
        .id();
    flame
}
