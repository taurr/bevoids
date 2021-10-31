use bevy::{core::FixedTimestep, ecs::system::EntityCommands, log, prelude::*};
use bevy_kira_audio::{Audio, AudioChannel};
use derive_more::{Constructor, Deref};
use rand::Rng;
use std::{f32::consts::PI, time::Duration};

use crate::{
    assets::LoadRelative,
    constants::*,
    plugins::{
        spawn_display_shadows, AsteroidMaterials, FadeDespawn, InsideWindow, Player, ScoreBoard,
        ShadowController, ShadowOf, Velocity,
    },
    Args, Bounds, GameState,
};

pub struct AsteroidPlugin;

#[derive(Debug, Clone, Copy, Deref, Constructor)]
pub struct RemoveAsteroidEvent(Entity);

#[derive(Debug, Clone, Copy, Deref, Constructor)]
pub struct AsteroidShotEvent(Entity);

#[derive(Component, Debug)]
pub struct Asteroid;

#[derive(Default)]
pub struct AsteroidCounter {
    spawned: u32,
    shot: u32,
}

#[derive(Debug, Clone, Copy, Deref, Constructor)]
struct AsteroidControllerShotEvent(Entity);

#[derive(Debug, Clone, Copy, Constructor)]
struct SpawnAsteroidEvent(f32, Option<Vec3>);

#[derive(Component, Debug)]
struct AsteroidSpawnDelay(f32);

#[derive(Component, Debug)]
struct AsteroidsSpawner;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AsteroidShotEvent>();
        app.add_event::<AsteroidControllerShotEvent>();
        app.add_event::<SpawnAsteroidEvent>();
        app.add_event::<RemoveAsteroidEvent>();

        app.add_system_set(
            SystemSet::on_enter(GameState::InGame).with_system(asteroid_enter_ingame.system()),
        );

        app.add_system_set(
            SystemSet::on_update(GameState::InGame)
                .with_run_criteria(FixedTimestep::step(DIFFICULTY_RAISER_TIMESTEP))
                .with_system(difficulty_raiser.system()),
        );
        app.add_system_set(
            SystemSet::on_update(GameState::InGame)
                .with_system(
                    find_shot_asteroid_controller
                        .system()
                        .label("find_asteroid_ctrl"),
                )
                .with_system(asteroid_spawner.system().label("asteroid_spawner"))
                .with_system(
                    split_and_despawn_shot_asteroids
                        .system()
                        .label("split_asteroid")
                        .after("find_asteroid_ctrl"),
                )
                .with_system(remove_asteroid_on_event.system().after("split_asteroid"))
                .with_system(
                    spawn_asteroid_on_event
                        .system()
                        .chain(spawn_asteroid_on_empty_field.system())
                        .after("asteroid_spawner"),
                )
                .with_system(
                    count_shot_asteroids
                        .system()
                        .after("find_asteroid_ctrl")
                        .before("split_asteroid"),
                )
                .with_system(
                    score_on_shot_asteroids
                        .system()
                        .after("find_asteroid_ctrl")
                        .before("split_asteroid"),
                )
                .with_system(
                    sound_on_shot_asteroids
                        .system()
                        .after("find_asteroid_ctrl")
                        .before("split_asteroid"),
                ),
        );

        app.add_system_set(
            SystemSet::on_exit(GameState::InGame).with_system(asteroid_exit_ingame.system()),
        );
    }
}

fn spawn_asteroid_on_empty_field(
    mut commands: Commands,
    query: Query<Entity, With<AsteroidsSpawner>>,
    counter: Res<AsteroidCounter>,
) {
    if counter.spawned == counter.shot {
        log::warn!("field empty - force spawning asteroid");
        let entity = query.get_single().expect("no spawner present");
        commands.entity(entity).remove::<Timer>();
    }
}

fn remove_asteroid_on_event(
    mut events: EventReader<RemoveAsteroidEvent>,
    mut commands: Commands,
    asteroids_query: Query<&ShadowOf>,
    material_query: Query<&Handle<ColorMaterial>>,
    shadows_query: Query<(Entity, &ShadowOf), With<Asteroid>>,
    mut materials: ResMut<AsteroidMaterials>,
) {
    events.iter().map(|e| e.0).for_each(|asteroid| {
        let ctrl = match asteroids_query.get(asteroid) {
            Ok(&ShadowOf {
                controller: ctrl, ..
            }) => ctrl,
            Err(_) => asteroid,
        };
        materials.push(
            material_query
                .get(ctrl)
                .expect("material missing on asteroid")
                .clone(),
        );
        despawn_asteroid(&mut commands, &asteroid, &shadows_query);
    });
}

fn find_shot_asteroid_controller(
    mut shot_events: EventReader<AsteroidShotEvent>,
    mut ctrl_events: EventWriter<AsteroidControllerShotEvent>,
    asteroids_query: Query<&ShadowOf>,
) {
    for asteroid in shot_events.iter().map(|e| e.0) {
        let ctrl = match asteroids_query.get(asteroid) {
            Ok(&ShadowOf {
                controller: ctrl, ..
            }) => ctrl,
            Err(_) => asteroid,
        };
        log::info!(?asteroid, asteroid_controller=?ctrl, "asteroid shot");
        ctrl_events.send(AsteroidControllerShotEvent::new(ctrl));
    }
}

fn count_shot_asteroids(
    mut events: EventReader<AsteroidControllerShotEvent>,
    mut counter: ResMut<AsteroidCounter>,
) {
    for _ in events.iter() {
        counter.shot += 1;
        log::info!(asteroids_shot = counter.shot);
    }
}

fn score_on_shot_asteroids(
    mut events: EventReader<AsteroidControllerShotEvent>,
    mut scores_query: Query<&mut ScoreBoard>,
    bounds_query: Query<&Bounds>,
) {
    for asteroid in events.iter().map(|e| e.0) {
        if let Ok(bounds) = bounds_query.get(asteroid) {
            for mut board in scores_query.iter_mut() {
                let a = (ASTEROID_MAX_SIZE - bounds.size().max_element())
                    / (ASTEROID_MAX_SIZE - ASTEROID_MIN_SIZE)
                    * ASTEROID_MAX_SCORE;
                let score: &mut u32 = (*board).as_mut();
                *score += a as u32;
                log::info!(score);
            }
        } else {
            log::warn!(?asteroid, "no bounds for asteroid");
        }
    }
}

fn sound_on_shot_asteroids(
    mut events: EventReader<AsteroidControllerShotEvent>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    args: Res<Args>,
) {
    for _ in events.iter() {
        let audio_channel = AudioChannel::new(AUDIO_CHANNEL_EXPLOSION_ASTEROID.into());
        audio.set_volume_in_channel(AUDIO_EXPLOSION_ASTEROID_VOLUME, &audio_channel);
        audio.play_in_channel(
            asset_server
                .load_relative(&AUDIO_EXPLOSION_ASTEROID, &*args)
                .expect("missing laser sound"),
            &audio_channel,
        );
    }
}

fn split_and_despawn_shot_asteroids(
    mut commands: Commands,
    mut events: EventReader<AsteroidShotEvent>,
    mut spawn_events: EventWriter<SpawnAsteroidEvent>,
    asteroids_query: Query<(&Bounds, &Transform, &Handle<ColorMaterial>), With<Asteroid>>,
    shadows_query: Query<(Entity, &ShadowOf), With<Asteroid>>,
    mut materials: ResMut<AsteroidMaterials>,
) {
    events.iter().map(|ev| ev as &Entity).for_each(|asteroid| {
        let (bounds, transform, material) = asteroids_query
            .get(*asteroid)
            .expect("asteroid not present");

        materials.push(material.clone());
        despawn_asteroid(&mut commands, asteroid as &Entity, &shadows_query);

        let max_size = bounds.size().max_element() * ASTEROID_SPLIT_SIZE_RATIO;
        let position = Some(transform.translation);

        (0..ASTEROID_SPLIT_INTO).for_each(|_| {
            spawn_events.send(SpawnAsteroidEvent::new(max_size, position));
        });
    });
}

fn asteroid_enter_ingame(
    mut commands: Commands,
    old_asteroids_query: Query<Entity, With<Asteroid>>,
) {
    // instantly clear old asteroid entities
    old_asteroids_query
        .iter()
        .for_each(|e| commands.entity(e).despawn_recursive());

    commands.insert_resource(AsteroidCounter::default());

    // start spawning new asteroid entities
    commands
        .spawn()
        .insert(AsteroidsSpawner)
        .insert(AsteroidSpawnDelay(ASTEROID_START_SPAWN_DELAY));
}

fn asteroid_exit_ingame(
    mut commands: Commands,
    spawner_query: Query<Entity, With<AsteroidsSpawner>>,
) {
    // remove the asteroid spawner entity
    // though the system doesn't run outside 'InGame', the entity would still exist,
    // causing multiple spawners when retrying the game...
    spawner_query
        .iter()
        .for_each(|e| commands.entity(e).despawn_recursive());
}

fn difficulty_raiser(mut query: Query<&mut AsteroidSpawnDelay, With<AsteroidsSpawner>>) {
    for mut delay in query.iter_mut() {
        *delay = AsteroidSpawnDelay(delay.0 * DIFFICULTY_RAISER_SPAWN_DELAY_MULTIPLIER);
        log::info!(?delay, "new delay between spawning asteroids");
    }
}

fn asteroid_spawner(
    mut commands: Commands,
    mut query: Query<(Entity, &AsteroidSpawnDelay, Option<&mut Timer>), With<AsteroidsSpawner>>,
    mut spawn_events: EventWriter<SpawnAsteroidEvent>,
    time: Res<Time>,
) {
    if let Ok((entity, delay, timer)) = query.get_single_mut() {
        if let Some(mut timer) = timer {
            if timer.tick(time.delta()).finished() {
                log::debug!("timed asteroid");
                spawn_events.send(SpawnAsteroidEvent::new(
                    rand::thread_rng().gen_range(ASTEROID_MIN_SIZE..ASTEROID_MAX_SIZE),
                    None,
                ));
                commands
                    .entity(entity)
                    .insert(Timer::new(Duration::from_secs_f32(delay.0), false));
            }
        } else {
            log::debug!("not timed asteroid");
            spawn_events.send(SpawnAsteroidEvent::new(
                rand::thread_rng().gen_range(ASTEROID_MIN_SIZE..ASTEROID_MAX_SIZE),
                None,
            ));
            commands
                .entity(entity)
                .insert(Timer::new(Duration::from_secs_f32(delay.0), false));
        }
    }
}

fn spawn_asteroid_on_event(
    mut events: EventReader<SpawnAsteroidEvent>,
    mut commands: Commands,
    mut counter: ResMut<AsteroidCounter>,
    mut materials: ResMut<AsteroidMaterials>,
    player_tf_query: Query<&Transform, (With<Player>, With<ShadowController>)>,
    window_bounds: Res<Bounds>,
) {
    let player_tf = player_tf_query.get_single().expect("player not present!");

    for SpawnAsteroidEvent(size, position) in events.iter() {
        if *size < ASTEROID_MIN_SIZE {
            continue;
        }

        let mut rng = rand::thread_rng();

        let position = position.unwrap_or_else(|| {
            loop {
                let position = {
                    let (w, h) = (window_bounds.width() / 2.0, window_bounds.height() / 2.0);
                    Vec2::new(rng.gen_range(-w..w), rng.gen_range(-h..h))
                };
                if position
                    .extend(player_tf.translation.z)
                    .distance(player_tf.translation)
                    > ASTEROIDS_PLAYER_SPAWN_DISTANCE
                {
                    break position;
                }
            }
            .extend(rng.gen_range(ASTEROID_Z_MIN..ASTEROID_Z_MAX))
        });

        let velocity = {
            let random_direction = rng.gen_range(0.0..(2. * PI));
            let random_speed = rng.gen_range(ASTEROID_MIN_SPEED..ASTEROID_MAX_SPEED);
            Quat::from_rotation_z(random_direction).mul_vec3(Vec3::Y) * random_speed
        };

        match spawn_asteroid(
            *size,
            position.truncate(),
            velocity.truncate(),
            &window_bounds,
            &mut materials,
            &mut commands,
        ) {
            Ok(_) => {
                counter.spawned += 1;
                log::info!(asteroids_spawned = counter.spawned);
            }
            Err(_) => log::warn!("failed spawning level asteroid"),
        };
    }
}

// region: helper functions

fn spawn_asteroid(
    size: f32,
    position: Vec2,
    velocity: Vec2,
    window_bounds: &Bounds,
    materials: &mut AsteroidMaterials,
    commands: &mut Commands,
) -> Result<(), ()> {
    match materials.pop() {
        Ok((material, material_size)) => {
            let mut rng = rand::thread_rng();
            let asteroid_scale = size / material_size.max_element();
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

            log::debug!(asteroid=?asteroid_id, "asteroid spawned");

            spawn_display_shadows(
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

fn despawn_asteroid(
    commands: &mut Commands,
    asteroid_ctrl: &Entity,
    shadows_query: &Query<(Entity, &ShadowOf), With<Asteroid>>,
) {
    // despawn controller
    commands
        .entity(*asteroid_ctrl)
        .remove_bundle::<(Asteroid, Velocity)>()
        .insert(FadeDespawn::from_secs_f32(ASTEROID_FADEOUT_SECONDS));

    // despawn all children
    for entity in shadows_query.iter().filter_map(|(entity, shadowof)| {
        match *asteroid_ctrl == shadowof.controller {
            true => Some(entity),
            false => None,
        }
    }) {
        commands
            .entity(entity)
            .remove_bundle::<(Asteroid, Velocity)>()
            .insert(FadeDespawn::from_secs_f32(ASTEROID_FADEOUT_SECONDS));
    }
}

// endregion
