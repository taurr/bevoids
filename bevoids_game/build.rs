use std::{env, path::Path};

fn main() {
    // bevy_embasset::include_all_assets(
    //     &Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("assets"),
    // );
    if bevy_embasset::include_assets(
        &Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("assets"),
        &[
            "sounds/asteroid_explode.wav",
            "sounds/laser.wav",
            "sounds/notification.wav",
            "sounds/ship_explode.wav",
            "sounds/thruster.wav",
            "gfx/asteroids/asteroid_1.png",
            "gfx/asteroids/asteroid_2.png",
            "gfx/asteroids/asteroid_3.png",
            "gfx/asteroids/asteroid_4.png",
            "gfx/asteroids/asteroid_5.png",
            "gfx/asteroids/asteroid_6.png",
            "gfx/asteroids/asteroid_7.png",
            "gfx/backgrounds/bg_1.jpg",
            "gfx/backgrounds/bg_2.jpg",
            "gfx/backgrounds/bg_3.jpg",
            "gfx/backgrounds/bg_4.jpg",
            "gfx/backgrounds/bg_5.jpg",
            "gfx/backgrounds/bg_6.jpg",
            "gfx/backgrounds/bg_7.jpg",
            "gfx/backgrounds/bg_8.jpg",
            "gfx/backgrounds/bg_9.jpg",
            "gfx/backgrounds/bg_10.jpg",
            "gfx/backgrounds/bg_11.jpg",
            "gfx/backgrounds/bg_12.jpg",
            "gfx/explosion.png",
            "gfx/flame.png",
            "gfx/laser.png",
            "gfx/spaceship.png",
            "gfx/trophy.png",
        ],
    )
    .is_err()
    {
        std::process::exit(-1);
    }
}
