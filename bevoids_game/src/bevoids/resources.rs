use bevoids_assets::*;
use bevy::prelude::*;
use bevy_effects::animation::TextureAtlasMap;

pub(crate) fn define_animations(
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut texture_atlas_map: ResMut<TextureAtlasMap>,
) {
    let texture_atlas = TextureAtlas::from_grid(
        asset_server.load(SpriteAsset::GfxExplosion),
        Vec2::new(100.0, 100.0),
        9,
        9,
    );
    texture_atlas_map.insert(
        SpriteAsset::GfxExplosion,
        texture_atlas,
        &mut texture_atlases,
    );
}
