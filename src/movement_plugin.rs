use std::ops::{Deref, DerefMut};
use crate::{GameState, WinSize};
use bevy::{math::vec3, prelude::*};

pub struct MovementPlugin;

#[derive(Debug, Default, Component)]
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
            SystemSet::on_update(GameState::InGame).with_system(linear_movement.system().chain(keep_in_bounds.system())),
        );
    }
}

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
