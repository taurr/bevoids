use bevy::{ecs::system::EntityCommands, log, prelude::*};
use derive_more::{Constructor, Deref};
use rand::Rng;
use std::{f32::consts::PI, time::Duration};

use crate::{
    effects::{AnimationEffect, PlaySfx, SfxCmdEvent},
    plugins::{
        spawn_display_shadows, Despawn, InsideWindow, Player, ScoreBoard, ShadowController,
        ShadowOf, Velocity,
    },
    resources::{GfxBounds, TextureAssetMap},
    settings::Settings,
    Animation, AsteroidTexture, GameState, SoundEffect,
};

pub struct AsteroidPlugin;

#[derive(Debug, Clone, Copy, Deref, Constructor)]
pub struct RemoveAsteroidEvent(Entity);

#[derive(Debug, Clone, Copy, Deref, Constructor)]
pub struct AsteroidShotEvent(Entity);

#[derive(Component, Debug, Reflect)]
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
struct AsteroidSpawnDelay(Duration);

#[derive(Component, Debug)]
struct AsteroidsSpawner;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AsteroidShotEvent>();
        app.add_event::<AsteroidControllerShotEvent>();
        app.add_event::<SpawnAsteroidEvent>();
        app.add_event::<RemoveAsteroidEvent>();

        app.register_type::<Asteroid>();

        app.add_system_set(
            SystemSet::on_enter(GameState::InGame).with_system(asteroid_enter_ingame),
        );

        app.add_system_set(
            SystemSet::on_update(GameState::InGame)
                .label("asteroid_spawner")
                .with_system(asteroid_spawner),
        );

        app.add_system_set(
            SystemSet::on_update(GameState::InGame)
                .label("find_asteroid_ctrl")
                .after("hit-test")
                .with_system(find_shot_asteroid_controller),
        );

        app.add_system_set(
            SystemSet::on_update(GameState::InGame)
                .after("asteroid_spawner")
                .with_system(spawn_asteroid_on_event.chain(spawn_asteroid_on_empty_field)),
        );

        app.add_system_set(
            SystemSet::on_update(GameState::InGame)
                .label("split_asteroid")
                .after("find_asteroid_ctrl")
                .with_system(split_and_despawn_shot_asteroid_controller),
        );

        app.add_system_set(
            SystemSet::on_update(GameState::InGame)
                .with_system(remove_asteroid_on_event)
                .after("split_asteroid"),
        );

        app.add_system_set(
            SystemSet::on_update(GameState::InGame)
                .after("find_asteroid_ctrl")
                .before("split_asteroid")
                .with_system(count_shot_asteroid_controller)
                .with_system(score_on_shot_asteroid_controller),
        );

        app.add_system_set(SystemSet::on_exit(GameState::InGame).with_system(asteroid_exit_ingame));
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
    mut anim_events: EventWriter<AnimationEffect<Animation>>,
    mut commands: Commands,
    transform_query: Query<(&Transform, &GfxBounds)>,
    asteroids_query: Query<&ShadowOf>,
    shadows_query: Query<(Entity, &ShadowOf), With<Asteroid>>,
    settings: Res<Settings>,
) {
    for asteroid in events.iter().map(|e| e.0) {
        let ctrl = match asteroids_query.get(asteroid) {
            Ok(&ShadowOf {
                controller: ctrl, ..
            }) => ctrl,
            Err(_) => asteroid,
        };

        log::warn!(?asteroid, ?ctrl, "remove asteroid");
        despawn_asteroid_in_explosion(
            &mut commands,
            &mut anim_events,
            &transform_query,
            ctrl,
            &shadows_query,
            &settings,
        );
    }
}

fn find_shot_asteroid_controller(
    mut shot_events: EventReader<AsteroidShotEvent>,
    mut ctrl_events: EventWriter<AsteroidControllerShotEvent>,
    mut sfx_event: EventWriter<SfxCmdEvent<SoundEffect>>,
    asteroids_query: Query<&ShadowOf>,
    transform_query: Query<&Transform>,
    win_bounds: Res<GfxBounds>,
) {
    for asteroid in shot_events.iter().map(|e| e.0) {
        let ctrl = match asteroids_query.get(asteroid) {
            Ok(&ShadowOf {
                controller: ctrl, ..
            }) => ctrl,
            Err(_) => asteroid,
        };
        log::info!(?asteroid, ?ctrl, "find ctrl");
        ctrl_events.send(AsteroidControllerShotEvent::new(ctrl));

        let transform = transform_query.get(asteroid).unwrap();
        let panning = (transform.translation.x + win_bounds.width() / 2.) / win_bounds.width();
        sfx_event.send(
            PlaySfx::new(SoundEffect::AsteroidExplode)
                .with_panning(panning)
                .into(),
        );
    }
}

fn count_shot_asteroid_controller(
    mut events: EventReader<AsteroidControllerShotEvent>,
    mut counter: ResMut<AsteroidCounter>,
) {
    for AsteroidControllerShotEvent(asteroid) in events.iter() {
        counter.shot += 1;
        log::info!(?asteroid, asteroids_shot = counter.shot);
    }
}

fn score_on_shot_asteroid_controller(
    mut events: EventReader<AsteroidControllerShotEvent>,
    mut scores_query: Query<&mut ScoreBoard>,
    bounds_query: Query<&GfxBounds>,
    settings: Res<Settings>,
) {
    for asteroid in events.iter().map(|e| e.0) {
        if let Ok(bounds) = bounds_query.get(asteroid) {
            for mut board in scores_query.iter_mut() {
                let a = (settings.asteroid.size_max - bounds.size().max_element())
                    / (settings.asteroid.size_max - settings.asteroid.size_min)
                    * settings.general.max_score;
                let score: &mut u32 = (*board).as_mut();
                *score += a as u32;
                log::info!(?asteroid, score, "update score");
            }
        } else {
            log::warn!(?asteroid, "no bounds for asteroid");
        }
    }
}

fn split_and_despawn_shot_asteroid_controller(
    mut events: EventReader<AsteroidControllerShotEvent>,
    mut remove_event: EventWriter<RemoveAsteroidEvent>,
    mut spawn_events: EventWriter<SpawnAsteroidEvent>,
    asteroids_query: Query<(&GfxBounds, &Transform), With<Asteroid>>,
    settings: Res<Settings>,
) {
    for asteroid in events.iter().map(|ev| ev as &Entity) {
        let (bounds, transform) = asteroids_query
            .get(*asteroid)
            .expect("asteroid not present");

        let max_size = bounds.size().max_element() * settings.asteroid.split_size_factor;
        if max_size >= settings.asteroid.size_min {
            let position = Some(transform.translation);
            log::info!(?asteroid, "split asteroid");
            for _ in 0..settings.asteroid.split_number {
                spawn_events.send(SpawnAsteroidEvent::new(max_size, position));
            }
        }

        remove_event.send(RemoveAsteroidEvent(*asteroid));
    }
}

fn asteroid_enter_ingame(
    mut commands: Commands,
    old_asteroids_query: Query<Entity, With<Asteroid>>,
    settings: Res<Settings>,
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
        .insert(AsteroidSpawnDelay(Duration::from_secs_f32(
            settings.asteroid.spawndelay_initial_seconds,
        )));
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

fn asteroid_spawner(
    mut commands: Commands,
    mut query: Query<(Entity, &mut AsteroidSpawnDelay, Option<&mut Timer>), With<AsteroidsSpawner>>,
    mut spawn_events: EventWriter<SpawnAsteroidEvent>,
    time: Res<Time>,
    settings: Res<Settings>,
) {
    match query.get_single_mut() {
        Ok((_, mut delay, Some(mut timer))) => {
            if timer.tick(time.delta()).finished() {
                log::debug!("timed asteroid");
                spawn_events.send(SpawnAsteroidEvent::new(
                    rand::thread_rng()
                        .gen_range(settings.asteroid.size_min..settings.asteroid.size_max),
                    None,
                ));
                log::warn!(duration=?delay.0, "time to next planned asteroid");
                timer.set_duration(delay.0);
                timer.reset();

                delay.0 = Duration::from_secs_f32(
                    (delay.0.as_secs_f32() * settings.asteroid.spawndelay_multiplier).clamp(
                        settings.asteroid.spawndelay_min_seconds,
                        settings.asteroid.spawndelay_initial_seconds,
                    ),
                );
            }
        }
        Ok((entity, delay, None)) => {
            log::debug!("not timed asteroid");
            spawn_events.send(SpawnAsteroidEvent::new(
                rand::thread_rng()
                    .gen_range(settings.asteroid.size_min..settings.asteroid.size_max),
                None,
            ));
            commands.entity(entity).insert(Timer::new(delay.0, false));
        }
        Err(_) => {}
    };
}

fn spawn_asteroid_on_event(
    mut events: EventReader<SpawnAsteroidEvent>,
    mut commands: Commands,
    mut counter: ResMut<AsteroidCounter>,
    mut material_assets: ResMut<Assets<ColorMaterial>>,
    textures: Res<TextureAssetMap<AsteroidTexture>>,
    player_tf_query: Query<&Transform, (With<Player>, With<ShadowController>)>,
    window_bounds: Res<GfxBounds>,
    settings: Res<Settings>,
) {
    let player_tf = player_tf_query.get_single().expect("player not present!");

    for SpawnAsteroidEvent(size, position) in
        events.iter().filter(|e| e.0 >= settings.asteroid.size_min)
    {
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
                    > settings.asteroid.spawn_player_distance
                {
                    break position;
                }
            }
            .extend(rng.gen_range(settings.asteroid.zpos_min..settings.asteroid.zpos_max))
        });

        let velocity = {
            let random_direction = rng.gen_range(0.0..(2. * PI));
            let random_speed =
                rng.gen_range(settings.asteroid.speed_min..settings.asteroid.speed_max);
            Quat::from_rotation_z(random_direction).mul_vec3(Vec3::Y) * random_speed
        };

        spawn_asteroid(
            *size,
            position,
            velocity.truncate(),
            &window_bounds,
            &textures,
            &mut commands,
            &mut material_assets,
        );
        counter.spawned += 1;
        log::info!(asteroids_spawned = counter.spawned);
    }
}

// region: helper functions

fn spawn_asteroid(
    size: f32,
    position: Vec3,
    velocity: Vec2,
    window_bounds: &GfxBounds,
    textures: &TextureAssetMap<AsteroidTexture>,
    commands: &mut Commands,
    material_assets: &mut Assets<ColorMaterial>,
) {
    let mut rng = rand::thread_rng();
    let asset_info = textures
        .get(AsteroidTexture(rng.gen_range(0..textures.len())))
        .expect("unable to get texture for asteroid");
    let material = material_assets.add(ColorMaterial::texture(asset_info.texture.clone()));

    let asteroid_scale = size / asset_info.size.max_element() as f32;
    let asteroid_size =
        Vec2::new(asset_info.size.x as f32, asset_info.size.y as f32) * asteroid_scale;
    let asteroid_id = commands
        .spawn_bundle(SpriteBundle {
            material: material.clone(),
            transform: Transform {
                translation: position,
                scale: Vec2::splat(asteroid_scale).extend(1.),
                ..Transform::default()
            },
            ..SpriteBundle::default()
        })
        .insert(Asteroid)
        .insert(GfxBounds::from_pos_and_size(
            position.truncate(),
            asteroid_size,
        ))
        .insert(ShadowController)
        .insert(Velocity::from(velocity))
        .insert(InsideWindow)
        .id();

    log::info!(asteroid=?asteroid_id, "asteroid spawned");

    spawn_display_shadows(
        asteroid_id,
        asteroid_size,
        asteroid_scale,
        material,
        &Some(|mut cmds: EntityCommands| {
            cmds.insert(Asteroid);
        }),
        window_bounds,
        commands,
    );
}

fn despawn_asteroid_in_explosion(
    commands: &mut Commands,
    anim_events: &mut EventWriter<AnimationEffect<Animation>>,
    transform_query: &Query<(&Transform, &GfxBounds)>,
    asteroid_ctrl: Entity,
    shadows_query: &Query<(Entity, &ShadowOf), With<Asteroid>>,
    settings: &Settings,
) {
    let (tf, bounds) = transform_query.get(asteroid_ctrl).unwrap();
    anim_events.send(AnimationEffect {
        key: Animation::BigExplosion,
        position: tf.translation,
        size: bounds.size().max_element(),
        fps: settings.general.animation_fps,
    });

    // despawn controller
    commands
        .entity(asteroid_ctrl)
        .remove_bundle::<(Asteroid, Velocity)>()
        .insert(Despawn);

    // despawn all children
    for entity in shadows_query.iter().filter_map(|(entity, shadowof)| {
        match asteroid_ctrl == shadowof.controller {
            true => Some(entity),
            false => None,
        }
    }) {
        commands
            .entity(entity)
            .remove_bundle::<(Asteroid, Velocity)>()
            .insert(Despawn);
    }
}

// endregion
