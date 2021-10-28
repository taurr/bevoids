use bevy::{log, prelude::*, sprite::collide_aabb::collide};

use crate::{
    asteroid_plugin::{despawn_asteroid, spawn_split_asteroids, Asteroid},
    fade_plugin::Fadeout,
    movement_plugin::{ShadowController, ShadowOf, Velocity},
    player_plugin::{bullet_spent, kill_player, Bullet, Player},
    AsteroidMaterials, Despawn, GameState, SpriteSize, WinSize,
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

fn rotate_bounding_box(size: Vec2, rotation: Quat) -> Vec2 {
    // get 4 corners around the position
    let (p1, p2, p3, p4) = {
        let (w, h) = (size / 2.0).into();
        (
            Vec3::from((-w, h, 0.0)),
            Vec3::from((w, h, 0.0)),
            Vec3::from((-w, -h, 0.0)),
            Vec3::from((w, -h, 0.0)),
        )
    };
    // rotate each corner
    let (p1, p2, p3, p4) = (
        rotation.mul_vec3(p1),
        rotation.mul_vec3(p2),
        rotation.mul_vec3(p3),
        rotation.mul_vec3(p4),
    );
    // we only need 2D
    let (p1, p2, p3, p4) = (p1.truncate(), p2.truncate(), p3.truncate(), p4.truncate());
    // final bounding box around position
    p1.max(p2).max(p3).max(p4) * 2.0
}

fn shot_hit_asteroid(
    bullet_query: Query<(Entity, &Transform, &SpriteSize), (With<Bullet>, Without<Despawn>)>,
    asteroids_query: Query<
        (
            Entity,
            &Transform,
            &SpriteSize,
            Option<&Velocity>,
            Option<&ShadowOf>,
        ),
        (With<Asteroid>, Without<Fadeout>, Without<Despawn>),
    >,
    controller_query: Query<
        (Entity, Option<&Velocity>),
        (
            With<Asteroid>,
            With<ShadowController>,
            Without<Fadeout>,
            Without<Despawn>,
        ),
    >,
    shadows_query: Query<(Entity, &ShadowOf), (With<Asteroid>, Without<Fadeout>, Without<Despawn>)>,
    player_query: Query<&Transform, With<Player>>,
    win_size: Res<WinSize>,
    mut commands: Commands,
    mut materials: ResMut<AsteroidMaterials>,
) {
    let mut spent_bullets = vec![];
    let mut asteroids_hit = vec![];

    for player_tf in player_query.iter() {
        'bullet: for (bullet_entity, bullet_transform, bullet_size) in bullet_query.iter() {
            'asteroid: for (asteroid, asteroid_transform, asteroid_size, velocity, shadowof) in
                asteroids_query.iter()
            {
                let (asteroid_ctrl, velocity) = shadowof
                    .and_then(|shadowof| {
                        controller_query
                            .iter()
                            .find(|(controller, _)| controller == &shadowof.0)
                    })
                    .unwrap_or((asteroid, velocity));

                if asteroids_hit.contains(&asteroid_ctrl) || velocity.is_none() {
                    continue 'asteroid;
                }

                if collide(
                    bullet_transform.translation,
                    rotate_bounding_box((*bullet_size).into(), bullet_transform.rotation),
                    asteroid_transform.translation,
                    (*asteroid_size).into(),
                )
                .is_some()
                {
                    log::info!(?asteroid_ctrl, ?asteroid, "bullet hit",);
                    asteroids_hit.push(asteroid_ctrl);
                    spent_bullets.push(bullet_entity);

                    spawn_split_asteroids(
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
        bullet_spent(&mut commands, bullet);
    }

    for asteroid in asteroids_hit {
        despawn_asteroid(&mut commands, asteroid, &shadows_query);
    }
}

fn asteroid_hit_player(
    player_query: Query<
        (
            Entity,
            &Transform,
            &SpriteSize,
            Option<&ShadowOf>,
            Option<&ShadowController>,
        ),
        (With<Player>, Without<Fadeout>, Without<Despawn>),
    >,
    asteroids_query: Query<
        (Entity, &Transform, &SpriteSize, Option<&ShadowOf>),
        (With<Asteroid>, Without<Fadeout>, Without<Despawn>),
    >,
    player_ctrl_query: Query<
        Entity,
        (
            With<Player>,
            With<ShadowController>,
            Without<Fadeout>,
            Without<Despawn>,
        ),
    >,
    asteroid_ctrl_query: Query<
        Entity,
        (
            With<Asteroid>,
            With<ShadowController>,
            Without<Fadeout>,
            Without<Despawn>,
        ),
    >,
    shadows_query: Query<(Entity, &ShadowOf), (With<Asteroid>, Without<Fadeout>, Without<Despawn>)>,
    mut commands: Commands,
) {
    let mut players_hit = vec![];
    let mut asteroids_hit = vec![];

    'player: for (player, player_tf, player_size, shadowof, controller) in player_query.iter() {
        if let Some(player_ctrl) = shadowof
            .and_then(|shadowof| {
                player_ctrl_query
                    .iter()
                    .find(|player_ctrl| player_ctrl == &shadowof.0)
            })
            .or_else(|| controller.map(|_| player))
        {
            'asteroid: for (asteroid, asteroid_transform, asteroid_size, shadowof) in
                asteroids_query.iter()
            {
                let asteroid_ctrl = shadowof
                    .and_then(|shadowof| {
                        asteroid_ctrl_query
                            .iter()
                            .find(|controller| controller == &shadowof.0)
                    })
                    .unwrap_or(asteroid);

                if asteroids_hit.contains(&asteroid_ctrl) {
                    continue 'asteroid;
                }

                if collide(
                    player_tf.translation,
                    rotate_bounding_box((*player_size).into(), player_tf.rotation),
                    asteroid_transform.translation,
                    (*asteroid_size).into(),
                )
                .is_some()
                {
                    log::info!(
                        ?player_ctrl,
                        ?player,
                        ?asteroid_ctrl,
                        ?asteroid,
                        "player hit!",
                    );
                    asteroids_hit.push(asteroid_ctrl);
                    players_hit.push(player_ctrl);

                    continue 'player;
                }
            }
        }
    }

    for player in players_hit {
        kill_player(&mut commands, player);
    }

    for asteroid in asteroids_hit {
        despawn_asteroid(&mut commands, asteroid, &shadows_query);
    }
}
