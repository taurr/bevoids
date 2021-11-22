use bevy::prelude::*;
use bevy_asset_map::{GfxBounds, TextureAssetMap};
use bevy_egui::{
    egui::{self, Align2, Color32, Label},
    EguiContext,
};
use rand::Rng;

use crate::bevoids::{
    asteroids::SpawnAsteroidEvent,
    player::{spawn_background, Background},
    resources::BackgroundTexture,
    settings::Settings,
    GameState,
};

pub(crate) fn set_menu_background(
    mut commands: Commands,
    mut color_assets: ResMut<Assets<ColorMaterial>>,
    background_query: Query<Entity, With<Background>>,
    background_asset_map: Res<TextureAssetMap<BackgroundTexture>>,
    win_bounds: Res<GfxBounds>,
    settings: Res<Settings>,
    mut spawn_event: EventWriter<SpawnAsteroidEvent>,
) {
    spawn_background(
        &background_asset_map,
        &mut color_assets,
        &background_query,
        &win_bounds,
        &mut commands,
    );

    let mut rng = rand::thread_rng();
    for _ in 0..settings.general.asteroids_in_start_menu {
        spawn_event.send(SpawnAsteroidEvent::new(
            rng.gen_range(settings.asteroid.size_min..settings.asteroid.size_max),
            None,
        ));
    }
}

pub(crate) fn start_menu(egui_context: Res<EguiContext>, mut state: ResMut<State<GameState>>) {
    let ctx = egui_context.ctx();

    egui::Window::new("Menu")
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::CENTER_CENTER, [0., 0.])
        .show(ctx, |ui| {
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    ui.add(Label::new("Game Menu").heading().text_color(Color32::WHITE));
                    if ui.button("Start").clicked() {
                        state.set(GameState::Playing).unwrap();
                    }
                    ui.add(
                        Label::new("Hit Enter to start")
                            .small()
                            .text_color(Color32::WHITE),
                    );
                },
            );
        });
}
