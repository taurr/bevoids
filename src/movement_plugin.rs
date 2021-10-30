use bevy::{ecs::system::EntityCommands, log, prelude::*};
use derive_more::{Add, AddAssign, Deref, DerefMut, From, Into, Sub, SubAssign};
use parry2d::bounding_volume::BoundingVolume;

use crate::{Bounds, GameState};

pub(crate) struct MovementPlugin;

#[derive(
    Component,
    Default,
    Debug,
    Copy,
    Clone,
    Deref,
    DerefMut,
    Add,
    Sub,
    AddAssign,
    SubAssign,
    From,
    Into,
)]
pub(crate) struct Velocity(Vec2);

#[derive(Component, Debug)]
pub(crate) struct ShadowController;

#[derive(Component, Debug, From, Into)]
pub(crate) struct ShadowOf {
    pub controller: Entity,
    pub displacement: Vec2,
}

#[derive(Component, Debug)]
pub(crate) struct InsideWindow;

pub(crate) fn spawn_display_shadows(
    controller: Entity,
    controller_position: Vec3,
    controller_size: Vec2,
    controller_scale: f32,
    controller_material: Handle<ColorMaterial>,
    component_inserter: &Option<impl Fn(EntityCommands)>,
    window_bounds: &Bounds,
    commands: &mut Commands,
) {
    for x in [-window_bounds.width(), window_bounds.width()] {
        for y in [-window_bounds.height(), window_bounds.height()] {
            let child = spawn_shadow(
                controller,
                controller_position,
                controller_size,
                controller_scale,
                &controller_material,
                Vec2::new(x, y),
                component_inserter,
                commands,
            );
            log::trace!(shadow=?child, ctrl=?controller, "shadow spawned");
        }
    }
    for x in [-window_bounds.width(), window_bounds.width()] {
        let child = spawn_shadow(
            controller,
            controller_position,
            controller_size,
            controller_scale,
            &controller_material,
            Vec2::new(x, 0.),
            component_inserter,
            commands,
        );
        log::trace!(shadow=?child, ctrl=?controller, "shadow spawned");
    }
    for y in [-window_bounds.height(), window_bounds.height()] {
        let child = spawn_shadow(
            controller,
            controller_position,
            controller_size,
            controller_scale,
            &controller_material,
            Vec2::new(0., y),
            component_inserter,
            commands,
        );
        log::trace!(shadow=?child, ctrl=?controller, "shadow spawned");
    }
}

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameState::InGame)
                .with_system(linear_movement.system().chain(move_shadow.system())),
        );
        app.add_system_set(
            SystemSet::on_update(GameState::GameOver)
                .with_system(linear_movement.system().chain(move_shadow.system())),
        );
    }
}

fn linear_movement(
    mut query: Query<(&mut Transform, &mut Bounds, &Velocity), Without<ShadowOf>>,
    window_bounds: Res<Bounds>,
    time: Res<Time>,
) {
    for (mut transform, mut bounds, velocity) in query.iter_mut() {
        let pos = &mut transform.translation;
        // move
        *pos += (Vec2::from(*velocity) * time.delta_seconds()).extend(0.);

        // keep inside window bounds
        let (w, h) = {
            let hext = window_bounds.as_aabb().half_extents();
            (hext.x, hext.y)
        };
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

        // set new bounds
        // TODO: should probably follow rotation as that changes bounds
        bounds.set_center(transform.translation.truncate());
    }
}

fn move_shadow(
    mut commands: Commands,
    mut shadows: Query<(Entity, &mut Transform, &mut Bounds, &ShadowOf)>,
    window_bounds: Res<Bounds>,
    controllers: Query<(Entity, &Transform), (With<ShadowController>, Without<ShadowOf>)>,
) {
    for (shadow, mut shadow_bounds, mut shadow_tf, displacement, controller_tf) in shadows
        .iter_mut()
        .map(|(entity, transform, bounds, shadowof)| {
            (
                entity,
                bounds,
                transform,
                shadowof.displacement,
                controllers
                    .iter()
                    .find(|(controller, _)| controller == &shadowof.controller)
                    .map(|(_, t)| t),
            )
        })
    {
        if let Some(controller_tf) = controller_tf {
            // follow controller
            shadow_tf.translation = controller_tf.translation + displacement.extend(0.);
            shadow_tf.rotation = controller_tf.rotation;

            // set new bounds
            // TODO: should probably follow rotation as that changes bounds
            shadow_bounds.set_center(shadow_tf.translation.truncate());

            // detect visibility
            if shadow_bounds.as_aabb().intersects(&window_bounds.as_aabb()) {
                commands.entity(shadow).insert(InsideWindow);
            } else {
                commands.entity(shadow).remove::<InsideWindow>();
            }
        } else {
            log::trace!(?shadow, "despawning orphan");
            commands.entity(shadow).despawn_recursive();
        }
    }
}

fn spawn_shadow(
    controller: Entity,
    controller_position: Vec3,
    controller_size: Vec2,
    controller_scale: f32,
    controller_material: &Handle<ColorMaterial>,
    displacement: Vec2,
    component_inserter: &Option<impl Fn(EntityCommands)>,
    commands: &mut Commands,
) -> Entity {
    let position = controller_position + displacement.extend(0.);
    let shadow_id = commands
        .spawn_bundle(SpriteBundle {
            material: controller_material.clone(),
            transform: Transform {
                translation: position,
                scale: Vec2::splat(controller_scale).extend(1.),
                ..Transform::default()
            },
            ..SpriteBundle::default()
        })
        .insert(Bounds::from_pos_and_size(
            position.truncate(),
            controller_size,
        ))
        .insert(ShadowOf {
            controller,
            displacement,
        })
        .id();
    if let Some(component_inserter) = component_inserter {
        component_inserter(commands.entity(shadow_id));
    }
    shadow_id
}
