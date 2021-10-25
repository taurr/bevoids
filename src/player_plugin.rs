use crate::{
    fade_plugin::DelayedFadeout, movement_plugin::Velocity, GameState, Textures, WinSize,
    BULLET_FADEOUT_SECONDS, BULLET_LIFETIME_SECONDS, BULLET_MAX_SIZE, BULLET_RELATIVE_Y,
    BULLET_RELATIVE_Z, BULLET_SPEED, FLAME_OPACITY, FLAME_RELATIVE_Y, FLAME_RELATIVE_Z,
    FLAME_WIDTH, PLAYER_ACCELLERATION, PLAYER_MAX_SIZE, PLAYER_MAX_SPEED, PLAYER_START_SPEED,
    PLAYER_TURN_SPEED, PLAYER_Z,
};
use bevy::{core::FixedTimestep, log, math::vec3, prelude::*};
use rand::Rng;
use std::{f32::consts::PI, time::Duration};

pub struct PlayerPlugin;

#[derive(Debug, Default, Component)]
pub struct Player;

#[derive(Debug, Default, Component)]
pub struct Bullet;

#[derive(Debug, Default, Component)]
pub struct Orientation(pub Quat);

#[derive(Debug, Default, Component)]
pub struct Flame;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(GameState::InGame).with_system(player_spawn.system()),
        );
        app.add_system_set(
            SystemSet::on_update(GameState::InGame).with_system(player_controls.system()),
        );
        app.add_system_set(
             SystemSet::on_update(GameState::InGame)
                 .with_run_criteria(FixedTimestep::step(1.0 as f64))
                 .with_system(log_player_stats.system()));
    }
}

fn player_spawn(
    mut commands: Commands,
    win_size: Res<WinSize>,
    textures: Res<Textures>,
    mut material_assets: ResMut<Assets<ColorMaterial>>,
) {
    log::debug!("player created");
    let mut rng = rand::thread_rng();

    let [w, h] = (win_size.0 * 2. / 6.).to_array();
    let random_position = Vec2::new(rng.gen_range(-w..w), rng.gen_range(-h..h));
    let random_rotation = Quat::from_rotation_z(rng.gen_range(0.0..(2. * PI)));
    let scale = PLAYER_MAX_SIZE
        / textures
            .get_size(&textures.spaceship)
            .unwrap()
            .max_element();
    let velocity = random_rotation.mul_vec3(Vec3::Y) * PLAYER_START_SPEED;

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
}

fn player_controls(
    mut commands: Commands,
    kb: Res<Input<KeyCode>>,
    mut player_query: Query<
        (Entity, &mut Velocity, &mut Orientation, &mut Transform),
        With<Player>,
    >,
    flame_query: Query<Entity, With<Flame>>,
    textures: Res<Textures>,
    mut material_assets: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
) {
    for (player, mut player_velocity, mut player_orientation, mut player_transform) in
        player_query.iter_mut()
    {
        // orientation
        if kb.pressed(KeyCode::Left) {
            player_orientation.0 = player_orientation.0.mul_quat(Quat::from_rotation_z(
                PLAYER_TURN_SPEED * time.delta_seconds(),
            ));
        } else if kb.pressed(KeyCode::Right) {
            player_orientation.0 = player_orientation.0.mul_quat(Quat::from_rotation_z(
                -PLAYER_TURN_SPEED * time.delta_seconds(),
            ));
        }
        player_transform.rotation = player_orientation.0;

        if kb.pressed(KeyCode::Up) {
            // accelleration
            let v = player_orientation
                .0
                .mul_vec3(vec3(0., PLAYER_ACCELLERATION, 0.))
                * time.delta_seconds();
            let velocity = **player_velocity + v;
            let capped_velocity = if velocity.length() > PLAYER_MAX_SPEED {
                velocity.normalize() * PLAYER_MAX_SPEED
            } else {
                velocity
            };
            **player_velocity = capped_velocity;
        } else {
            // decellerate
            if player_velocity.length() > 0. {
                let v = player_velocity.normalize()
                    * f32::min(
                        PLAYER_ACCELLERATION * 0.5 * time.delta_seconds(),
                        player_velocity.length(),
                    );
                **player_velocity -= v;
            }
        }
        if kb.just_pressed(KeyCode::Up) {
            log::trace!("accellerate on");
            let flame = spawn_flame(
                &mut commands,
                &textures,
                &mut material_assets,
                &player_transform,
            );
            commands.entity(player).push_children(&[flame]);
        }
        if kb.just_released(KeyCode::Up) {
            log::trace!("accellerate off");
            for flame in flame_query.iter() {
                commands.entity(flame).despawn();
            }
        }

        // fire
        if kb.just_pressed(KeyCode::Space) {
            log::debug!("fire!");
            spawn_bullet(
                &mut commands,
                &textures,
                &mut material_assets,
                &player_transform,
                &player_orientation,
            );
        }
    }
}

fn spawn_bullet(
    commands: &mut Commands,
    textures: &Textures,
    material_assets: &mut Assets<ColorMaterial>,
    player_transform: &Transform,
    player_orientation: &Orientation,
) {
    let texture = textures.shot.clone();
    let scale = BULLET_MAX_SIZE / textures.get_size(&texture).unwrap().max_element();
    commands
        .spawn_bundle(SpriteBundle {
            // TODO: ColorMaterial should not be created each time we show the flame
            material: material_assets.add(ColorMaterial::texture(textures.shot.clone())),
            // TODO: Transform should not be created each time we show the flame
            transform: Transform {
                translation: player_transform.translation
                    + player_orientation
                        .0
                        .mul_vec3(vec3(0., BULLET_RELATIVE_Y, BULLET_RELATIVE_Z)),
                rotation: Quat::from_rotation_z(PI / 2.).mul_quat(player_orientation.0),
                scale: Vec2::splat(scale).extend(1.),
            },
            ..Default::default()
        })
        .insert(Bullet)
        .insert(DelayedFadeout::new(
            Duration::from_secs_f32(BULLET_LIFETIME_SECONDS),
            Duration::from_secs_f32(BULLET_FADEOUT_SECONDS),
        ))
        .insert(Velocity::new(player_orientation.0.mul_vec3(vec3(
            0.,
            BULLET_SPEED,
            0.,
        ))));
}

fn spawn_flame(
    commands: &mut Commands,
    textures: &Textures,
    material_assets: &mut Assets<ColorMaterial>,
    player_transform: &Transform,
) -> Entity {
    let texture = textures.flame.clone();
    let flame_width = textures.get_size(&texture).unwrap().x;
    let scale = FLAME_WIDTH / flame_width;
    let flame = commands
        .spawn_bundle(SpriteBundle {
            // TODO: ColorMaterial should not be created each time we show the flame
            material: material_assets.add(ColorMaterial::modulated_texture(
                texture,
                *Color::WHITE.clone().set_a(FLAME_OPACITY),
            )),
            // TODO: Transform should not be created each time we show the flame
            transform: Transform {
                translation: Vec2::new(0., FLAME_RELATIVE_Y).extend(FLAME_RELATIVE_Z)
                    / player_transform.scale,
                rotation: Default::default(),
                scale: Vec2::splat(scale / player_transform.scale.x).extend(1.),
            },
            ..Default::default()
        })
        .insert(Flame)
        .id();
    flame
}

fn log_player_stats(
        player_query: Query<&Velocity, With<Player>>,
) {
    for velocity in player_query.iter() {
        log::trace!("speed: {}", velocity.length());
    }
}
