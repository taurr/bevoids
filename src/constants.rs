use std::f32::consts::PI;

pub(crate) const WIN_WIDTH: f32 = 1024.;

pub(crate) const WIN_HEIGHT: f32 = 800.;

pub(crate) const DIFFICULTY_RAISER_TIMESTEP: f64 = 20.;
pub(crate) const DIFFICULTY_RAISER_SPAWN_DELAY_MULTIPLIER: f32 = 0.95;
pub(crate) const ASTEROID_START_SPAWN_DELAY: f32 = 15.;

pub(crate) const ASTEROID_MAX_SCORE: f32 = 100.;

pub(crate) const ASTEROID_SPLIT_INTO: usize = 2;

pub(crate) const ASTEROID_SPLIT_SIZE_RATIO: f32 = 2. / 3.;

pub(crate) const ASTEROIDS_PLAYER_SPAWN_DISTANCE: f32 = 200.;

pub(crate) const ASTEROIDS_MAX_ACTIVE: usize = 500;

pub(crate) const ASTEROID_Z_MIN: f32 = 100.;

pub(crate) const ASTEROID_Z_MAX: f32 = 200.;

pub(crate) const ASTEROID_MIN_SIZE: f32 = 20.;

pub(crate) const ASTEROID_MAX_SIZE: f32 = 150.;

pub(crate) const ASTEROID_MIN_SPEED: f32 = 25.;

pub(crate) const ASTEROID_MAX_SPEED: f32 = 125.;

pub(crate) const ASTEROID_FADEOUT_SECONDS: f32 = 0.20;

pub(crate) const BULLET_PLAYER_RELATIVE_Z: f32 = -1.;

pub(crate) const BULLET_PLAYER_RELATIVE_Y: f32 = 20.;

pub(crate) const BULLET_MAX_SIZE: f32 = 25.;

pub(crate) const BULLET_SPEED: f32 = 500.;

pub(crate) const BULLET_LIFETIME_SECONDS: f32 = 1.5;

pub(crate) const BULLET_FADEOUT_SECONDS: f32 = 0.25;

pub(crate) const FLAME_RELATIVE_Z: f32 = -10.;

pub(crate) const FLAME_RELATIVE_Y: f32 = -32.;

pub(crate) const FLAME_WIDTH: f32 = 15.;

pub(crate) const PLAYER_Z: f32 = 900.;

pub(crate) const PLAYER_MAX_SIZE: f32 = 50.;

pub(crate) const PLAYER_TURN_SPEED: f32 = 1.5 * PI;

pub(crate) const PLAYER_ACCELLERATION: f32 = 250.;

pub(crate) const PLAYER_DECCELLERATION: f32 = 33.3;

pub(crate) const PLAYER_START_SPEED: f32 = 200.;

pub(crate) const PLAYER_MAX_SPEED: f32 = 800.;

pub(crate) const PLAYER_FADEOUT_SECONDS: f32 = 0.5;

pub(crate) const AUDIO_LASER: &str = "sounds/laser.wav";
pub(crate) const AUDIO_CHANNEL_LASER: &str = "laser";
pub(crate) const AUDIO_LASER_VOLUME: f32 = 1.0;
pub(crate) const AUDIO_THRUSTER: &str = "sounds/thruster.wav";
pub(crate) const AUDIO_CHANNEL_THRUSTER: &str = "thruster";
pub(crate) const AUDIO_THRUSTER_VOLUME: f32 = 0.5;
pub(crate) const AUDIO_EXPLOSION_SHIP: &str = "sounds/ship_explode.wav";
pub(crate) const AUDIO_CHANNEL_EXPLOSION_SHIP: &str = "ship";
pub(crate) const AUDIO_EXPLOSION_SHIP_VOLUME: f32 = 1.0;
pub(crate) const AUDIO_EXPLOSION_ASTEROID: &str = "sounds/asteroid_explode.wav";
pub(crate) const AUDIO_CHANNEL_EXPLOSION_ASTEROID: &str = "asteroid";
pub(crate) const AUDIO_EXPLOSION_ASTEROID_VOLUME: f32 = 1.0;
