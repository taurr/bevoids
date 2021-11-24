use bevy::prelude::KeyCode;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub general: General,
    pub laser: Laser,
    pub player: Player,
    pub volume: Volume,
    pub window: Window,
    pub asteroid: Asteroid,
    pub keycodes: KeyCodes,
}

#[derive(Serialize, Deserialize)]
pub struct General {
    pub animation_fps: f32,
    pub background_fade_seconds: f32,
    pub asteroids_in_start_menu: usize,
    pub highscores_capacity: u8,
}

#[derive(Serialize, Deserialize)]
pub struct KeyCodes {
    pub turn_left: Vec<KeyCode>,
    pub turn_right: Vec<KeyCode>,
    pub modifier: Vec<KeyCode>,
    pub accellerate: Vec<KeyCode>,
    pub fire: Vec<KeyCode>,
}

impl Default for KeyCodes {
    fn default() -> Self {
        Self {
            turn_left: vec![KeyCode::Left, KeyCode::A],
            turn_right: vec![KeyCode::Right, KeyCode::D],
            modifier: vec![KeyCode::RControl, KeyCode::LControl],
            accellerate: vec![KeyCode::Up, KeyCode::W],
            fire: vec![KeyCode::Space],
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Asteroid {
    pub max_score: f32,
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

#[derive(Serialize, Deserialize)]
pub struct Window {
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
pub struct Laser {
    pub size: f32,
    pub speed: f32,
    pub lifetime_miliseconds: u64,
    pub fadeout_miliseconds: u64,
}

#[derive(Serialize, Deserialize)]
pub struct Volume {
    pub laser: f32,
    pub thruster: f32,
    pub ship_explosion: f32,
    pub asteroid_explosion: f32,
}
