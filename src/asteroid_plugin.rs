use bevy::{ecs::system::EntityCommands, log, prelude::*};
use derive_more::{From, Into};
use rand::Rng;
use std::{f32::consts::PI, time::Duration};

use crate::{
    constants::*,
    fade_plugin::Fadeout,
    movement_plugin::{
        spawn_shadows_for_display_wrap, InsideWindow, ShadowController, ShadowOf, Velocity,
    },
    player_plugin::Player,
    AsteroidMaterials, Bounds, Despawn, GameState,
};

pub struct AsteroidPlugin;

#[derive(Component, Debug)]
pub struct Asteroid;

#[derive(Component, Debug)]
pub struct AsteroidsInGame(u8);

pub(crate) fn spawn_split_asteroids(
    asteroid_size: Vec2,
    asteroid_position: &Vec3,
    player_position: &Vec3,
    winow_bounds: &Bounds,
    materials: &mut AsteroidMaterials,
    commands: &mut Commands,
) {
    let max_size = asteroid_size.max_element() * ASTEROID_SPLIT_SIZE_RATIO;
    if max_size < ASTEROID_MIN_SIZE {
        return;
    }

    let angle_between_splits = 2. * PI / ASTEROID_SPLIT_INTO as f32;
    // to not send a split asteroid towards the user, skew the angle generation with half the angle
    let skew_angle = angle_between_splits / 2.;
    // skew is in relation to the vector between the asteroid and the player
    let player_asteroid_dir = player_position.truncate() - asteroid_position.truncate();

    let mut rng = rand::thread_rng();

    log::debug!("spawning split asteroids");
    for velocity in (0..ASTEROID_SPLIT_INTO).map(|i| {
        let asteroid_angle = angle_between_splits * i as f32 + skew_angle;
        Quat::from_rotation_z(asteroid_angle)
            .mul_vec3(player_asteroid_dir.extend(0.))
            .normalize()
            * rng.gen_range(ASTEROID_MIN_SPEED..ASTEROID_MAX_SPEED)
    }) {
        match spawn_asteroid(
            max_size,
            asteroid_position.truncate(),
            velocity.truncate(),
            winow_bounds,
            materials,
            commands,
        ) {
            Ok(_) => log::debug!("split asteroids spawned"),
            Err(_) => log::warn!("failed spawning split asteroid"),
        };
    }
}

pub(crate) fn despawn_asteroid(
    commands: &mut Commands,
    asteroid_ctrl: Entity,
    shadows_query: &Query<
        (Entity, &ShadowOf),
        (With<Asteroid>, Without<Fadeout>, Without<Despawn>),
    >,
) {
    commands
        .entity(asteroid_ctrl)
        .remove_bundle::<(Asteroid, Velocity)>()
        .insert(FreeMaterialAndFadeout);

    // despawn all children
    for entity in shadows_query
        .iter()
        .filter(|(_, shadowof)| asteroid_ctrl == shadowof.controller)
        .map(|(entity, _)| entity)
    {
        commands
            .entity(entity)
            .remove_bundle::<(Asteroid, Velocity)>()
            .insert(Despawn);
    }
}

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(GameState::InGame).with_system(asteroid_level_init.system()),
        );
        app.add_system_set(
            SystemSet::on_update(GameState::InGame)
                .with_system(asteroid_spawner.system())
                .with_system(asteroid_despawner.system()),
        );
    }
}

#[derive(Component, Debug)]
struct FreeMaterialAndFadeout;

#[derive(Component, Debug, From, Into)]
struct AsteroidsToSpawn(usize);

fn asteroid_despawner(
    mut commands: Commands,
    asteroid_query: Query<(Entity, &Handle<ColorMaterial>), With<FreeMaterialAndFadeout>>,
    mut materials: ResMut<AsteroidMaterials>,
) {
    for (asteroid, material) in asteroid_query.iter() {
        materials.push(material.clone());

        log::trace!("free material");
        commands
            .entity(asteroid)
            .remove_bundle::<(Asteroid, Velocity, FreeMaterialAndFadeout)>()
            .insert(Fadeout::from_secs_f32(ASTEROID_FADEOUT_SECONDS));
    }
}

fn asteroid_level_init(mut commands: Commands) {
    commands.insert_resource(AsteroidsInGame(0));
    commands
        .spawn()
        .insert(AsteroidsToSpawn(ASTEROIDS_LEVEL_SPAWN));
}

fn asteroid_spawner(
    mut commands: Commands,
    mut query: Query<(Entity, &mut AsteroidsToSpawn, Option<&mut Timer>)>,
    mut materials: ResMut<AsteroidMaterials>,
    player_query: Query<&Transform, With<Player>>,
    window_bounds: Res<Bounds>,
    time: Res<Time>,
) {
    if let Ok((entity, mut level_asteroids, timer)) = query.get_single_mut() {
        let asteroids_to_spawn = timer
            .and_then(|mut timer| {
                if timer.tick(time.delta()).finished() {
                    Some(Ok(1))
                } else {
                    Some(Err(()))
                }
            })
            .unwrap_or_else(|| {
                if ASTEROID_SPAWN_DELAY >= 0.01 {
                    commands.entity(entity).insert(Timer::new(
                        Duration::from_secs_f32(ASTEROID_SPAWN_DELAY),
                        true,
                    ));
                }
                Ok(1)
            });
        if asteroids_to_spawn.is_err() {
            return;
        }
        let asteroids_to_spawn = asteroids_to_spawn.unwrap();

        let mut rng = rand::thread_rng();
        for _ in 0..asteroids_to_spawn {
            let asteroid_max_size = rng.gen_range(ASTEROID_MIN_SIZE..ASTEROID_MAX_SIZE);

            let asteroid_position = loop {
                let position = {
                    let (w, h) = (window_bounds.width() / 2.0, window_bounds.height() / 2.0);
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

            let asteroid_velocity = {
                let random_direction = rng.gen_range(0.0..(2. * PI));
                let random_speed = rng.gen_range(ASTEROID_MIN_SPEED..ASTEROID_MAX_SPEED);
                Quat::from_rotation_z(random_direction).mul_vec3(Vec3::Y) * random_speed
            };

            log::debug!("spawning level asteroid");
            match spawn_asteroid(
                asteroid_max_size,
                asteroid_position.truncate(),
                asteroid_velocity.truncate(),
                &window_bounds,
                &mut materials,
                &mut commands,
            ) {
                Ok(_) => {}
                Err(_) => log::warn!("failed spawning level asteroid"),
            };
        }

        level_asteroids.0 -= asteroids_to_spawn;
        if level_asteroids.0 == 0 {
            log::info!("all asteroids spawned for level");
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_asteroid(
    max_size: f32,
    position: Vec2,
    velocity: Vec2,
    window_bounds: &Bounds,
    materials: &mut AsteroidMaterials,
    commands: &mut Commands,
) -> Result<(), ()> {
    match materials.pop() {
        Ok((material, material_size)) => {
            let mut rng = rand::thread_rng();
            let asteroid_scale = max_size / material_size.max_element();
            let asteroid_position = position.extend(rng.gen_range(ASTEROID_Z_MIN..ASTEROID_Z_MAX));
            let asteroid_size = material_size * asteroid_scale;
            let asteroid_id = commands
                .spawn_bundle(SpriteBundle {
                    material: material.clone(),
                    transform: Transform {
                        translation: asteroid_position,
                        scale: Vec2::splat(asteroid_scale).extend(1.),
                        ..Transform::default()
                    },
                    ..SpriteBundle::default()
                })
                .insert(Asteroid)
                .insert(Bounds::from_pos_and_size(position, asteroid_size))
                .insert(ShadowController)
                .insert(Velocity::from(velocity))
                .insert(InsideWindow)
                .id();

            log::info!(?asteroid_size, asteroid=?asteroid_id, "asteroid spawned");

            spawn_shadows_for_display_wrap(
                asteroid_id,
                asteroid_position,
                asteroid_size,
                asteroid_scale,
                material,
                &Some(|mut cmds: EntityCommands| {
                    cmds.insert(Asteroid);
                }),
                window_bounds,
                commands,
            );
            Ok(())
        }
        Err(_) => Err(()),
    }
}
