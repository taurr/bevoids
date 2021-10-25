use crate::GameState;
use bevy::prelude::*;
use std::time::Duration;

#[derive(Debug, Default, Component)]
pub struct Fadeout {
    speed: f32,
    value: f32,
}
impl Fadeout {
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

#[derive(Debug, Default, Component)]
pub struct Fadein {
    speed: f32,
    value: f32,
}
impl Fadein {
    #[allow(dead_code)]
    pub fn new(duration: Duration) -> Self {
        Self {
            speed: duration.as_secs_f32(),
            value: 0.,
        }
    }

    #[allow(dead_code)]
    pub fn from_secs_f32(duration: f32) -> Self {
        Self {
            speed: duration,
            value: 0.,
        }
    }
}

#[derive(Debug, Default, Component)]
pub struct DelayedFadeout {
    timer: Timer,
    speed: Duration,
}
#[allow(dead_code)]
impl DelayedFadeout {
    pub fn new(delay: Duration, fade: Duration) -> Self {
        Self {
            timer: Timer::new(delay, false),
            speed: fade,
        }
    }
}

pub struct FadePlugin;

impl Plugin for FadePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameState::InGame)
                .with_system(delayed_fadeout.system())
                .with_system(fadeout.system())
                .with_system(fadein.system()),
        );
    }
}

fn delayed_fadeout(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DelayedFadeout), With<Handle<ColorMaterial>>>,
    time: Res<Time>,
) {
    for (entity, mut expiry) in query.iter_mut() {
        if expiry.timer.tick(time.delta()).finished() {
            commands
                .entity(entity)
                .remove::<DelayedFadeout>()
                .insert(Fadeout::new(expiry.speed));
        }
    }
}

fn fadeout(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Fadeout, &Handle<ColorMaterial>)>,
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
            commands.entity(entity).despawn_recursive();
        }
        if let Some(material) = color_material_assets.get_mut(material_handle) {
            material.color.set_a(fadeout.value);
        }
    }
}

fn fadein(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Fadein, &mut Visible, &Handle<ColorMaterial>)>,
    mut color_material_assets: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
) {
    for (entity, mut fadein, mut visible, material_handle) in query.iter_mut() {
        fadein.value = if fadein.speed > 0. {
            fadein.value + (1.0 / fadein.speed) * time.delta_seconds()
        } else {
            1.
        }
        .clamp(0.0, 1.0);

        if fadein.value >= 1.0 {
            commands.entity(entity).remove::<Fadein>();
        }
        if let Some(material) = color_material_assets.get_mut(material_handle) {
            material.color.set_a(fadein.value);
            visible.is_visible = true;
        }
    }
}
