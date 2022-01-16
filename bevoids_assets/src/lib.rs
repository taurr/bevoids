use std::path::Path;

use bevy::asset::AssetIo;
use bevy_embasset::EmbassetIo;

pub struct BevoidsAssets(EmbassetIo);

macro_rules! include_asset {
    ($io:ident, $asset:literal) => {
        $io.add_embedded_asset(
            Path::new($asset),
            include_bytes!(concat!("../assets/", $asset)),
        );
    };
}

fn add_embasset_assets(io: &mut EmbassetIo) {
    include_asset!(io, "sounds/asteroid_explode.wav");
    include_asset!(io, "sounds/laser.wav");
    include_asset!(io, "sounds/notification.wav");
    include_asset!(io, "sounds/ship_explode.wav");
    include_asset!(io, "sounds/thruster.wav");
    include_asset!(io, "gfx/asteroids/asteroid_1.png");
    include_asset!(io, "gfx/asteroids/asteroid_2.png");
    include_asset!(io, "gfx/asteroids/asteroid_3.png");
    include_asset!(io, "gfx/asteroids/asteroid_4.png");
    include_asset!(io, "gfx/asteroids/asteroid_5.png");
    include_asset!(io, "gfx/asteroids/asteroid_6.png");
    include_asset!(io, "gfx/asteroids/asteroid_7.png");
    include_asset!(io, "gfx/backgrounds/bg_1.jpg");
    include_asset!(io, "gfx/backgrounds/bg_2.jpg");
    include_asset!(io, "gfx/backgrounds/bg_3.jpg");
    include_asset!(io, "gfx/backgrounds/bg_4.jpg");
    include_asset!(io, "gfx/backgrounds/bg_5.jpg");
    include_asset!(io, "gfx/backgrounds/bg_6.jpg");
    include_asset!(io, "gfx/backgrounds/bg_7.jpg");
    include_asset!(io, "gfx/backgrounds/bg_8.jpg");
    include_asset!(io, "gfx/backgrounds/bg_9.jpg");
    include_asset!(io, "gfx/backgrounds/bg_10.jpg");
    include_asset!(io, "gfx/backgrounds/bg_11.jpg");
    include_asset!(io, "gfx/backgrounds/bg_12.jpg");
    include_asset!(io, "gfx/explosion.png");
    include_asset!(io, "gfx/flame.png");
    include_asset!(io, "gfx/laser.png");
    include_asset!(io, "gfx/spaceship.png");
    include_asset!(io, "gfx/trophy.png");
}

impl Default for BevoidsAssets {
    fn default() -> Self {
        Self::new()
    }
}

impl BevoidsAssets {
    pub fn new() -> Self {
        let mut io = EmbassetIo::new();
        add_embasset_assets(&mut io);
        Self(io)
    }
}

impl AssetIo for BevoidsAssets {
    fn load_path<'a>(
        &'a self,
        path: &'a std::path::Path,
    ) -> bevy::asset::BoxedFuture<'a, Result<Vec<u8>, bevy::asset::AssetIoError>> {
        self.0.load_path(path)
    }

    fn read_directory(
        &self,
        path: &std::path::Path,
    ) -> Result<Box<dyn Iterator<Item = std::path::PathBuf>>, bevy::asset::AssetIoError> {
        self.0.read_directory(path)
    }

    fn is_directory(&self, path: &std::path::Path) -> bool {
        self.0.is_directory(path)
    }

    fn watch_path_for_changes(
        &self,
        path: &std::path::Path,
    ) -> Result<(), bevy::asset::AssetIoError> {
        self.0.watch_path_for_changes(path)
    }

    fn watch_for_changes(&self) -> Result<(), bevy::asset::AssetIoError> {
        self.0.watch_for_changes()
    }
}
