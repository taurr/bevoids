use bevy::{log, prelude::*};
use bevy_asset_map::GfxBounds;
use bevy_effects::despawn::Despawn;
use parry2d::bounding_volume::BoundingVolume;

use super::{
    asteroids::{Asteroid, AsteroidExplosionEvent, AsteroidShotEvent},
    laser::Laser,
    movement::InsideWindow,
    player::{Player, PlayerDeadEvent},
};

pub(crate) fn hittest_shot_vs_asteroid(
    mut commands: Commands,
    laser_query: Query<(Entity, &GfxBounds), With<Laser>>,
    asteroids_query: Query<(Entity, &GfxBounds), (With<Asteroid>, With<InsideWindow>)>,
    mut asteroid_shot_event: EventWriter<AsteroidShotEvent>,
) {
    'laser: for (laser_entity, laser_bounds) in laser_query.iter() {
        for (asteroid, asteroid_bounds) in asteroids_query.iter() {
            if laser_bounds
                .as_sphere()
                .intersects(&asteroid_bounds.as_sphere())
            {
                log::debug!(?asteroid, "laser hit asteroid");
                asteroid_shot_event.send(AsteroidShotEvent::new(asteroid));
                commands.entity(laser_entity).insert(Despawn);
                continue 'laser;
            }
        }
    }
}

pub(crate) fn hittest_player_vs_asteroid(
    player_query: Query<&GfxBounds, (With<Player>, With<InsideWindow>)>,
    asteroids_query: Query<(Entity, &GfxBounds), (With<Asteroid>, With<InsideWindow>)>,
    mut player_dead_event: EventWriter<PlayerDeadEvent>,
    mut remove_asteroid_event: EventWriter<AsteroidExplosionEvent>,
) {
    'player: for player_bounds in player_query.iter() {
        let player_sphere = player_bounds.as_sphere();

        for (asteroid, asteroid_bounds) in asteroids_query.iter() {
            let asteroid_sphere = asteroid_bounds.as_sphere();
            if player_sphere.intersects(&asteroid_sphere) {
                player_dead_event.send(PlayerDeadEvent);
                remove_asteroid_event.send(AsteroidExplosionEvent::new(asteroid));
                continue 'player;
            }
        }
    }
}
