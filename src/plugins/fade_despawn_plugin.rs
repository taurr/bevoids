use bevy::{ecs::system::EntityCommands, log, prelude::*};
use std::time::Duration;

pub struct FadeDespawnPlugin;

#[derive(Component, Debug, Reflect)]
pub struct Despawn;

#[derive(Component, Debug, Reflect)]
pub struct FadeDespawn {
    speed: f32,
    value: f32,
}

#[derive(Component)]
pub struct DelayedFadeDespawn {
    timer: Timer,
    speed: Duration,
    func: Option<Box<dyn FnOnce(&mut EntityCommands) + Send + Sync>>,
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
            func: None,
        }
    }
    pub fn before_fading<F>(mut self, func: F) -> Self
    where
        F: FnOnce(&mut EntityCommands) + Send + Sync + 'static,
    {
        self.func = Some(Box::new(func));
        self
    }
}

impl Plugin for FadeDespawnPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<FadeDespawn>();
        app.register_type::<Despawn>();

        app.add_system_to_stage(
            CoreStage::PostUpdate,
            despawn.system().label("system_despawn"),
        )
        .add_system(delayed_fade_despawn.system().before("system_despawn"))
        .add_system(fade_despawn.system().before("system_despawn"));
    }
}

fn despawn(mut commands: Commands, query: Query<Entity, With<Despawn>>) {
    for entity in query.iter() {
        log::trace!(?entity, "despawning");
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
            let mut entity_commands = commands.entity(entity);
            if let Some(func) = expiry.func.take() {
                func(&mut entity_commands);
            }
            entity_commands
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
        fadeout.value =
            (fadeout.value - (1.0 / fadeout.speed) * time.delta_seconds()).clamp(0., 1.);

        if fadeout.value <= 0. {
            log::trace!(?entity, "faded");
            if let Some(material) = color_material_assets.get_mut(material_handle) {
                material.color.set_a(1.);
            }
            commands
                .entity(entity)
                .remove::<FadeDespawn>()
                .insert(Despawn);
        } else if let Some(material) = color_material_assets.get_mut(material_handle) {
            material.color.set_a(fadeout.value);
        }
    }
}
