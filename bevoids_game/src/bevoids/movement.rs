use bevy::{ecs::system::EntityCommands, log, prelude::*};
use derive_more::{Add, Deref, DerefMut, From, Into, Sub};
use enum_iterator::IntoEnumIterator;
use parry2d::bounding_volume::BoundingVolume;

use crate::bounds::GfxBounds;

#[derive(Debug, Copy, Clone, Deref, DerefMut, Add, Sub, From, Into, Component)]
pub struct Velocity(pub Vec2);

#[derive(Debug, Component)]
pub struct ShadowController;

#[derive(Debug, Component)]
pub struct NonWrapping;

#[allow(non_camel_case_types)]
#[derive(Debug, IntoEnumIterator, PartialEq, Copy, Clone, Component)]
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

#[derive(Debug, From, Into, Component)]
pub struct ShadowOf {
    pub controller: Entity,
    placement: ShadowPlacement,
}

#[derive(Debug, Component)]
pub struct InsideWindow;

pub struct EnterWindowEvent(Entity);

pub struct ExitWindowEvent(Entity);

pub fn spawn_display_shadows(
    controller: Entity,
    controller_size: Vec2,
    controller_image: Handle<Image>,
    component_inserter: &Option<impl Fn(EntityCommands)>,
    window_bounds: &GfxBounds,
    commands: &mut Commands,
) {
    for placement in ShadowPlacement::into_enum_iter() {
        let shadow_id = commands
            .spawn_bundle(SpriteBundle {
                texture: controller_image.clone(),
                transform: Transform {
                    translation: window_bounds.size().extend(0.),
                    ..Transform::default()
                },
                sprite: Sprite {
                    custom_size: Some(controller_size),
                    ..Default::default()
                },
                ..SpriteBundle::default()
            })
            .insert(GfxBounds::from_pos_and_size(
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

pub fn wrapping_linear_movement_system(
    mut query: Query<
        (&mut Transform, &mut GfxBounds, &Velocity),
        (Without<ShadowOf>, Without<NonWrapping>),
    >,
    window_bounds: Res<GfxBounds>,
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

pub fn non_wrapping_linear_movement_system(
    mut query: Query<
        (Entity, &mut Transform, &mut GfxBounds, &Velocity),
        (Without<ShadowOf>, With<NonWrapping>),
    >,
    mut enter_window_event: EventWriter<EnterWindowEvent>,
    mut exit_window_event: EventWriter<ExitWindowEvent>,
    window_bounds: Res<GfxBounds>,
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
            (true, false) => exit_window_event.send(ExitWindowEvent(entity)),
            (false, true) => enter_window_event.send(EnterWindowEvent(entity)),
            (false, false) => {}
        }
    }
}

pub fn move_shadow_system(
    mut commands: Commands,
    mut shadows: Query<(Entity, &mut Transform, &mut GfxBounds, &ShadowOf)>,
    controllers: Query<(Entity, &Transform), (With<ShadowController>, Without<ShadowOf>)>,
    window_bounds: Res<GfxBounds>,
) {
    let (w, h) = {
        let b = window_bounds.size();
        (b.x, b.y)
    };
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
            if shadow_bounds.as_aabb().intersects(window_bounds.as_aabb()) {
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
