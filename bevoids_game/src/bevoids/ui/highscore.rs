use bevoids_assets::SpriteAsset;
use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align2, Color32, Label, RichText, ScrollArea},
    EguiContext,
};

use crate::bevoids::{highscore::HighScoreRepository, GameState};

const TROPHY_TEXTURE_ID: u64 = 0;

pub(crate) fn display_highscore_menu_system(
    mut egui_context: ResMut<EguiContext>,
    mut state: ResMut<State<GameState>>,
    highscores: Res<HighScoreRepository>,
    mut started: Local<bool>,
    assets: Res<AssetServer>,
) {
    let mut hint: String = "".to_string();

    if !*started {
        let texture_handle = assets.load(SpriteAsset::GfxTrophy);
        egui_context.set_egui_texture(TROPHY_TEXTURE_ID, texture_handle);
    }

    egui::Window::new("HighScore Menu")
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::CENTER_CENTER, [0., 0.])
        .default_width(480.)
        .show(egui_context.ctx_mut(), |ui| {
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    ui.horizontal(|ui| {
                        const HSPACE: f32 = 10.;
                        const VTROPHY: f32 = 50.;
                        const HTROPHY: f32 = VTROPHY * 0.77;
                        let x_spacing = ui.spacing().item_spacing.x;
                        let twidth =
                            ui.available_width() - 2. * HTROPHY - 2. * HSPACE - 2. * x_spacing;

                        ui.add_space(HSPACE);
                        ui.add(egui::widgets::Image::new(
                            egui::TextureId::User(TROPHY_TEXTURE_ID),
                            [HTROPHY, VTROPHY],
                        ));
                        ui.add_sized(
                            [twidth, VTROPHY],
                            Label::new(RichText::new("Highscores").heading().color(Color32::WHITE)),
                        );
                        ui.add(egui::widgets::Image::new(
                            egui::TextureId::User(TROPHY_TEXTURE_ID),
                            [HTROPHY, VTROPHY],
                        ));
                    });

                    ui.add(egui::Separator::default().horizontal().spacing(20.));

                    let row_height = ui.fonts()[egui::TextStyle::Body].row_height();
                    let num_rows = highscores.count();
                    ScrollArea::vertical()
                        .max_height(row_height * 11. + 4.)
                        .show_rows(ui, row_height, num_rows, |ui, row_range| {
                            for (n, highscore) in highscores
                                .iter()
                                .skip(row_range.start)
                                .take(row_range.end - row_range.start)
                                .enumerate()
                            {
                                ui.horizontal(|ui| {
                                    ui.add(Label::new(
                                        RichText::new(format!("{: >3}", 1 + n + row_range.start))
                                            .small()
                                            .color(Color32::LIGHT_BLUE),
                                    ));
                                    ui.add(Label::new(
                                        RichText::new(format!("{: >9}", highscore.score()))
                                            .monospace()
                                            .color(Color32::WHITE),
                                    ));
                                    ui.add(Label::new(
                                        RichText::new(format!("{}", highscore.name(),))
                                            .monospace()
                                            .color(Color32::WHITE),
                                    ));
                                });
                            }
                        });

                    ui.add(egui::Separator::default().horizontal().spacing(20.));
                    let mainmenu_button = ui.button("Main Menu");

                    if !*started {
                        *started = true;
                        mainmenu_button.request_focus();
                    }

                    if mainmenu_button.has_focus() {
                        hint = "Hit Enter for main menu".to_string();
                    }
                    if mainmenu_button.clicked() {
                        *started = false;
                        state.set(GameState::MainMenu).unwrap();
                    }
                },
            );
        });

    egui::Window::new("HighScore Hint")
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::RIGHT_BOTTOM, [-5., -5.])
        .show(egui_context.ctx_mut(), |ui| {
            ui.add(egui::Label::new(RichText::new(hint).small()));
        });
}
