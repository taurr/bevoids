use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align2, Color32, Label, ScrollArea},
    EguiContext,
};

use crate::bevoids::{highscore::HighScoreRepository, GameState};

pub(crate) fn display_highscore_menu(
    egui_context: Res<EguiContext>,
    mut state: ResMut<State<GameState>>,
    highscores: Res<HighScoreRepository>,
    mut started: Local<bool>,
) {
    let ctx = egui_context.ctx();
    let mut hint: String = "".to_string();

    egui::Window::new("HighScore Menu")
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::CENTER_CENTER, [0., 0.])
        .default_width(480.)
        .show(ctx, |ui| {
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    ui.add(
                        Label::new("Highscores")
                            .heading()
                            .text_color(Color32::WHITE),
                    );
                    ui.add(egui::Separator::default().horizontal().spacing(20.));
                    // TODO: display 2 small trophys - 1 in each side

                    let row_height = ui.fonts()[egui::TextStyle::Body].row_height();
                    let num_rows = highscores.count();
                    ScrollArea::from_max_height(row_height * 11. + 4.).show_rows(
                        ui,
                        row_height,
                        num_rows,
                        |ui, row_range| {
                            for (n, highscore) in highscores
                                .iter()
                                .skip(row_range.start)
                                .take(row_range.end - row_range.start)
                                .enumerate()
                            {
                                ui.horizontal(|ui| {
                                    ui.add(
                                        Label::new(format!("{: >3}", 1 + n + row_range.start))
                                            .small()
                                            .text_color(Color32::LIGHT_BLUE),
                                    );
                                    ui.add(
                                        Label::new(format!("{: >9}", highscore.score()))
                                            .monospace()
                                            .text_color(Color32::WHITE),
                                    );
                                    ui.add(
                                        Label::new(format!("{}", highscore.name(),))
                                            .text_color(Color32::WHITE)
                                            .monospace(),
                                    );
                                });
                            }
                        },
                    );

                    ui.add(egui::Separator::default().horizontal().spacing(20.));
                    let mainmenu_button = ui.button("Main Menu");
                    if mainmenu_button.clicked() {
                        *started = false;
                        state.set(GameState::MainMenu).unwrap();
                    }

                    if mainmenu_button.has_focus() {
                        hint = "Hit Enter for main menu".to_string();
                    } else if !*started {
                        *started = true;
                        mainmenu_button.request_focus();
                    }
                },
            );
        });

    egui::Window::new("HighScore Hint")
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::RIGHT_BOTTOM, [-5., -5.])
        .show(ctx, |ui| {
            ui.add(egui::Label::new(hint).small());
        });
}
