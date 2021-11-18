use bevy::{ecs::system::EntityCommands, log, math::vec3, prelude::*};
use bevy_asset_map::{GfxBounds, TextureAssetMap};
use bevy_effects::{
    animation::AnimationEffectEvent,
    despawn::Despawn,
    sound::{LoopSfx, PlaySfx, SetPanSfx, SfxCmdEvent, StopSfx},
};
use rand::Rng;
use std::f32::consts::PI;

use super::{
    laser::FireLaserEvent,
    movement::{spawn_display_shadows, InsideWindow, ShadowController, Velocity},
    settings::Settings,
    AnimationAtlas, BackgroundTexture, GameState, GeneralTexture, SoundEffect,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct PlayerDeadEvent;

#[derive(Component, Debug)]
pub(crate) struct Player;

#[derive(Component, Debug)]
pub(crate) struct Flame;

pub(crate) fn handle_player_dead(
    mut events: EventReader<PlayerDeadEvent>,
    mut sfx_event: EventWriter<SfxCmdEvent<SoundEffect>>,
    mut anim_effect_event: EventWriter<AnimationEffectEvent<AnimationAtlas>>,
    player_query: Query<(Entity, &Transform, &GfxBounds), (With<Player>, With<ShadowController>)>,
    mut commands: Commands,
    mut state: ResMut<State<GameState>>,
    win_bounds: Res<GfxBounds>,
    settings: Res<Settings>,
) {
    for _ in events.iter() {
        for (player, transform, bounds) in player_query.iter() {
            let panning = (transform.translation.x + win_bounds.width() / 2.) / win_bounds.width();
            sfx_event.send(
                PlaySfx::new(SoundEffect::ShipExplode)
                    .with_panning(panning)
                    .into(),
            );

            anim_effect_event.send(AnimationEffectEvent {
                key: AnimationAtlas::BigExplosion,
                position: transform.translation,
                size: bounds.size().max_element(),
                fps: settings.general.animation_fps,
            });
            log::warn!(?player, "player dead");
            commands
                .entity(player)
                .remove::<Player>()
                .remove::<Velocity>()
                .insert(Despawn);
        }

        state.set(GameState::GameOver).unwrap();
    }
}

pub(crate) fn spawn_player(
    mut commands: Commands,
    mut color_assets: ResMut<Assets<ColorMaterial>>,
    window_bounds: Res<GfxBounds>,
    texture_asset_map: Res<TextureAssetMap<GeneralTexture>>,
    background_asset_map: Res<TextureAssetMap<BackgroundTexture>>,
    win_bounds: Res<GfxBounds>,
    settings: Res<Settings>,
) {
    let mut rng = rand::thread_rng();

    let player_position = Vec3::new(
        rng.gen_range(-window_bounds.width() / 2.0..window_bounds.width() / 2.0),
        rng.gen_range(-window_bounds.height() / 2.0..window_bounds.height() / 2.0),
        settings.player.zpos,
    );

    let spaceship_texture = texture_asset_map
        .get(GeneralTexture::Spaceship)
        .expect("no texture for spaceship");
    let texture_size = spaceship_texture.size;
    let player_material = color_assets.add(spaceship_texture.texture.clone().into());
    let player_scale = settings.player.size / texture_size.max_element() as f32;
    let player_size = Vec2::new(texture_size.x as f32, texture_size.y as f32) * player_scale;
    let random_rotation = Quat::from_rotation_z(rng.gen_range(0.0..(2. * PI)));
    let player_velocity = random_rotation.mul_vec3(Vec3::Y).truncate() * 1.;

    let player_id: Entity = commands
        .spawn_bundle(SpriteBundle {
            material: player_material.clone(),
            transform: Transform {
                translation: player_position,
                rotation: random_rotation,
                scale: Vec2::splat(player_scale).extend(1.),
            },
            ..SpriteBundle::default()
        })
        .insert(Player)
        .insert(GfxBounds::from_pos_and_size(
            player_position.truncate(),
            player_size,
        ))
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

    let bg_texture = background_asset_map
        .get(BackgroundTexture(
            rng.gen_range(0..background_asset_map.len()),
        ))
        .expect("no texture for background");
    let bg_material = color_assets.add(bg_texture.texture.clone().into());
    let bg_size = bg_texture.size;
    let bg_scale = f32::max(
        win_bounds.width() / bg_size.x as f32,
        win_bounds.height() / bg_size.y as f32,
    );
    commands.spawn_bundle(SpriteBundle {
        material: bg_material,
        transform: Transform {
            scale: Vec3::splat(bg_scale),
            ..Default::default()
        },
        ..SpriteBundle::default()
    });
}

pub(crate) fn stop_thruster_sounds(mut sfx_event: EventWriter<SfxCmdEvent<SoundEffect>>) {
    sfx_event.send(StopSfx::new(SoundEffect::Thruster).into());
}

pub(crate) fn player_controls(
    commands: Commands,
    kb: Res<Input<KeyCode>>,
    sfx_event: EventWriter<SfxCmdEvent<SoundEffect>>,
    fire_laser_event: EventWriter<FireLaserEvent>,
    mut player_query: Query<(Entity, &mut Velocity, &mut Transform), With<Player>>,
    flame_query: Query<Entity, With<Flame>>,
    mut color_assets: ResMut<Assets<ColorMaterial>>,
    texture_asset_map: Res<TextureAssetMap<GeneralTexture>>,
    time: Res<Time>,
    settings: Res<Settings>,
    bounds: Res<GfxBounds>,
) {
    let (player, mut player_velocity, mut player_transform) =
        player_query.get_single_mut().expect("no player to control");

    fire_laser(&kb, fire_laser_event);
    turn_player(&kb, &time, &mut player_transform, &settings);
    accelleration(
        &kb,
        player,
        &mut player_velocity,
        &player_transform,
        sfx_event,
        &time,
        commands,
        flame_query,
        &mut color_assets,
        &texture_asset_map,
        &bounds,
        &settings,
    );
}

fn accelleration(
    kb: &Input<KeyCode>,
    player: Entity,
    player_velocity: &mut Velocity,
    player_transform: &Transform,
    mut sfx_event: EventWriter<SfxCmdEvent<SoundEffect>>,
    time: &Time,
    mut commands: Commands,
    flame_query: Query<Entity, With<Flame>>,
    color_assets: &mut Assets<ColorMaterial>,
    texture_asset_map: &TextureAssetMap<GeneralTexture>,
    bounds: &GfxBounds,
    settings: &Settings,
) {
    if kb.pressed(KeyCode::Up) {
        // accelleration
        let delta_v = player_transform
            .rotation
            .mul_vec3(vec3(0., settings.player.accelleration, 0.))
            .truncate()
            * time.delta_seconds();
        let velocity =
            (Vec2::from(*player_velocity) + delta_v).clamp_length(0., settings.player.max_speed);
        **player_velocity = velocity.into();
        let panning = (player_transform.translation.x + bounds.width() / 2.) / bounds.width();
        if kb.just_pressed(KeyCode::Up) {
            log::trace!("accellerate on");
            sfx_event.send(
                LoopSfx::new(SoundEffect::Thruster)
                    .with_panning(panning)
                    .into(),
            );
            let flame = spawn_flame(
                &mut commands,
                color_assets,
                texture_asset_map,
                player_transform,
                settings,
            );
            commands.entity(player).push_children(&[flame]);
        } else {
            sfx_event.send(SetPanSfx::new(SoundEffect::Thruster, panning).into());
        }
    } else {
        // decellerate
        let delta_v = Vec2::from(*player_velocity).normalize()
            * settings.player.decelleration
            * time.delta_seconds();
        let velocity =
            (Vec2::from(*player_velocity) - delta_v).clamp_length(0., settings.player.max_speed);
        *player_velocity = velocity.into();
        if kb.just_released(KeyCode::Up) {
            log::trace!("accellerate off");
            sfx_event.send(StopSfx::new(SoundEffect::Thruster).into());
            for flame in flame_query.iter() {
                commands.entity(flame).despawn();
            }
        }
    }
}

fn fire_laser(kb: &Input<KeyCode>, mut fire_laser_events: EventWriter<FireLaserEvent>) {
    if kb.just_pressed(KeyCode::Space) {
        log::trace!("fire!");
        fire_laser_events.send(FireLaserEvent);
    }
}

fn turn_player(
    kb: &Input<KeyCode>,
    time: &Time,
    player_transform: &mut Transform,
    settings: &Settings,
) {
    let speed = if kb.pressed(KeyCode::RControl) {
        settings.player.turn_speed_fast
    } else {
        settings.player.turn_speed_slow
    };

    if kb.pressed(KeyCode::Left) {
        player_transform.rotation = player_transform
            .rotation
            .mul_quat(Quat::from_rotation_z(speed * time.delta_seconds()));
    } else if kb.pressed(KeyCode::Right) {
        player_transform.rotation = player_transform
            .rotation
            .mul_quat(Quat::from_rotation_z(-speed * time.delta_seconds()));
    }
}

fn spawn_flame(
    commands: &mut Commands,
    color_assets: &mut Assets<ColorMaterial>,
    textures: &TextureAssetMap<GeneralTexture>,
    player_transform: &Transform,
    settings: &Settings,
) -> Entity {
    let texture = textures
        .get(GeneralTexture::Flame)
        .expect("no flame texture");
    let flame_width = texture.size.x as f32;
    let scale = settings.player.flame_width / flame_width;
    let flame = commands
        .spawn_bundle(SpriteBundle {
            material: color_assets.add(texture.texture.clone().into()),
            transform: Transform {
                translation: Vec3::new(0., settings.player.flame_ypos, -1.0)
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