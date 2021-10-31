use bevy::{ecs::system::EntityCommands, log, math::vec3, prelude::*};
use derive_more::{Deref, DerefMut, From, Into};
use rand::Rng;
use std::{f32::consts::PI, time::Duration};

use crate::{
    constants::*,
    fade_despawn_plugin::{DelayedFadeDespawn, Despawn, FadeDespawn},
    movement_plugin::{spawn_display_shadows, InsideWindow, ShadowController, Velocity},
    textures::Textures,
    Bounds, GameState,
};

pub(crate) struct PlayerPlugin;

#[derive(Component, Debug)]
pub(crate) struct Player;

#[derive(Component, Debug)]
pub(crate) struct Flame;

#[derive(Component, Debug)]
pub(crate) struct Bullet;

#[derive(Component, Debug, Default, From, Into, Copy, Clone, Deref, DerefMut)]
pub(crate) struct Orientation(Quat);

pub(crate) fn kill_player(commands: &mut Commands, player: Entity) {
    log::warn!(?player, "player dead");
    commands
        .entity(player)
        .remove::<Player>()
        .remove::<Velocity>()
        .insert(FadeDespawn::from_secs_f32(PLAYER_FADEOUT_SECONDS));
}

pub(crate) fn bullet_spent(commands: &mut Commands, bullet: Entity) {
    log::debug!(?bullet, "bullet spent");
    commands.entity(bullet).remove::<Bullet>().insert(Despawn);
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(GameState::InGame).with_system(player_spawn.system()),
        );
        app.add_system_set(
            SystemSet::on_update(GameState::InGame).with_system(player_controls.system()),
        );
    }
}

fn player_spawn(
    mut commands: Commands,
    window_bounds: Res<Bounds>,
    textures: Res<Textures>,
    mut material_assets: ResMut<Assets<ColorMaterial>>,
) {
    log::debug!("spawning player");
    let mut rng = rand::thread_rng();

    let player_position_vec2 = Vec2::new(
        rng.gen_range(-window_bounds.width() / 2.0..window_bounds.width() / 2.0),
        rng.gen_range(-window_bounds.height() / 2.0..window_bounds.height() / 2.0),
    );
    let player_position_vec3 = player_position_vec2.extend(PLAYER_Z);

    let texture_size = textures.get_size(&textures.spaceship).unwrap();
    let player_material = material_assets.add(textures.spaceship.clone().into());
    let player_scale = PLAYER_MAX_SIZE / texture_size.max_element();
    let player_size = texture_size * player_scale;
    let random_rotation = Quat::from_rotation_z(rng.gen_range(0.0..(2. * PI)));
    let player_velocity = random_rotation.mul_vec3(Vec3::Y).truncate() * PLAYER_START_SPEED;

    let player_id: Entity = commands
        .spawn_bundle(SpriteBundle {
            material: player_material.clone(),
            transform: Transform {
                translation: player_position_vec3,
                rotation: random_rotation,
                scale: Vec2::splat(player_scale).extend(1.),
            },
            ..SpriteBundle::default()
        })
        .insert(Player)
        .insert(Bounds::from_pos_and_size(player_position_vec2, player_size))
        .insert(Velocity::from(player_velocity))
        .insert(Orientation(random_rotation))
        .insert(ShadowController)
        .insert(InsideWindow)
        .id();

    spawn_display_shadows(
        player_id,
        player_position_vec3,
        player_size,
        player_scale,
        player_material,
        &Some(|mut cmds: EntityCommands| {
            cmds.insert(Player);
        }),
        &window_bounds,
        &mut commands,
    );

    log::info!(player=?player_id, "player spawned");
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
            let delta_v = player_orientation
                .0
                .mul_vec3(vec3(0., PLAYER_ACCELLERATION, 0.))
                .truncate()
                * time.delta_seconds();
            let velocity =
                (Vec2::from(*player_velocity) + delta_v).clamp_length(0., PLAYER_MAX_SPEED);
            **player_velocity = velocity.into();
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
        } else {
            // decellerate
            let delta_v = Vec2::from(*player_velocity).normalize()
                * PLAYER_DECCELLERATION
                * time.delta_seconds();
            let velocity =
                (Vec2::from(*player_velocity) - delta_v).clamp_length(0., PLAYER_MAX_SPEED);
            **player_velocity = velocity.into();
            if kb.just_released(KeyCode::Up) {
                log::trace!("accellerate off");
                for flame in flame_query.iter() {
                    commands.entity(flame).despawn();
                }
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
                *player_velocity,
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
    player_velocity: Velocity,
    player_orientation: &Orientation,
) {
    let bullet_texture_size = textures.get_size(&textures.shot).unwrap();
    let bullet_scale = BULLET_MAX_SIZE / bullet_texture_size.max_element();
    let bullet_velocity = {
        let bullet_velocity = player_orientation
            .mul_vec3(vec3(0., BULLET_SPEED, 0.))
            .truncate();
        Velocity::from(
            bullet_velocity
                + Vec2::from(*player_velocity).project_onto(Vec2::from(bullet_velocity)),
        )
    };

    let bullet_position = player_transform.translation
        + player_orientation.mul_vec3(Vec3::new(
            0.,
            BULLET_PLAYER_RELATIVE_Y,
            BULLET_PLAYER_RELATIVE_Z,
        ));
    let bullet_id = commands
        .spawn_bundle(SpriteBundle {
            material: material_assets.add(ColorMaterial::texture(textures.shot.clone())),
            transform: Transform {
                translation: bullet_position,
                rotation: Quat::from_rotation_z(PI / 2.).mul_quat(player_orientation.0),
                scale: Vec2::splat(bullet_scale).extend(1.),
            },
            ..SpriteBundle::default()
        })
        .insert(Bullet)
        .insert(Bounds::from_pos_and_size(
            bullet_position.truncate(),
            bullet_texture_size * bullet_scale,
        ))
        .insert(DelayedFadeDespawn::new(
            Duration::from_secs_f32(BULLET_LIFETIME_SECONDS),
            Duration::from_secs_f32(BULLET_FADEOUT_SECONDS),
        ))
        .insert(bullet_velocity)
        .id();
    log::debug!(buller=?bullet_id, "spawned bullet");
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
            material: material_assets.add(ColorMaterial::modulated_texture(
                texture,
                Color::WHITE.clone(),
            )),
            transform: Transform {
                translation: Vec3::new(0., FLAME_RELATIVE_Y, FLAME_RELATIVE_Z)
                    / player_transform.scale,
                rotation: Quat::default(),
                scale: Vec2::splat(scale / player_transform.scale.x).extend(1.),
            },
            ..SpriteBundle::default()
        })
        .insert(Flame)
        .id();
    flame
}
