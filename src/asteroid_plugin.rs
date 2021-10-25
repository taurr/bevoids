use crate::{
    movement_plugin::{spawn_shadows_for_display_wrap, ShadowController, Velocity},
    player_plugin::Player,
    AsteroidMaterials, GameState, SpriteSize, WinSize, ASTEROIDS_LEVEL_SPAWN,
    ASTEROIDS_PLAYER_SPAWN_DISTANCE, ASTEROID_FADEIN_SECONDS, ASTEROID_MAX_SIZE,
    ASTEROID_MAX_SPEED, ASTEROID_MIN_SIZE, ASTEROID_MIN_SPEED, ASTEROID_Z_MAX, ASTEROID_Z_MIN,
};
use bevy::{ecs::system::EntityCommands, log, prelude::*};
use rand::Rng;
use std::{f32::consts::PI, time::Duration};

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(GameState::InGame).with_system(asteroid_level_init.system()),
        );
        app.add_system_set(
            SystemSet::on_update(GameState::InGame).with_system(asteroid_spawner.system()),
        );
    }
}

#[derive(Debug, Default, Component)]
struct AsteroidsToSpawn(usize);

#[derive(Debug, Default, Component)]
pub struct Asteroid;

pub fn split_asteroid(
    asteroid_size: &Vec2,
    asteroid_position: &Vec3,
    asteroid_velocity: &Velocity,
    win_size: &WinSize,
    materials: &mut AsteroidMaterials,
    commands: &mut Commands,
) {
    let size = asteroid_size.max_element() * 2. / 3.;
    if size < ASTEROID_MIN_SIZE {
        return;
    }

    let tracjetories = {
        let mut rng = rand::thread_rng();
        let direction = asteroid_velocity.normalize();
        let minimum_speed = f32::min(size / ASTEROID_FADEIN_SECONDS, ASTEROID_MIN_SPEED);
        const ANGLE: f32 = PI / 3.;
        [
            Quat::from_rotation_z(ANGLE)
                .mul_vec3(direction * rng.gen_range(minimum_speed..ASTEROID_MAX_SPEED)),
            Quat::from_rotation_z(-ANGLE)
                .mul_vec3(direction * rng.gen_range(minimum_speed..ASTEROID_MAX_SPEED)),
        ]
    };

    for velocity in tracjetories {
        match spawn_asteroid(
            size,
            asteroid_position,
            &velocity,
            win_size,
            materials,
            commands,
        ) {
            Ok(_) => log::debug!("spawned child asteroid"),
            Err(_) => log::warn!("failed spawning child asteroid"),
        };
    }
}

fn asteroid_level_init(mut commands: Commands) {
    commands
        .spawn()
        .insert(Timer::new(Duration::from_secs_f32(0.15), true))
        .insert(AsteroidsToSpawn(ASTEROIDS_LEVEL_SPAWN));
}

fn asteroid_spawner(
    mut commands: Commands,
    win_size: Res<WinSize>,
    mut query: Query<(Entity, &mut AsteroidsToSpawn, Option<&mut Timer>)>,
    player_query: Query<&Transform, With<Player>>,
    mut materials: ResMut<AsteroidMaterials>,
    time: Res<Time>,
) {
    if let Ok((entity, mut level_asteroids, timer)) = query.get_single_mut() {
        if let Some(mut timer) = timer {
            if !timer.tick(time.delta()).finished() {
                return;
            }
        }

        level_asteroids.0 -= 1;
        if level_asteroids.0 == 0 {
            log::info!("all asteroids spawned for level");
            commands.entity(entity).despawn();
        }

        let mut rng = rand::thread_rng();

        let size = rng.gen_range(ASTEROID_MIN_SIZE..ASTEROID_MAX_SIZE);

        let position = loop {
            let position = {
                let [w, h] = (win_size.0 / 2.0).to_array();
                Vec2::new(rng.gen_range(-w..w), rng.gen_range(-h..h))
            };
            if player_query.iter().all(|player| {
                position
                    .extend(player.translation.z)
                    .distance(player.translation)
                    > ASTEROIDS_PLAYER_SPAWN_DISTANCE
            }) {
                break position;
            }
        }
        .extend(rng.gen_range(ASTEROID_Z_MIN..ASTEROID_Z_MAX));

        let velocity = {
            let random_direction = rng.gen_range(0.0..(2. * PI));
            let random_speed = rng.gen_range(ASTEROID_MIN_SPEED..ASTEROID_MAX_SPEED);
            Quat::from_rotation_z(random_direction).mul_vec3(Vec3::Y) * random_speed
        };

        match spawn_asteroid(
            size,
            &position,
            &velocity,
            &win_size,
            &mut materials,
            &mut commands,
        ) {
            Ok(_) => log::debug!("spawned asteroid"),
            Err(_) => log::warn!("failed spawning asteroid"),
        };
    }
}

fn spawn_asteroid(
    size: f32,
    position: &Vec3,
    velocity: &Vec3,
    win_size: &WinSize,
    materials: &mut AsteroidMaterials,
    commands: &mut Commands,
) -> Result<(), ()> {
    match materials.pop() {
        Ok((material, material_size)) => {
            let mut rng = rand::thread_rng();
            let scale = size / material_size.max_element();
            log::debug!("spawn child asteroid");
            let translation =
                Vec2::from(*position).extend(rng.gen_range(ASTEROID_Z_MIN..ASTEROID_Z_MAX));
            let id = commands
                .spawn_bundle(SpriteBundle {
                    material: material.clone(),
                    transform: Transform {
                        translation,
                        scale: Vec2::splat(scale).extend(1.),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Asteroid)
                .insert(SpriteSize(material_size * scale))
                .insert(ShadowController)
                .insert(Velocity::new(*velocity))
                .id();

            spawn_shadows_for_display_wrap(
                id,
                material,
                SpriteSize(material_size * scale),
                win_size,
                scale,
                translation,
                &Some(|mut cmds: EntityCommands| {
                    cmds.insert(Asteroid);
                }),
                &mut *commands,
            );
            Ok(())
        }
        Err(_) => Err(()),
    }
}
