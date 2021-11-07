use std::f32::consts::PI;

use bevy::{app::Events, prelude::*, window::WindowResized};
use derive_more::AsRef;
use parry2d::{
    bounding_volume::{BoundingSphere, AABB},
    math::Point,
};

pub struct BoundsPlugin;

impl Plugin for BoundsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PostStartup, initialize_window_bounds)
            .add_system(resized.system());
    }
}

#[derive(Debug, Component, Copy, Clone, AsRef)]
pub struct Bounds {
    #[as_ref]
    aabb: AABB,
    #[as_ref]
    sphere: BoundingSphere,
}

impl Bounds {
    pub fn from_pos_and_size(position: Vec2, size: Vec2) -> Bounds {
        let (w, h) = (size.x / 2., size.y / 2.);
        let aabb = AABB::new(
            Point::from([position.x - w, position.y - h]),
            Point::from([position.x + w, position.y + h]),
        );
        let sphere = Bounds::create_sphere(&aabb);
        Self { aabb, sphere }
    }

    pub fn from_window(window: &Window) -> Self {
        Bounds::from_pos_and_size(Vec2::ZERO, Vec2::from((window.width(), window.height())))
    }

    pub fn size(&self) -> Vec2 {
        let extents = self.aabb.extents();
        Vec2::from((extents.x, extents.y))
    }

    pub fn width(&self) -> f32 {
        self.aabb.extents().x
    }

    pub fn height(&self) -> f32 {
        self.aabb.extents().y
    }

    pub fn set_center(&mut self, position: Vec2) {
        self.aabb = AABB::from_half_extents(
            Point::from([position.x, position.y]),
            self.aabb.half_extents(),
        );
        self.sphere = Bounds::create_sphere(&self.aabb);
    }

    pub fn as_aabb(&self) -> AABB {
        self.aabb
    }

    pub fn as_sphere(&self) -> BoundingSphere {
        self.sphere
    }

    fn create_sphere(aabb: &AABB) -> BoundingSphere {
        let ext = aabb.half_extents();
        let radius = f32::cos(PI / 4.0) * (ext.x + ext.y) / 2.;
        BoundingSphere::new(aabb.center(), radius)
    }
}

fn initialize_window_bounds(mut commands: Commands, mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    commands.insert_resource(Bounds::from_window(window));
}

fn resized(resize_event: Res<Events<WindowResized>>, mut bounds: ResMut<Bounds>) {
    let mut reader = resize_event.get_reader();
    for e in reader.iter(&resize_event) {
        *bounds = Bounds::from_pos_and_size(Vec2::ZERO, Vec2::new(e.width, e.height));
    }
}
