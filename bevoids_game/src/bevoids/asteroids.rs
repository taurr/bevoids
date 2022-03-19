use bevoids_assets::{AsteroidAsset, SoundAsset, SpriteAsset};
use bevy::{ecs::system::EntityCommands, log, prelude::*};
use bevy_effects::{
    animation::{SpawnSpriteAnimation, SpriteAnimation, TextureAtlasMap},
    despawn::Despawn,
    sound::{PlaySfx, SfxCmdEvent},
};
use bevy_embasset::EnumCount;
use derive_more::{Constructor, Deref};
use itertools::Itertools;
use rand::Rng;
use std::{f32::consts::PI, time::Duration};

use crate::{
    bevoids::highscore::{AddScoreEvent, Score},
    bounds::GfxBounds,
};

use super::{
    movement::{spawn_display_shadows, InsideWindow, ShadowController, ShadowOf, Velocity},
    player::Player,
    settings::Settings,
};

/// Remove an asteroid  - no points
#[derive(Debug, Clone, Copy, Deref, Constructor)]
pub(crate) struct AsteroidExplosionEvent(Entity);

// Asteroid has been shot - points + split
#[derive(Debug, Clone, Copy, Deref, Constructor)]
pub(crate) struct AsteroidShotEvent(Entity);

#[derive(Debug, Clone, Copy, Constructor)]
pub(crate) struct SpawnAsteroidEvent {
    size: f32,
    position: Option<Vec3>,
    is_background: bool,
}

// Marks an entity as an asteroid
#[derive(Debug, Copy, Clone, Component)]
pub(crate) struct Asteroid;

// Marks an entity as an asteroid
#[derive(Debug, Copy, Clone, Component)]
pub(crate) struct BackgroundAsteroid;

#[derive(Default)]
pub(crate) struct AsteroidCounter {
    spawned: usize,
    shot: usize,
}

#[derive(Debug, Component)]
pub(crate) struct AsteroidsSpawner {
    delay: Duration,
    timer: Timer,
    paused: bool,
}

pub(crate) fn spawn_asteroid_spawner_system(mut commands: Commands, settings: Res<Settings>) {
    // start spawning new asteroid entities
    let delay = settings.asteroid.spawndelay_initial;
    commands.spawn().insert(AsteroidsSpawner {
        delay,
        timer: Timer::new(delay, false),
        paused: false,
    });
}

pub(crate) fn asteroid_spawner_system(
    mut spawner_query: Query<&mut AsteroidsSpawner>,
    mut spawn_event: EventWriter<SpawnAsteroidEvent>,
    asteroids_query: Query<&Asteroid>,
    settings: Res<Settings>,
    time: Res<Time>,
) {
    let mut spawner_data = spawner_query.iter_mut().next().unwrap();

    match (
        asteroids_query.iter().next().is_none(),
        spawner_data.paused,
        spawner_data.timer.tick(time.delta()).finished(),
    ) {
        (true, false, _) => {
            // reset timer with a short timeout to NOT spawn asteroid because player crashed!
            let delay = settings.asteroid.spawndelay;
            spawner_data.timer.set_duration(delay);
            spawner_data.timer.reset();
            spawner_data.paused = true;
            log::debug!("field empty - respiratory pause");
        }
        (true, true, false) => { /* No asteroids in the field, but we're on top of it, waiting for timer */
        }
        (true, true, true) => {
            // we're here after user has cleared the field + a small pause
            let delay = spawner_data.delay;
            spawner_data.timer.set_duration(delay);
            spawner_data.timer.reset();
            spawner_data.paused = false;
            log::warn!("field empty - spawning asteroid");
            spawn_event.send(SpawnAsteroidEvent::new(
                rand::thread_rng()
                    .gen_range(settings.asteroid.size_min..settings.asteroid.size_max),
                None,
                false,
            ));
        }
        (false, false, true) => {
            // start timer again, new timeout
            let delay = Duration::from_secs_f32(
                (spawner_data.delay.as_secs_f32() * settings.asteroid.spawndelay_multiplier).clamp(
                    settings.asteroid.spawndelay_min.as_secs_f32(),
                    settings.asteroid.spawndelay_initial.as_secs_f32(),
                ),
            );
            spawner_data.timer.set_duration(delay);
            spawner_data.timer.reset();
            spawner_data.delay = delay;
            log::warn!(duration=?delay, "spawning planned asteroid");
            spawn_event.send(SpawnAsteroidEvent::new(
                rand::thread_rng()
                    .gen_range(settings.asteroid.size_min..settings.asteroid.size_max),
                None,
                false,
            ));
        }
        _ => {}
    };
}

pub(crate) fn shot_asteroid_system(
    mut shot_events: EventReader<AsteroidShotEvent>,
    mut spawn_event: EventWriter<SpawnAsteroidEvent>,
    mut remove_event: EventWriter<AsteroidExplosionEvent>,
    mut score_event: EventWriter<AddScoreEvent>,
    mut counter: ResMut<AsteroidCounter>,
    transform_and_bounds_query: Query<(&Transform, &GfxBounds), With<Asteroid>>,
    shadowof_query: Query<&ShadowOf, With<Asteroid>>,
    settings: Res<Settings>,
) {
    let shot_asteroids = shot_events
        .iter()
        .map(|AsteroidShotEvent(e)| match shadowof_query.get(*e) {
            Ok(&ShadowOf {
                controller: ctrl, ..
            }) => ctrl,
            Err(_) => *e,
        })
        .unique();

    for (asteroid, asteroid_tf, asteroid_bounds) in
        shot_asteroids.filter_map(|e| match transform_and_bounds_query.get(e) {
            Ok((t, b)) => Some((e, t, b)),
            Err(_) => None,
        })
    {
        // update counter
        counter.shot += 1;
        log::info!(asteroids_shot = counter.shot);

        // add score
        score_event.send(AddScoreEvent(Score::new(
            ((settings.asteroid.size_max - asteroid_bounds.size().max_element())
                / (settings.asteroid.size_max - settings.asteroid.size_min)
                * settings.asteroid.max_score) as u32,
        )));

        // spawn split asteroids
        let split_size = asteroid_bounds.size().max_element() * settings.asteroid.split_size_factor;
        if split_size >= settings.asteroid.size_min {
            log::debug!(?asteroid, "split asteroid");
            for _ in 0..settings.asteroid.split_number {
                spawn_event.send(SpawnAsteroidEvent::new(
                    split_size,
                    Some(asteroid_tf.translation),
                    false,
                ));
            }
        }

        remove_event.send(AsteroidExplosionEvent(asteroid));
    }
}

pub(crate) fn despawn_asteroid_spawner_system(
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

pub(crate) fn spawn_asteroid_event_system(
    mut spawn_asteroid_events: EventReader<SpawnAsteroidEvent>,
    mut commands: Commands,
    mut counter: Option<ResMut<AsteroidCounter>>,
    asset_server: Res<AssetServer>,
    player_tf_query: Query<&Transform, (With<Player>, With<ShadowController>)>,
    window_bounds: Res<GfxBounds>,
    settings: Res<Settings>,
) {
    let player_tf = player_tf_query.iter().next();

    for SpawnAsteroidEvent {
        size,
        position,
        is_background,
    } in spawn_asteroid_events
        .iter()
        .filter(|&e| e.size >= settings.asteroid.size_min)
    {
        let mut rng = rand::thread_rng();

        let position = position.unwrap_or_else(|| {
            random_2d_position_no_closer_than(
                player_tf,
                settings.asteroid.spawn_player_distance,
                &window_bounds,
            )
            .extend(rng.gen_range(settings.asteroid.zpos_min..settings.asteroid.zpos_max))
        });
        let velocity = {
            let random_direction = rng.gen_range(0.0..(2. * PI));
            let random_speed =
                rng.gen_range(settings.asteroid.speed_min..settings.asteroid.speed_max);
            Quat::from_rotation_z(random_direction)
                .mul_vec3(Vec3::Y)
                .truncate()
                * random_speed
        };

        let texture = asset_server.load(
            AsteroidAsset::iter()
                .nth(rng.gen_range(0..(AsteroidAsset::COUNT - 1)))
                .unwrap(),
        );
        let custom_size = Vec2::splat(*size);
        let asteroid_id = commands
            .spawn_bundle(SpriteBundle {
                texture: texture.clone(),
                transform: Transform {
                    translation: position,
                    ..Transform::default()
                },
                sprite: Sprite {
                    custom_size: Some(custom_size),
                    ..Default::default()
                },
                ..SpriteBundle::default()
            })
            .insert(GfxBounds::from_pos_and_size(
                position.truncate(),
                custom_size,
            ))
            .insert(ShadowController)
            .insert(Velocity::from(velocity))
            .insert(InsideWindow)
            .id();
        if *is_background {
            commands.entity(asteroid_id).insert(BackgroundAsteroid);
        } else {
            commands.entity(asteroid_id).insert(Asteroid);
        };

        spawn_display_shadows(
            asteroid_id,
            custom_size,
            texture,
            &Some(|mut cmds: EntityCommands| {
                if *is_background {
                    cmds.insert(BackgroundAsteroid);
                } else {
                    cmds.insert(Asteroid);
                };
            }),
            &window_bounds,
            &mut commands,
        );

        if let Some(counter) = counter.as_mut() {
            counter.spawned += 1;
            log::debug!(asteroid=?asteroid_id, asteroids_spawned = counter.spawned, "asteroid spawned");
        }
    }
}

fn random_2d_position_no_closer_than(
    position: Option<&Transform>,
    distance: f32,
    window_bounds: &Res<GfxBounds>,
) -> Vec2 {
    let mut rng = rand::thread_rng();
    loop {
        let rnd_position = {
            let (w, h) = (window_bounds.width() / 2.0, window_bounds.height() / 2.0);
            Vec2::new(rng.gen_range(-w..w), rng.gen_range(-h..h))
        };
        if let Some(player_tf) = position {
            if rnd_position
                .extend(player_tf.translation.z)
                .distance(player_tf.translation)
                > distance
            {
                break rnd_position;
            }
        } else {
            break rnd_position;
        }
    }
}

pub(crate) fn asteroid_explosion_system(
    mut remove_events: EventReader<AsteroidExplosionEvent>,
    mut sfx_event: EventWriter<SfxCmdEvent<SoundAsset>>,
    mut commands: Commands,
    texture_atlas_map: Res<TextureAtlasMap>,
    transform_and_bounds_query: Query<(&Transform, &GfxBounds), With<Asteroid>>,
    shadows_query: Query<(Entity, &ShadowOf), With<Asteroid>>,
    settings: Res<Settings>,
    win_bounds: Res<GfxBounds>,
) {
    let asteroids = remove_events
        .iter()
        .map(
            |AsteroidExplosionEvent(e)| match shadows_query.get(*e).map(|x| x.1) {
                Ok(&ShadowOf {
                    controller: ctrl, ..
                }) => ctrl,
                Err(_) => *e,
            },
        )
        .unique();

    let explosion_atlas = texture_atlas_map.get(SpriteAsset::GfxExplosion).unwrap();

    for (asteroid, asteroid_tf, asteroid_bounds) in
        asteroids.filter_map(|e| match transform_and_bounds_query.get(e) {
            Ok((t, b)) => Some((e, t, b)),
            Err(_) => None,
        })
    {
        log::debug!(?asteroid, "asteroid exploding");

        let mut anim_position = asteroid_tf.translation;
        anim_position.z -= 1.;

        // display explosion
        // TODO: we need to stop the anim!
        commands.spawn_sprite_animation(
            explosion_atlas,
            SpriteAnimation {
                fps: settings.general.animation_fps,
                position: anim_position,
                size: Some(asteroid_bounds.size()),
                ..Default::default()
            },
        );

        // play explosion
        sfx_event.send(
            PlaySfx::new(SoundAsset::AsteroidExplode)
                .with_panning({
                    let (transform, _) = transform_and_bounds_query.get(asteroid).unwrap();
                    (transform.translation.x + win_bounds.width() / 2.) / win_bounds.width()
                })
                .into(),
        );

        // despawn controller
        commands
            .entity(asteroid)
            .remove_bundle::<(Asteroid, Velocity)>()
            .insert(Despawn);

        // despawn all shadows
        for entity in shadows_query.iter().filter_map(|(entity, shadowof)| {
            match shadowof.controller == asteroid {
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
}
