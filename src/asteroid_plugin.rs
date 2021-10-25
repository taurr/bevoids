use crate::{
    fade_plugin::Fadein, movement_plugin::Velocity, GameState, Textures, WinSize,
    ASTEROID_FADEIN_SECONDS, ASTEROID_MAX_SIZE, ASTEROID_MAX_SPEED, ASTEROID_MIN_SIZE,
    ASTEROID_MIN_SPEED, ASTEROID_SPAWN_SECONDS, ASTEROID_Z_MAX, ASTEROID_Z_MIN,
};
use bevy::{core::FixedTimestep, log, math::vec3, prelude::*};
use rand::Rng;
use std::f32::consts::PI;

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        // app.add_system_set(
        //     SystemSet::on_update(GameState::InGame)
        //         .with_run_criteria(FixedTimestep::step(ASTEROID_SPAWN_SECONDS as f64))
        //         .with_system(asteroid_spawn.system()),
        // );
    }
}

#[derive(Debug, Default, Component)]
pub struct Asteroid;
/*
#[allow(dead_code)]
pub fn split_asteroid(
    commands: &mut Commands,
    asteroid_max_size: &ScaleToMaxSize,
    asteroid_velocity: &Velocity,
    asteroid_position: Vec3,
    materials: &Res<Textures>,
    color_material_assets: &mut ResMut<Assets<ColorMaterial>>,
) {
    let max_size = **asteroid_max_size * 2. / 3.;
    if max_size >= ASTEROID_MIN_SIZE {
        let mut rng = rand::thread_rng();
        let direction = asteroid_velocity.normalize();
        let min_v = f32::min(max_size / ASTEROID_FADEIN_SECONDS, ASTEROID_MIN_SPEED);
        let velocity_1 = Quat::from_rotation_z(PI / 4.)
            .mul_vec3(direction * rng.gen_range(min_v..ASTEROID_MAX_SPEED));
        let velocity_2 = Quat::from_rotation_z(-PI / 4.)
            .mul_vec3(direction * rng.gen_range(min_v..ASTEROID_MAX_SPEED));
        spawn_explicit_asteroid(
            commands,
            max_size,
            asteroid_position, // + velocity_1.normalize() * max_size,
            velocity_1,
            materials,
            color_material_assets,
        );
        spawn_explicit_asteroid(
            commands,
            max_size,
            asteroid_position, // + velocity_2.normalize() * max_size,
            velocity_2,
            materials,
            color_material_assets,
        );
    }
}
*/

/*
pub fn spawn_explicit_asteroid(
    commands: &mut Commands,
    max_size: f32,
    position: Vec3,
    velocity: Vec3,
    textures: &Res<Textures>,
    color_material_assets: &mut ResMut<Assets<ColorMaterial>>,
) {
    if textures.asteroids.is_empty() {
        eprintln!("No textures for the asteroids!");
        return;
    }
    let mut rng = rand::thread_rng();
    let texture = textures.asteroids[rng.gen_range(0..textures.asteroids.len())].clone();
    let position = vec3(
        position.x,
        position.y,
        rng.gen_range(ASTEROID_Z_MIN..ASTEROID_Z_MAX),
    );

    log::info!("launching asteroid");
        commands
        .spawn_bundle(SpriteBundle {
            material: material_assets.add(textures.spaceship.clone().into()),
            transform: Transform {
                translation: random_position.extend(PLAYER_Z),
                rotation: random_rotation,
                scale: Vec2::splat(scale).extend(1.),
            },
            ..Default::default()
        })
        .insert(Player)
        .insert(Orientation(random_rotation))
        .insert(Velocity::new(velocity));

    commands
        .spawn()
        .insert_bundle(SpriteBundle {
            material: color_material_assets.add(texture.into()),
            transform: Transform {
                translation: position,
                ..Default::default()
            },
            visible: Visible {
                is_transparent: true,
                is_visible: false,
            },
            ..Default::default()
        })
        .insert(ScaleToMaxSize::new(max_size))
        //.insert(Fadein::from_secs_f32(ASTEROID_FADEIN_SECONDS))
        .insert(Asteroid)
        .insert(Velocity::new(velocity));
}
*/

/*
fn asteroid_spawn(
    kb: Res<Input<KeyCode>>,
    mut commands: Commands,
    win_size: Res<WinSize>,
    materials: Res<Textures>,
    mut color_material_assets: ResMut<Assets<ColorMaterial>>,
) {
    if kb.just_pressed(KeyCode::Escape) {
        let mut rng = rand::thread_rng();

        let angle = rng.gen_range(0.0..(2. * PI));
        let velocity = Quat::from_rotation_z(angle).mul_vec3(Vec3::Y)
            * rng.gen_range(ASTEROID_MIN_SPEED..ASTEROID_MAX_SPEED);
        let max_size = rng.gen_range(ASTEROID_MIN_SIZE..ASTEROID_MAX_SIZE);
        let target_position = {
            let w = win_size.0.x / 2.0 - 50.;
            let h = win_size.0.y / 2.0 - 50.;
            Vec3::new(rng.gen_range(-w..w), rng.gen_range(-h..h), 0.)
        };

        spawn_explicit_asteroid(
            &mut commands,
            max_size,
            target_position,
            velocity,
            &materials,
            &mut color_material_assets,
        );
    }
}
*/
