use bevy::{log, prelude::*};
use parry2d::bounding_volume::BoundingVolume;

use crate::{
    plugins::{
        Asteroid, AsteroidShotEvent, InsideWindow, Laser, LaserSpentEvent, Player, PlayerDeadEvent,
        RemoveAsteroidEvent,
    },
    GameState, GfxBounds,
};

pub struct HitTestPlugin;

impl Plugin for HitTestPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameState::InGame)
                .label("hit-test")
                .with_system(shot_hit_asteroid)
                .with_system(asteroid_hit_player),
        );
    }
}

fn shot_hit_asteroid(
    laser_query: Query<(Entity, &GfxBounds), With<Laser>>,
    asteroids_query: Query<(Entity, &GfxBounds), (With<Asteroid>, With<InsideWindow>)>,
    mut asteroid_shot_events: EventWriter<AsteroidShotEvent>,
    mut laser_spent_events: EventWriter<LaserSpentEvent>,
) {
    'laser: for (laser_entity, laser_bounds) in laser_query.iter() {
        for (asteroid, asteroid_bounds) in asteroids_query.iter() {
            if laser_bounds
                .as_sphere()
                .intersects(&asteroid_bounds.as_sphere())
            {
                log::info!(?asteroid, "laser hit asteroid");
                asteroid_shot_events.send(AsteroidShotEvent::new(asteroid));
                laser_spent_events.send(LaserSpentEvent::new(laser_entity));
                continue 'laser;
            }
        }
    }
}

fn asteroid_hit_player(
    player_query: Query<&GfxBounds, (With<Player>, With<InsideWindow>)>,
    asteroids_query: Query<(Entity, &GfxBounds), (With<Asteroid>, With<InsideWindow>)>,
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
