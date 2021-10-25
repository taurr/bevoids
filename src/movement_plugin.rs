use crate::{GameState, SpriteSize, WinSize};
use bevy::{ecs::system::EntityCommands, log, math::vec3, prelude::*};
use std::ops::{Deref, DerefMut};

pub struct MovementPlugin;

#[derive(Debug, Default, Component, Copy, Clone)]
pub struct Velocity(pub Vec3);

impl Velocity {
    pub fn new(movement_pr_sec: Vec3) -> Self {
        Self(movement_pr_sec)
    }
}

impl Deref for Velocity {
    type Target = Vec3;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Velocity {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameState::InGame)
                .with_system(linear_movement.system())
                .with_system(move_shadow.system())
                .with_system(keep_in_bounds.system()),
        );
    }
}

#[derive(Debug, Default, Component)]
pub struct ShadowController;

#[derive(Debug, Component)]
pub struct ShadowOf(pub Entity, pub Vec3);

#[allow(dead_code)]
pub fn overlaps(pos1: Vec3, size1: Vec2, pos2: Vec3, size2: Vec2) -> bool {
    fn overlaps_segment(p1: f32, len1: f32, p2: f32, len2: f32) -> bool {
        let p1_left = p1 - len1 / 2.0;
        let p1_right = p1 + len1 / 2.0;
        let p2_left = p2 - len2 / 2.0;
        let p2_right = p2 + len2 / 2.0;

        (p1_left >= p2_left && p1_left <= p2_right)
            || (p1_right >= p2_left && p1_right <= p2_right)
            || (p2_left >= p1_left && p2_left <= p1_right)
            || (p2_right >= p1_left && p2_right <= p1_right)
    }

    let r1 = overlaps_segment(pos1.x, size1.x, pos2.x, size2.x);
    let r2 = overlaps_segment(pos1.y, size1.y, pos2.y, size2.y);
    r1 && r2
}

fn linear_movement(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in query.iter_mut() {
        let d = velocity.0 * time.delta_seconds();
        transform.translation = vec3(
            transform.translation.x + d.x,
            transform.translation.y + d.y,
            transform.translation.z,
        );
    }
}

fn move_shadow(
    mut commands: Commands,
    mut shadows: Query<(Entity, &mut Transform, &ShadowOf)>,
    controllers: Query<(Entity, &Transform), (With<ShadowController>, Without<ShadowOf>)>,
) {
    for (shadow, mut shadow_tf, displacement, controller_tf) in
        shadows.iter_mut().map(|(entity, transform, shadowof)| {
            (
                entity,
                transform,
                shadowof.1,
                controllers
                    .iter()
                    .find(|(e, _)| e == &shadowof.0)
                    .map(|(_, t)| t),
            )
        })
    {
        if let Some(controller_tf) = controller_tf {
            shadow_tf.translation = controller_tf.translation + displacement;
            shadow_tf.rotation = controller_tf.rotation;
        } else {
            log::debug!("despawning child w.o. controller");
            commands.entity(shadow).despawn_recursive();
        }
    }
}

fn keep_in_bounds(mut query: Query<&mut Transform, With<Velocity>>, win_size: Res<WinSize>) {
    for mut transform in query.iter_mut() {
        let pos = &mut transform.translation;
        let [w, h] = (win_size.0 / 2.).to_array();
        if pos.x > w {
            pos.x -= w * 2.;
        } else if pos.x < -w {
            pos.x += w * 2.;
        }
        if pos.y > h {
            pos.y -= h * 2.;
        } else if pos.y < -h {
            pos.y += h * 2.;
        }
    }
}

pub fn spawn_shadows_for_display_wrap(
    id: Entity,
    material: Handle<ColorMaterial>,
    sprite_size: SpriteSize,
    win_size: &WinSize,
    controller_scale: f32,
    controller_translation: Vec3,
    component_inserter: &Option<impl Fn(EntityCommands)>,
    commands: &mut Commands,
) {
    for x in [-win_size.0.x, win_size.0.x] {
        for y in [-win_size.0.y, win_size.0.y] {
            spawn_shadow(
                id,
                sprite_size,
                controller_scale,
                controller_translation,
                Vec3::new(x, y, 0.),
                &material,
                component_inserter,
                commands,
            );
        }
    }
    for x in [-win_size.0.x, win_size.0.x] {
        spawn_shadow(
            id,
            sprite_size,
            controller_scale,
            controller_translation,
            Vec3::new(x, 0., 0.),
            &material,
            component_inserter,
            commands,
        );
    }
    for y in [-win_size.0.y, win_size.0.y] {
        spawn_shadow(
            id,
            sprite_size,
            controller_scale,
            controller_translation,
            Vec3::new(0., y, 0.),
            &material,
            component_inserter,
            commands,
        );
    }
}

fn spawn_shadow(
    shadow_controller: Entity,
    sprite_size: SpriteSize,
    controller_scale: f32,
    controller_translation: Vec3,
    displacement: Vec3,
    material: &Handle<ColorMaterial>,
    component_inserter: &Option<impl Fn(EntityCommands)>,
    commands: &mut Commands,
) {
    let id = commands
        .spawn_bundle(SpriteBundle {
            material: material.clone(),
            transform: Transform {
                translation: controller_translation + displacement,
                scale: Vec2::splat(controller_scale).extend(1.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(sprite_size)
        .insert(ShadowOf(shadow_controller, displacement))
        .id();
    if let Some(component_inserter) = component_inserter {
        component_inserter(commands.entity(id));
    }
}
