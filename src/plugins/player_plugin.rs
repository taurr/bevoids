use bevy::{ecs::system::EntityCommands, log, math::vec3, prelude::*};
use bevy_kira_audio::{Audio, AudioChannel};
use derive_more::{Constructor, Deref, DerefMut, From, Into};
use rand::Rng;
use std::f32::consts::PI;

use crate::{
    assets::LoadRelative,
    constants::*,
    plugins::{
        spawn_display_shadows, FadeDespawn, FireBulletEvent, InsideWindow, ShadowController,
        Textures, Velocity,
    },
    Args, Bounds, GameState,
};

pub struct PlayerPlugin;

#[derive(Debug, Clone, Copy)]
pub struct PlayerDeadEvent;

#[derive(Component, Debug)]
pub struct Player;

#[derive(Component, Debug, Default, From, Into, Copy, Clone, Deref, DerefMut, Constructor)]
pub struct Orientation(pub Quat);

#[derive(Component, Debug)]
struct Flame;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerDeadEvent>();

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
    player_query: Query<Entity, With<Player>>,
    mut commands: Commands,
    mut state: ResMut<State<GameState>>,
) {
    for _ in events.iter() {
        log::warn!("player dead");
        player_query.iter().for_each(|player| {
            commands
                .entity(player)
                .remove::<Player>()
                .remove::<Velocity>()
                .insert(FadeDespawn::from_secs_f32(PLAYER_FADEOUT_SECONDS));
        });

        state.set(GameState::GameOver).unwrap();
    }
}

fn player_dead_sound(
    mut events: EventReader<PlayerDeadEvent>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    args: Res<Args>,
) {
    for _ in events.iter() {
        let audio_channel = AudioChannel::new(AUDIO_CHANNEL_EXPLOSION_SHIP.into());
        audio.set_volume_in_channel(AUDIO_EXPLOSION_SHIP_VOLUME, &audio_channel);
        audio.play_in_channel(
            asset_server
                .load_relative(&AUDIO_EXPLOSION_SHIP, &*args)
                .expect("missing laser sound"),
            &audio_channel,
        );
    }
}

fn player_spawn(
    mut commands: Commands,
    window_bounds: Res<Bounds>,
    textures: Res<Textures>,
    mut material_assets: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = rand::thread_rng();

    let player_position_vec2 = Vec2::new(
        rng.gen_range(-window_bounds.width() / 2.0..window_bounds.width() / 2.0),
        rng.gen_range(-window_bounds.height() / 2.0..window_bounds.height() / 2.0),
    );
    let player_position_vec3 = player_position_vec2.extend(PLAYER_Z);

    let texture_size = textures.get_size(&textures.spaceship).unwrap();
    let player_material = material_assets.add(textures.spaceship.clone().into());
    let player_scale = PLAYER_MAX_SIZE / texture_size.max_element();
    let player_size = texture_size * player_scale;
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
        .insert(Orientation(random_rotation))
        .insert(ShadowController)
        .insert(InsideWindow)
        .id();

    spawn_display_shadows(
        player_id,
        player_position_vec3,
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
    audio.stop_channel(&AudioChannel::new(AUDIO_CHANNEL_THRUSTER.into()));
}

fn player_controls(
    commands: Commands,
    kb: Res<Input<KeyCode>>,
    mut player_query: Query<
        (Entity, &mut Velocity, &mut Orientation, &mut Transform),
        With<Player>,
    >,
    flame_query: Query<Entity, With<Flame>>,
    textures: Res<Textures>,
    mut material_assets: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
    args: Res<Args>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    fire_bullet_events: EventWriter<FireBulletEvent>,
) {
    let (player, mut player_velocity, mut player_orientation, mut player_transform) =
        player_query.get_single_mut().expect("no player to control");

    fire_laser(&kb, fire_bullet_events);
    turn_player(&kb, &mut player_orientation, &time, &mut player_transform);
    accelleration(
        &kb,
        &player_orientation,
        &time,
        &mut player_velocity,
        &audio,
        &asset_server,
        &args,
        commands,
        &textures,
        &mut material_assets,
        player_transform,
        player,
        flame_query,
    );
}

fn accelleration(
    kb: &Input<KeyCode>,
    player_orientation: &Orientation,
    time: &Time,
    player_velocity: &mut Velocity,
    audio: &Audio,
    asset_server: &AssetServer,
    args: &Args,
    mut commands: Commands,
    textures: &Textures,
    mut material_assets: &mut Assets<ColorMaterial>,
    player_transform: Mut<Transform>,
    player: Entity,
    flame_query: Query<Entity, With<Flame>>,
) {
    if kb.pressed(KeyCode::Up) {
        // accelleration
        let delta_v = player_orientation
            .0
            .mul_vec3(vec3(0., PLAYER_ACCELLERATION, 0.))
            .truncate()
            * time.delta_seconds();
        let velocity = (Vec2::from(*player_velocity) + delta_v).clamp_length(0., PLAYER_MAX_SPEED);
        **player_velocity = velocity.into();
        if kb.just_pressed(KeyCode::Up) {
            log::trace!("accellerate on");
            let audio_channel = AudioChannel::new(AUDIO_CHANNEL_THRUSTER.into());
            audio.set_volume_in_channel(AUDIO_THRUSTER_VOLUME, &audio_channel);
            audio.play_looped_in_channel(
                asset_server
                    .load_relative(&AUDIO_THRUSTER, &*args)
                    .expect("missing laser sound"),
                &audio_channel,
            );
            let flame = spawn_flame(
                &mut commands,
                textures,
                &mut material_assets,
                &player_transform,
            );
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
            audio.stop_channel(&AudioChannel::new(AUDIO_CHANNEL_THRUSTER.into()));
            for flame in flame_query.iter() {
                commands.entity(flame).despawn();
            }
        }
    }
}

fn fire_laser(kb: &Input<KeyCode>, mut fire_bullet_events: EventWriter<FireBulletEvent>) {
    if kb.just_pressed(KeyCode::Space) {
        log::debug!("fire!");
        fire_bullet_events.send(FireBulletEvent);
    }
}

fn turn_player(
    kb: &Input<KeyCode>,
    player_orientation: &mut Orientation,
    time: &Time,
    player_transform: &mut Transform,
) {
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
}

fn spawn_flame(
    commands: &mut Commands,
    textures: &Textures,
    material_assets: &mut Assets<ColorMaterial>,
    player_transform: &Transform,
) -> Entity {
    let texture = textures.flame.clone();
    let flame_width = textures
        .get_size(&texture)
        .expect("no size for flame texture")
        .x;
    let scale = FLAME_WIDTH / flame_width;
    let flame = commands
        .spawn_bundle(SpriteBundle {
            material: material_assets.add(ColorMaterial::modulated_texture(
                texture,
                Color::WHITE.clone(),
            )),
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
