use bevy::{log, prelude::*};
use parry2d::bounding_volume::BoundingVolume;

use crate::{
    asteroid_plugin::{despawn_asteroid, spawn_split_asteroids, Asteroid},
    constants::{ASTEROID_MAX_SCORE, ASTEROID_MAX_SIZE, ASTEROID_MIN_SIZE},
    fade_despawn_plugin::{Despawn, FadeDespawn},
    movement_plugin::{InsideWindow, ShadowController, ShadowOf},
    player_plugin::{bullet_spent, kill_player, Bullet, Player},
    scoreboard::ScoreBoard,
    textures::AsteroidMaterials,
    Bounds, GameState,
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
    player_query: Query<&Transform, (With<Player>, With<InsideWindow>)>,
    bullet_query: Query<(Entity, &Bounds), (With<Bullet>, Without<Despawn>)>,
    asteroids_query: Query<
        (Entity, &Transform, &Bounds, Option<&ShadowOf>),
        (
            With<Asteroid>,
            Without<FadeDespawn>,
            Without<Despawn>,
            With<InsideWindow>,
        ),
    >,
    asteroid_ctrl_query: Query<
        Entity,
        (
            With<Asteroid>,
            With<ShadowController>,
            Without<FadeDespawn>,
            Without<Despawn>,
        ),
    >,
    shadows_query: Query<
        (Entity, &ShadowOf),
        (With<Asteroid>, Without<FadeDespawn>, Without<Despawn>),
    >,
    window_bounds: Res<Bounds>,
    mut scores_query: Query<&mut ScoreBoard>,
    mut commands: Commands,
    mut materials: ResMut<AsteroidMaterials>,
) {
    let mut spent_bullets = vec![];
    let mut asteroids_hit = vec![];

    for player_tf in player_query.iter() {
        'bullet: for (bullet_entity, bullet_bounds) in bullet_query.iter() {
            let bullet_sphere = bullet_bounds.as_sphere();
            'asteroid: for (asteroid, asteroid_tf, asteroid_bounds, shadowof) in
                asteroids_query.iter()
            {
                let asteroid_ctrl = shadowof
                    .and_then(|shadowof| {
                        asteroid_ctrl_query
                            .iter()
                            .find(|controller| controller == &shadowof.controller)
                    })
                    .unwrap_or(asteroid);

                if asteroids_hit.contains(&asteroid_ctrl) {
                    continue 'asteroid;
                }

                if bullet_sphere.intersects(&asteroid_bounds.as_sphere()) {
                    log::info!(?asteroid_ctrl, ?asteroid, "bullet hit",);
                    for mut board in scores_query.iter_mut() {
                        let a = (ASTEROID_MAX_SIZE - asteroid_bounds.size().max_element())
                            / (ASTEROID_MAX_SIZE - ASTEROID_MIN_SIZE)
                            * ASTEROID_MAX_SCORE;
                        let score: &mut u32 = (*board).as_mut();
                        *score += a as u32;
                    }
                    asteroids_hit.push(asteroid_ctrl);
                    spent_bullets.push(bullet_entity);

                    spawn_split_asteroids(
                        asteroid_bounds.size(),
                        &asteroid_tf.translation,
                        &player_tf.translation,
                        &window_bounds,
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
            &Bounds,
            Option<&ShadowOf>,
            Option<&ShadowController>,
        ),
        (
            With<Player>,
            Without<FadeDespawn>,
            Without<Despawn>,
            With<InsideWindow>,
        ),
    >,
    asteroids_query: Query<
        (Entity, &Bounds, Option<&ShadowOf>),
        (
            With<Asteroid>,
            Without<FadeDespawn>,
            Without<Despawn>,
            With<InsideWindow>,
        ),
    >,
    player_ctrl_query: Query<
        Entity,
        (
            With<Player>,
            With<ShadowController>,
            Without<FadeDespawn>,
            Without<Despawn>,
        ),
    >,
    asteroid_ctrl_query: Query<
        Entity,
        (
            With<Asteroid>,
            With<ShadowController>,
            Without<FadeDespawn>,
            Without<Despawn>,
        ),
    >,
    shadows_query: Query<
        (Entity, &ShadowOf),
        (With<Asteroid>, Without<FadeDespawn>, Without<Despawn>),
    >,
    mut state: ResMut<State<GameState>>,
    mut commands: Commands,
) {
    let mut players_hit = vec![];
    let mut asteroids_hit = vec![];

    'player: for (player, player_bounds, shadowof, controller) in player_query.iter() {
        if let Some(player_ctrl) = shadowof
            .and_then(|shadowof| {
                player_ctrl_query
                    .iter()
                    .find(|player_ctrl| player_ctrl == &shadowof.controller)
            })
            .or_else(|| controller.map(|_| player))
        {
            let player_sphere = player_bounds.as_sphere();
            'asteroid: for (asteroid, asteroid_bounds, shadowof) in asteroids_query.iter() {
                let asteroid_ctrl = shadowof
                    .and_then(|shadowof| {
                        asteroid_ctrl_query
                            .iter()
                            .find(|controller| controller == &shadowof.controller)
                    })
                    .unwrap_or(asteroid);

                if asteroids_hit.contains(&asteroid_ctrl) {
                    continue 'asteroid;
                }

                let asteroid_sphere = asteroid_bounds.as_sphere();
                if player_sphere.intersects(&asteroid_sphere) {
                    log::debug!(?player_sphere, ?asteroid_sphere);
                    log::info!(
                        ?player_ctrl,
                        ?player,
                        ?asteroid_ctrl,
                        ?asteroid,
                        "player hit!",
                    );
                    asteroids_hit.push(asteroid_ctrl);
                    players_hit.push(player_ctrl);

                    state.set(GameState::GameOver).unwrap();

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
