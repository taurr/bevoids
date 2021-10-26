use bevy::{log, prelude::*, sprite::collide_aabb::collide};

use crate::{
    asteroid_plugin::{split_asteroid, Asteroid},
    fade_plugin::Fadeout,
    movement_plugin::{ShadowController, ShadowOf, Velocity},
    player_plugin::{Bullet, Player},
    AsteroidMaterials, GameState, SpriteSize, WinSize, ASTEROID_FADEOUT_BULLET_SECONDS,
};

pub(crate) struct HitTestPlugin;

impl Plugin for HitTestPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameState::InGame)
                .with_system(shot_hit_asteroid.system())
                .with_system(asteroid_hit_player.system()),
        );
    }
}

fn shot_hit_asteroid(
    bullet_query: Query<(Entity, &Transform, &SpriteSize), With<Bullet>>,
    asteroids_query: Query<
        (
            Entity,
            &Transform,
            &SpriteSize,
            Option<&Velocity>,
            Option<&ShadowOf>,
        ),
        (With<Asteroid>, Without<Fadeout>),
    >,
    controller_query: Query<
        (Entity, Option<&Velocity>),
        (With<Asteroid>, With<ShadowController>, Without<Fadeout>),
    >,
    shadows_query: Query<(Entity, &ShadowOf), With<Asteroid>>,
    player_query: Query<&Transform, With<Player>>,
    win_size: Res<WinSize>,
    mut commands: Commands,
    mut materials: ResMut<AsteroidMaterials>,
) {
    let mut spent_bullets = vec![];
    let mut asteroids_hit = vec![];

    for player_tf in player_query.iter() {
        'bullet: for (bullet_entity, bullet_transform, bullet_size) in bullet_query.iter() {
            for (asteroid_entity, asteroid_transform, asteroid_size, velocity, shadowof) in
                asteroids_query.iter()
            {
                let (controller, velocity) = shadowof
                    .and_then(|shadowof| {
                        controller_query
                            .iter()
                            .find(|(controller, _)| controller == &shadowof.0)
                    })
                    .unwrap_or((asteroid_entity, velocity));

                if asteroids_hit.contains(&controller) {
                    continue;
                }

                if velocity.is_none() {
                    log::warn!(
                    "no velocity on controller - it's likely a double hit on a stopped asteroid"
                );
                    continue;
                }

                // TODO: take bullet orientation into account for collision check!
                if collide(
                    bullet_transform.translation,
                    (*bullet_size).into(),
                    asteroid_transform.translation,
                    (*asteroid_size).into(),
                )
                .is_some()
                {
                    log::debug!("bullet hits asteroid");
                    asteroids_hit.push(controller);
                    spent_bullets.push(bullet_entity);

                    split_asteroid(
                        &Vec2::from(*asteroid_size),
                        &asteroid_transform.translation,
                        &player_tf.translation,
                        &win_size,
                        &mut materials,
                        &mut commands,
                    );

                    continue 'bullet;
                }
            }
        }
    }

    for bullet in spent_bullets {
        commands.entity(bullet).despawn();
    }

    for asteroid in asteroids_hit {
        // TODO: once in a blue moon, we try to insert Fadeout on an entity that has been removed - why?

        // removing Asteroid component stops us from finding the asteroid again
        // removing the Velocity stops the asteroid movement
        // adding FadeOut fades the asteroids, and despawns when done!

        // do remember the "shadows" as well as the controller
        for entity in shadows_query
            .iter()
            .filter(|(_, shadowof)| asteroid == shadowof.0)
            .map(|(entity, _)| entity)
        {
            commands
                .entity(entity)
                .remove_bundle::<(Asteroid, Velocity)>()
                .insert(Fadeout::from_secs_f32(ASTEROID_FADEOUT_BULLET_SECONDS));
        }
        commands
            .entity(asteroid)
            .remove_bundle::<(Asteroid, Velocity)>()
            .insert(Fadeout::from_secs_f32(ASTEROID_FADEOUT_BULLET_SECONDS));
    }
}

fn asteroid_hit_player(
    asteroids_query: Query<
        (
            Entity,
            &Transform,
            &SpriteSize,
            Option<&Velocity>,
            Option<&ShadowOf>,
        ),
        (With<Asteroid>, Without<Fadeout>),
    >,
    controller_query: Query<
        (Entity, Option<&Velocity>),
        (With<Asteroid>, With<ShadowController>, Without<Fadeout>),
    >,
    shadows_query: Query<(Entity, &ShadowOf), With<Asteroid>>,
    player_query: Query<&Transform, With<Player>>,
    win_size: Res<WinSize>,
    mut commands: Commands,
    mut materials: ResMut<AsteroidMaterials>,
) {
    // TODO: implement this hit test
}
