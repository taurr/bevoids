pub use bevy_embasset::EnumCount;

bevy_embasset::assets!(
    pub enum SpriteAsset {
        GfxExplosion = "explosion.png",
        GfxFlame = "flame.png",
        GfxLaser = "laser.png",
        GfxSpaceship = "spaceship.png",
        GfxTrophy = "trophy.png",
    },
    pub struct SpriteAssetIo {
        root = "../assets/gfx/"
    }
);

bevy_embasset::assets!(
    pub enum AsteroidAsset {
        GfxAsteroids1 = "asteroid_1.png",
        GfxAsteroids2 = "asteroid_2.png",
        GfxAsteroids3 = "asteroid_3.png",
        GfxAsteroids4 = "asteroid_4.png",
        GfxAsteroids5 = "asteroid_5.png",
        GfxAsteroids6 = "asteroid_6.png",
        GfxAsteroids7 = "asteroid_7.png",
    },
    pub struct AsteroidAssetIo {
        root = "../assets/gfx/asteroids/"
    }
);

bevy_embasset::assets!(
    pub enum BackgroundAsset {
        Backgrounds1 = "bg_1.jpg",
        Backgrounds2 = "bg_2.jpg",
        Backgrounds3 = "bg_3.jpg",
        Backgrounds4 = "bg_4.jpg",
        Backgrounds5 = "bg_5.jpg",
        Backgrounds6 = "bg_6.jpg",
        Backgrounds7 = "bg_7.jpg",
        Backgrounds8 = "bg_8.jpg",
        Backgrounds9 = "bg_12.jpg",
        Backgrounds10 = "bg_9.jpg",
        Backgrounds11 = "bg_10.jpg",
        Backgrounds12 = "bg_11.jpg",
    },
    pub struct BackgroundAssetIo {
        root = "../assets/gfx/backgrounds/"
    }
);

bevy_embasset::assets!(
    pub enum SoundAsset {
        AsteroidExplode = "asteroid_explode.wav",
        Laser = "laser.wav",
        Notification = "notification.wav",
        ShipExplode = "ship_explode.wav",
        Thruster = "thruster.wav",
    },
    pub struct SoundAssetIo {
        root = "../assets/sounds/"
    }
);

#[cfg(test)]
mod tests {
    use bevy::asset::AssetPath;
    use bevy_embasset::AssetIoAlternative;

    use super::*;

    #[test]
    fn sound_asset_io_is_asset_io_alternative() {
        fn assert<T: Into<AssetIoAlternative>>() {}
        assert::<SoundAssetIo>();
    }

    #[test]
    fn sound_asset_is_into_asset_path() {
        fn assert<'a, T: 'a + Into<AssetPath<'a>>>() {}
        assert::<SoundAsset>();
    }
}
