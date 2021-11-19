use bevy::{
    ecs::{
        schedule::{IntoRunCriteria, RunCriteriaDescriptorOrLabel},
        system::EntityCommands,
    },
    log,
    prelude::*,
};
use std::{cell::Cell, sync::Mutex, time::Duration};

pub struct DespawnPlugin {
    run_criteria: Mutex<Cell<Option<RunCriteriaDescriptorOrLabel>>>,
}

impl Default for DespawnPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl DespawnPlugin {
    pub fn new() -> Self {
        Self {
            run_criteria: Mutex::new(Cell::new(None)),
        }
    }

    pub fn with_run_criteria<Marker, T: IntoRunCriteria<Marker>>(run_criteria: T) -> Self {
        Self {
            run_criteria: Mutex::new(Cell::new(Some(run_criteria.into()))),
        }
    }
}

impl Plugin for DespawnPlugin {
    fn build(&self, app: &mut App) {
        let fade_set = SystemSet::new()
            .with_system(delayed_despawn)
            .with_system(delayed_fade_despawn)
            .with_system(fade_despawn);
        if let Some(r) = self.run_criteria.lock().unwrap().take() {
            app.add_system_set_to_stage(CoreStage::PostUpdate, fade_set.with_run_criteria(r));
        } else {
            app.add_system_set_to_stage(CoreStage::PostUpdate, fade_set);
        }

        app.add_system_set_to_stage(CoreStage::Last, SystemSet::new().with_system(despawn));
    }
}

/// Component used to despawn entities after [Corestage::PostUpdate].
#[derive(Component)]
pub struct Despawn;

/// Component used to despawn entities after a specific duration.
#[derive(Component)]
pub struct DelayedDespawn {
    timer: Timer,
    before_despawn: Option<Box<dyn FnOnce(&mut EntityCommands) + Send + Sync>>,
}

/// Component added to entites that should fade to invisibility, then despawn.
/// Requires the entity to have a [ColorMaterial]
#[derive(Component)]
pub struct FadeDespawn {
    fade_duration: Duration,
    alpha_value: f32,
    before_despawn: Option<Box<dyn FnOnce(&mut EntityCommands) + Send + Sync>>,
}

/// Component added to entites that after a delay should fade to invisibility, then despawn.
/// Requires the entity to have a [ColorMaterial]
#[derive(Component)]
pub struct DelayedFadeDespawn {
    timer: Timer,
    fade_duration: Duration,
    before_fade: Option<Box<dyn FnOnce(&mut EntityCommands) + Send + Sync>>,
    before_despawn: Option<Box<dyn FnOnce(&mut EntityCommands) + Send + Sync>>,
}

impl Default for Despawn {
    fn default() -> Self {
        Self::new()
    }
}

impl Despawn {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }

    #[allow(dead_code)]
    pub fn with_delay(delay: Duration) -> DelayedDespawn {
        DelayedDespawn::from(delay)
    }

    #[allow(dead_code)]
    pub fn fade_over(duration: Duration) -> FadeDespawn {
        FadeDespawn::from(duration)
    }
}

impl From<Duration> for DelayedDespawn {
    fn from(delay: Duration) -> Self {
        Self::new(delay)
    }
}

impl DelayedDespawn {
    #[allow(dead_code)]
    pub fn new(delay: Duration) -> Self {
        Self {
            timer: Timer::new(delay, false),
            before_despawn: None,
        }
    }

    #[allow(dead_code)]
    pub fn before_despawning<F>(self, func: F) -> Self
    where
        F: FnOnce(&mut EntityCommands) + Send + Sync + 'static,
    {
        Self {
            before_despawn: Some(Box::new(func)),
            ..self
        }
    }
}

impl From<Duration> for FadeDespawn {
    fn from(fade_time: Duration) -> Self {
        Self::new(fade_time)
    }
}

impl From<&mut DelayedFadeDespawn> for FadeDespawn {
    fn from(dfd: &mut DelayedFadeDespawn) -> Self {
        Self {
            alpha_value: 1.,
            fade_duration: dfd.fade_duration,
            before_despawn: dfd.before_despawn.take(),
        }
    }
}

impl FadeDespawn {
    #[allow(dead_code)]
    pub fn new(fade_time: Duration) -> Self {
        Self {
            fade_duration: fade_time,
            before_despawn: None,
            alpha_value: 1.,
        }
    }

    #[allow(dead_code)]
    pub fn before_despawn<F>(self, func: F) -> Self
    where
        F: FnOnce(&mut EntityCommands) + Send + Sync + 'static,
    {
        Self {
            before_despawn: Some(Box::new(func)),
            ..self
        }
    }

    #[allow(dead_code)]
    pub fn delay(self, delay: Duration) -> DelayedFadeDespawn {
        if let Some(before_despawn) = self.before_despawn {
            DelayedFadeDespawn::new(delay, self.fade_duration).before_despawn(before_despawn)
        } else {
            DelayedFadeDespawn::new(delay, self.fade_duration)
        }
    }
}

impl DelayedFadeDespawn {
    #[allow(dead_code)]
    pub fn new(delay: Duration, fade_duration: Duration) -> Self {
        Self {
            timer: Timer::new(delay, false),
            fade_duration,
            before_fade: None,
            before_despawn: None,
        }
    }

    #[allow(dead_code)]
    pub fn before_despawn<F>(self, func: F) -> Self
    where
        F: FnOnce(&mut EntityCommands) + Send + Sync + 'static,
    {
        Self {
            before_despawn: Some(Box::new(func)),
            ..self
        }
    }

    #[allow(dead_code)]
    pub fn before_fading<F>(self, func: F) -> Self
    where
        F: FnOnce(&mut EntityCommands) + Send + Sync + 'static,
    {
        Self {
            before_fade: Some(Box::new(func)),
            ..self
        }
    }
}

fn despawn(mut commands: Commands, query: Query<Entity, With<Despawn>>) {
    for entity in query.iter() {
        log::trace!(?entity, "despawning");
        commands.entity(entity).despawn_recursive();
    }
}

fn delayed_despawn(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DelayedDespawn)>,
    time: Res<Time>,
) {
    for (entity, mut expiry) in query.iter_mut() {
        if expiry.timer.tick(time.delta()).finished() {
            let mut entity_commands = commands.entity(entity);
            if let Some(func) = expiry.before_despawn.take() {
                func(&mut entity_commands);
            }
            commands.entity(entity).despawn_recursive();
        }
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
            if let Some(func) = expiry.before_fade.take() {
                func(&mut entity_commands);
            }
            entity_commands
                .remove::<DelayedFadeDespawn>()
                .insert(FadeDespawn::from(&mut *expiry));
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
        fadeout.alpha_value = (fadeout.alpha_value
            - (1.0 / fadeout.fade_duration.as_secs_f32()) * time.delta_seconds())
        .clamp(0., 1.);

        if fadeout.alpha_value <= 0.01 {
            log::trace!(?entity, "faded");
            // reset material
            if let Some(material) = color_material_assets.get_mut(material_handle) {
                material.color.set_a(1.);
            }
            commands.entity(entity).despawn_recursive();
        } else if let Some(material) = color_material_assets.get_mut(material_handle) {
            material.color.set_a(fadeout.alpha_value);
        }
    }
}
