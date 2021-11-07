use bevy::{ecs::system::EntityCommands, log, math::vec3, prelude::*};
use bevy_kira_audio::{Audio, AudioChannel};
use rand::Rng;
use std::f32::consts::PI;

use crate::{
    constants::*,
    plugins::{
        spawn_display_shadows, Despawn, FireLaserEvent, InsideWindow, ShadowController, Velocity,
    },
    resources::{AudioAssets, Bounds, TextureAssets},
    Animations, AudioChannels, GameState, GeneralTexture, Sounds, StartSingleAnimation,
};

pub struct PlayerPlugin;

#[derive(Debug, Clone, Copy)]
pub struct PlayerDeadEvent;

#[derive(Component, Debug, Reflect)]
pub struct Player;

#[derive(Component, Debug, Reflect)]
struct Flame;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerDeadEvent>();

        app.register_type::<Player>().register_type::<Flame>();

        app.add_system_set(
            SystemSet::on_enter(GameState::InGame).with_system(player_spawn.system()),
        );

        app.add_system_set(
            SystemSet::on_update(GameState::InGame)
                .with_system(player_dead_gameover.system())
                .with_system(player_dead_sound.system())
                .with_system(player_controls.system()),
        );

        app.add_system_set(SystemSet::on_exit(GameState::InGame).with_system(exit_ingame.system()));
    }
}

fn player_dead_gameover(
    mut events: EventReader<PlayerDeadEvent>,
    mut anim_events: EventWriter<StartSingleAnimation>,
    player_query: Query<(Entity, &Transform, &Bounds), With<Player>>,
    mut commands: Commands,
    mut state: ResMut<State<GameState>>,
) {
    for _ in events.iter() {
        log::warn!("player dead");
        player_query.iter().for_each(|x| {
            anim_events.send(StartSingleAnimation {
                key: Animations::BigExplosion,
                position: x.1.translation,
                size: x.2.size().max_element(),
            });
            commands
                .entity(x.0)
                .remove::<Player>()
                .remove::<Velocity>()
                .insert(Despawn);
        });

        state.set(GameState::GameOver).unwrap();
    }
}

fn player_dead_sound(
    mut events: EventReader<PlayerDeadEvent>,
    channels: Res<AudioChannels>,
    audio_assets: Res<AudioAssets<Sounds>>,
    audio: Res<Audio>,
) {
    for _ in events.iter() {
        audio.play_in_channel(
            audio_assets
                .get(Sounds::ShipExplode)
                .expect("missing laser sound"),
            &channels.ship_explode,
        );
    }
}

fn player_spawn(
    mut commands: Commands,
    mut color_assets: ResMut<Assets<ColorMaterial>>,
    window_bounds: Res<Bounds>,
    textures: Res<TextureAssets<GeneralTexture>>,
) {
    let mut rng = rand::thread_rng();

    let player_position_vec2 = Vec2::new(
        rng.gen_range(-window_bounds.width() / 2.0..window_bounds.width() / 2.0),
        rng.gen_range(-window_bounds.height() / 2.0..window_bounds.height() / 2.0),
    );
    let player_position_vec3 = player_position_vec2.extend(PLAYER_Z);

    let spaceship_texture = textures
        .get(GeneralTexture::Spaceship)
        .expect("no texture for spaceship");
    let texture_size = spaceship_texture.size;
    let player_material = color_assets.add(spaceship_texture.texture.clone().into());
    let player_scale = PLAYER_MAX_SIZE / texture_size.max_element() as f32;
    let player_size = Vec2::new(texture_size.x as f32, texture_size.y as f32) * player_scale;
    let random_rotation = Quat::from_rotation_z(rng.gen_range(0.0..(2. * PI)));
    let player_velocity = random_rotation.mul_vec3(Vec3::Y).truncate() * PLAYER_START_SPEED;

    let player_id: Entity = commands
        .spawn_bundle(SpriteBundle {
            material: player_material.clone(),
            transform: Transform {
                translation: player_position_vec3,
                rotation: random_rotation,
                scale: Vec2::splat(player_scale).extend(1.),
            },
            ..SpriteBundle::default()
        })
        .insert(Player)
        .insert(Bounds::from_pos_and_size(player_position_vec2, player_size))
        .insert(Velocity::from(player_velocity))
        .insert(ShadowController)
        .insert(InsideWindow)
        .id();

    spawn_display_shadows(
        player_id,
        player_size,
        player_scale,
        player_material,
        &Some(|mut cmds: EntityCommands| {
            cmds.insert(Player);
        }),
        &window_bounds,
        &mut commands,
    );

    log::info!(player=?player_id, "player spawned");
}

fn exit_ingame(audio: Res<Audio>) {
    audio.stop_channel(&AudioChannel::new(Sounds::Thruster.to_string()));
}

fn player_controls(
    commands: Commands,
    kb: Res<Input<KeyCode>>,
    color_assets: ResMut<Assets<ColorMaterial>>,
    mut player_query: Query<(Entity, &mut Velocity, &mut Transform), With<Player>>,
    flame_query: Query<Entity, With<Flame>>,
    textures: Res<TextureAssets<GeneralTexture>>,
    time: Res<Time>,
    audio: Res<Audio>,
    channels: Res<AudioChannels>,
    audio_assets: Res<AudioAssets<Sounds>>,
    fire_laser_events: EventWriter<FireLaserEvent>,
) {
    let (player, mut player_velocity, mut player_transform) =
        player_query.get_single_mut().expect("no player to control");

    fire_laser(&kb, fire_laser_events);
    turn_player(&kb, &time, &mut player_transform);
    accelleration(
        &kb,
        &time,
        &mut player_velocity,
        &audio,
        &channels,
        color_assets,
        &audio_assets,
        commands,
        &textures,
        player_transform,
        player,
        flame_query,
    );
}

fn accelleration(
    kb: &Input<KeyCode>,
    time: &Time,
    player_velocity: &mut Velocity,
    audio: &Audio,
    channels: &AudioChannels,
    color_assets: ResMut<Assets<ColorMaterial>>,
    audio_assets: &AudioAssets<Sounds>,
    mut commands: Commands,
    textures: &TextureAssets<GeneralTexture>,
    player_transform: Mut<Transform>,
    player: Entity,
    flame_query: Query<Entity, With<Flame>>,
) {
    if kb.pressed(KeyCode::Up) {
        // accelleration
        let delta_v = player_transform
            .rotation
            .mul_vec3(vec3(0., PLAYER_ACCELLERATION, 0.))
            .truncate()
            * time.delta_seconds();
        let velocity = (Vec2::from(*player_velocity) + delta_v).clamp_length(0., PLAYER_MAX_SPEED);
        **player_velocity = velocity.into();
        if kb.just_pressed(KeyCode::Up) {
            log::trace!("accellerate on");
            audio.play_looped_in_channel(
                audio_assets
                    .get(Sounds::Thruster)
                    .expect("missing laser sound"),
                &channels.thruster,
            );
            let flame = spawn_flame(&mut commands, color_assets, textures, &player_transform);
            commands.entity(player).push_children(&[flame]);
        }
    } else {
        // decellerate
        let delta_v =
            Vec2::from(*player_velocity).normalize() * PLAYER_DECCELLERATION * time.delta_seconds();
        let velocity = (Vec2::from(*player_velocity) - delta_v).clamp_length(0., PLAYER_MAX_SPEED);
        **player_velocity = velocity.into();
        if kb.just_released(KeyCode::Up) {
            log::trace!("accellerate off");
            audio.stop_channel(&AudioChannel::new(Sounds::Thruster.to_string()));
            for flame in flame_query.iter() {
                commands.entity(flame).despawn();
            }
        }
    }
}

fn fire_laser(kb: &Input<KeyCode>, mut fire_laser_events: EventWriter<FireLaserEvent>) {
    if kb.just_pressed(KeyCode::Space) {
        log::debug!("fire!");
        fire_laser_events.send(FireLaserEvent);
    }
}

fn turn_player(kb: &Input<KeyCode>, time: &Time, player_transform: &mut Transform) {
    if kb.pressed(KeyCode::Left) {
        player_transform.rotation = player_transform.rotation.mul_quat(Quat::from_rotation_z(
            PLAYER_TURN_SPEED * time.delta_seconds(),
        ));
    } else if kb.pressed(KeyCode::Right) {
        player_transform.rotation = player_transform.rotation.mul_quat(Quat::from_rotation_z(
            -PLAYER_TURN_SPEED * time.delta_seconds(),
        ));
    }
}

fn spawn_flame(
    commands: &mut Commands,
    mut color_assets: ResMut<Assets<ColorMaterial>>,
    textures: &TextureAssets<GeneralTexture>,
    player_transform: &Transform,
) -> Entity {
    let texture = textures
        .get(GeneralTexture::Flame)
        .expect("no flame texture");
    let flame_width = texture.size.x as f32;
    let scale = FLAME_WIDTH / flame_width;
    let flame = commands
        .spawn_bundle(SpriteBundle {
            material: color_assets.add(texture.texture.clone().into()),
            transform: Transform {
                translation: Vec3::new(0., FLAME_RELATIVE_Y, FLAME_RELATIVE_Z)
                    / player_transform.scale,
                rotation: Quat::default(),
                scale: Vec2::splat(scale / player_transform.scale.x).extend(1.),
            },
            ..SpriteBundle::default()
        })
        .insert(Flame)
        .id();
    flame
}
