use bevy::prelude::*;
use parry2d::bounding_volume::BoundingVolume;

use crate::{
    plugins::{
        Asteroid, AsteroidShotEvent, Bullet, BulletSpentEvent, InsideWindow, Player,
        PlayerDeadEvent, RemoveAsteroidEvent,
    },
    Bounds, GameState,
};

pub struct HitTestPlugin;

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
    bullet_query: Query<(Entity, &Bounds), With<Bullet>>,
    asteroids_query: Query<(Entity, &Bounds), (With<Asteroid>, With<InsideWindow>)>,
    mut asteroid_shot_events: EventWriter<AsteroidShotEvent>,
    mut bullet_spent_events: EventWriter<BulletSpentEvent>,
) {
    'bullet: for (bullet_entity, bullet_bounds) in bullet_query.iter() {
        for (asteroid, asteroid_bounds) in asteroids_query.iter() {
            if bullet_bounds
                .as_sphere()
                .intersects(&asteroid_bounds.as_sphere())
            {
                asteroid_shot_events.send(AsteroidShotEvent::new(asteroid));
                bullet_spent_events.send(BulletSpentEvent::new(bullet_entity));
                continue 'bullet;
            }
        }
    }
}

fn asteroid_hit_player(
    player_query: Query<&Bounds, (With<Player>, With<InsideWindow>)>,
    asteroids_query: Query<(Entity, &Bounds), (With<Asteroid>, With<InsideWindow>)>,
    mut player_dead_events: EventWriter<PlayerDeadEvent>,
    mut remove_asteroid_events: EventWriter<RemoveAsteroidEvent>,
) {
    'player: for player_bounds in player_query.iter() {
        let player_sphere = player_bounds.as_sphere();

        for (asteroid, asteroid_bounds) in asteroids_query.iter() {
            let asteroid_sphere = asteroid_bounds.as_sphere();
            if player_sphere.intersects(&asteroid_sphere) {
                player_dead_events.send(PlayerDeadEvent);
                remove_asteroid_events.send(RemoveAsteroidEvent::new(asteroid));
                continue 'player;
            }
        }
    }
}
