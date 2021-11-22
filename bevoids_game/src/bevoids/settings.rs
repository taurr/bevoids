use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub general: General,
    pub laser: Laser,
    pub player: Player,
    pub volume: Volume,
    pub window: Window,
    pub asteroid: Asteroid,
}

#[derive(Deserialize)]
pub struct Asteroid {
    pub size_max: f32,
    pub size_min: f32,
    pub speed_max: f32,
    pub speed_min: f32,
    pub zpos_min: f32,
    pub zpos_max: f32,
    pub spawndelay_seconds: f32,
    pub spawndelay_initial_seconds: f32,
    pub spawndelay_min_seconds: f32,
    pub spawndelay_multiplier: f32,
    pub spawn_player_distance: f32,
    pub split_number: u32,
    pub split_size_factor: f32,
}

#[derive(Deserialize)]
pub struct General {
    pub animation_fps: f32,
    pub max_score: f32,
    pub asteroids_in_start_menu: usize,
}

#[derive(Deserialize)]
pub struct Window {
    pub width: u32,
    pub height: u32,
}

#[derive(Deserialize)]
pub struct Player {
    pub size: f32,
    pub zpos: f32,
    pub gun_ypos: f32,
    pub accelleration: f32,
    pub decelleration: f32,
    pub turn_speed_slow: f32,
    pub turn_speed_fast: f32,
    pub max_speed: f32,
    pub flame_width: f32,
    pub flame_ypos: f32,
}

#[derive(Deserialize)]
pub struct Laser {
    pub size: f32,
    pub speed: f32,
    pub lifetime_miliseconds: u64,
    pub fadeout_miliseconds: u64,
}

#[derive(Deserialize)]
pub struct Volume {
    pub laser: f32,
    pub thruster: f32,
    pub ship_explosion: f32,
    pub asteroid_explosion: f32,
}
