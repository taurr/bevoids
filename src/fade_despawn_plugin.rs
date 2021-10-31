use bevy::{log, prelude::*};
use std::time::Duration;

pub struct FadeDespawnPlugin;

#[derive(Component, Debug)]
pub struct Despawn;

#[derive(Component, Debug)]
pub struct FadeDespawn {
    speed: f32,
    value: f32,
}

#[derive(Component, Debug)]
pub struct DelayedFadeDespawn {
    timer: Timer,
    speed: Duration,
}

impl FadeDespawn {
    #[allow(dead_code)]
    pub fn new(duration: Duration) -> Self {
        Self {
            speed: duration.as_secs_f32(),
            value: 1.,
        }
    }

    #[allow(dead_code)]
    pub fn from_secs_f32(duration: f32) -> Self {
        Self {
            speed: duration,
            value: 1.,
        }
    }
}

impl DelayedFadeDespawn {
    pub fn new(delay: Duration, fade: Duration) -> Self {
        Self {
            timer: Timer::new(delay, false),
            speed: fade,
        }
    }
}

impl Plugin for FadeDespawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PostUpdate, despawn.system())
            .add_system(delayed_fade_despawn.system())
            .add_system(fade_despawn.system());
    }
}

fn despawn(mut commands: Commands, query: Query<Entity, With<Despawn>>) {
    for entity in query.iter() {
        log::debug!(?entity, "despawning");
        commands.entity(entity).despawn_recursive();
    }
}

fn delayed_fade_despawn(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DelayedFadeDespawn), With<Handle<ColorMaterial>>>,
    time: Res<Time>,
) {
    for (entity, mut expiry) in query.iter_mut() {
        if expiry.timer.tick(time.delta()).finished() {
            commands
                .entity(entity)
                .remove::<DelayedFadeDespawn>()
                .insert(FadeDespawn::new(expiry.speed));
        }
    }
}

fn fade_despawn(
    mut commands: Commands,
    mut query: Query<(Entity, &mut FadeDespawn, &Handle<ColorMaterial>)>,
    mut color_material_assets: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
) {
    for (entity, mut fadeout, material_handle) in query.iter_mut() {
        fadeout.value = if fadeout.speed > 0. {
            fadeout.value - (1.0 / fadeout.speed) * time.delta_seconds()
        } else {
            0.
        }
        .clamp(0.0, 1.0);

        if fadeout.value <= 0. {
            log::trace!(?entity, "faded");
            commands
                .entity(entity)
                .remove::<FadeDespawn>()
                .insert(Despawn);
        }
        if let Some(material) = color_material_assets.get_mut(material_handle) {
            material.color.set_a(fadeout.value);
        }
    }
}
