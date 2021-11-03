use bevy::{ecs::system::EntityCommands, log, prelude::*};
use derive_more::{Add, AddAssign, Deref, DerefMut, From, Into, Sub, SubAssign};
use enum_iterator::IntoEnumIterator;
use parry2d::bounding_volume::BoundingVolume;

use crate::Bounds;

pub struct MovementPlugin;

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
pub struct Velocity(pub Vec2);

#[derive(Component, Debug)]
pub struct ShadowController;

#[derive(Component, Debug)]
pub struct NonWrapping;

#[allow(non_camel_case_types)]
#[derive(Debug, IntoEnumIterator, PartialEq, Copy, Clone)]
enum ShadowPlacement {
    MinW_MaxH,
    MedW_MaxH,
    MaxW_MaxH,
    MinW_MedY,
    MaxW_MedY,
    MinW_MinH,
    MedW_MinH,
    MaxW_MinH,
}

#[derive(Component, Debug, From, Into)]
pub struct ShadowOf {
    pub controller: Entity,
    placement: ShadowPlacement,
}

#[derive(Component, Debug)]
pub struct InsideWindow;

pub struct EnterWindow(Entity);
pub struct ExitWindow(Entity);

pub fn spawn_display_shadows(
    controller: Entity,
    controller_size: Vec2,
    controller_scale: f32,
    controller_material: Handle<ColorMaterial>,
    component_inserter: &Option<impl Fn(EntityCommands)>,
    window_bounds: &Bounds,
    commands: &mut Commands,
) {
    for placement in ShadowPlacement::into_enum_iter() {
        let shadow_id = commands
            .spawn_bundle(SpriteBundle {
                material: controller_material.clone(),
                transform: Transform {
                    translation: window_bounds.size().extend(0.),
                    scale: Vec2::splat(controller_scale).extend(1.),
                    ..Transform::default()
                },
                ..SpriteBundle::default()
            })
            .insert(Bounds::from_pos_and_size(
                window_bounds.size(),
                controller_size,
            ))
            .insert(ShadowOf {
                controller,
                placement,
            })
            .id();
        if let Some(component_inserter) = component_inserter {
            component_inserter(commands.entity(shadow_id));
        }

        log::trace!(shadow=?shadow_id, ctrl=?controller, "shadow spawned");
    }
}

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EnterWindow>();
        app.add_event::<ExitWindow>();

        app.add_system(wrapping_linear_movement.system().before("shadow_movement"))
            .add_system(
                non_wrapping_linear_movement
                    .system()
                    .before("shadow_movement"),
            )
            .add_system(move_shadow.system().label("shadow_movement"));
    }
}

fn wrapping_linear_movement(
    mut query: Query<
        (&mut Transform, &mut Bounds, &Velocity),
        (Without<ShadowOf>, Without<NonWrapping>),
    >,
    window_bounds: Res<Bounds>,
    time: Res<Time>,
) {
    let window_half_bounds = window_bounds.as_aabb().half_extents();

    for (mut transform, mut bounds, velocity) in query.iter_mut() {
        let pos = &mut transform.translation;

        *pos += (Vec2::from(*velocity) * time.delta_seconds()).extend(0.);

        // keep inside window bounds
        if pos.x > window_half_bounds.x {
            pos.x -= window_half_bounds.x * 2.;
        } else if pos.x < -window_half_bounds.x {
            pos.x += window_half_bounds.x * 2.;
        }
        if pos.y > window_half_bounds.y {
            pos.y -= window_half_bounds.y * 2.;
        } else if pos.y < -window_half_bounds.y {
            pos.y += window_half_bounds.y * 2.;
        }

        bounds.set_center(pos.truncate());
    }
}

fn non_wrapping_linear_movement(
    mut query: Query<
        (Entity, &mut Transform, &mut Bounds, &Velocity),
        (Without<ShadowOf>, With<NonWrapping>),
    >,
    mut enter_window: EventWriter<EnterWindow>,
    mut exit_window: EventWriter<ExitWindow>,
    window_bounds: Res<Bounds>,
    time: Res<Time>,
) {
    let (ww, wh) = {
        let b = window_bounds.as_aabb().half_extents();
        (b.x, b.y)
    };
    for (entity, mut transform, mut bounds, velocity) in query.iter_mut() {
        let pos = &mut transform.translation;
        let was_in_window = pos.x >= -ww && pos.x <= ww && pos.y >= -wh && pos.y <= wh;

        *pos += (Vec2::from(*velocity) * time.delta_seconds()).extend(0.);
        bounds.set_center(pos.truncate());

        match (
            was_in_window,
            pos.x >= -ww && pos.x <= ww && pos.y >= -wh && pos.y <= wh,
        ) {
            (true, true) => {}
            (true, false) => exit_window.send(ExitWindow(entity)),
            (false, true) => enter_window.send(EnterWindow(entity)),
            (false, false) => {}
        }
    }
}

fn move_shadow(
    mut commands: Commands,
    mut shadows: Query<(Entity, &mut Transform, &mut Bounds, &ShadowOf)>,
    controllers: Query<(Entity, &Transform), (With<ShadowController>, Without<ShadowOf>)>,
    window_bounds: Res<Bounds>,
) {
    let [w, h] = window_bounds.size().to_array();
    for (shadow, mut shadow_bounds, mut shadow_tf, placement, controller_tf) in shadows
        .iter_mut()
        .map(|(entity, transform, bounds, shadowof)| {
            (
                entity,
                bounds,
                transform,
                shadowof.placement,
                controllers
                    .iter()
                    .find(|(controller, _)| controller == &shadowof.controller)
                    .map(|(_, t)| t),
            )
        })
    {
        if let Some(controller_tf) = controller_tf {
            // follow controller
            shadow_tf.translation = controller_tf.translation
                + Vec2::from(match placement {
                    ShadowPlacement::MinW_MaxH => (-w, h),
                    ShadowPlacement::MedW_MaxH => (0., h),
                    ShadowPlacement::MaxW_MaxH => (w, h),
                    ShadowPlacement::MinW_MedY => (-w, 0.),
                    ShadowPlacement::MaxW_MedY => (w, 0.),
                    ShadowPlacement::MinW_MinH => (-w, -h),
                    ShadowPlacement::MedW_MinH => (0., -h),
                    ShadowPlacement::MaxW_MinH => (w, -h),
                })
                .extend(0.);
            shadow_tf.rotation = controller_tf.rotation;

            // set new bounds
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
