use bevy::prelude::*;
use bevy_egui::{
    egui::{self, RichText},
    EguiContext,
};

use crate::bevoids::{highscore::Score, GameState};

pub(crate) fn display_gameover_menu(
    egui_context: Res<EguiContext>,
    score: Res<Score>,
    mut state: ResMut<State<GameState>>,
    mut started: Local<bool>,
) {
    let ctx = egui_context.ctx();
    let score = score.to_string();
    let mut hint: String = "".to_string();

    egui::Window::new("GameOver Menu")
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, [0., 0.])
        .show(ctx, |ui| {
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    ui.add(egui::Label::new(
                        RichText::new("Game Over")
                            .heading()
                            .color(egui::Color32::WHITE),
                    ));
                    ui.add(egui::Separator::default().horizontal().spacing(20.));

                    ui.add(egui::Label::new(
                        RichText::new(score).color(egui::Color32::GREEN),
                    ));

                    let play_button = ui.button("Try Again");
                    if play_button.clicked() {
                        *started = false;
                        state.set(GameState::Playing).unwrap();
                    }
                    let mainmenu_button = ui.button("Main Menu");
                    if mainmenu_button.clicked() {
                        *started = false;
                        state.set(GameState::MainMenu).unwrap();
                    }

                    if play_button.has_focus() {
                        hint = "Hit Enter to try again".to_string();
                    } else if mainmenu_button.has_focus() {
                        hint = "Hit Enter for mainmenu".to_string();
                    } else if !*started {
                        *started = true;
                        play_button.request_focus();
                    }
                },
            )
        });

    egui::Window::new("GameOver Hint")
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::RIGHT_BOTTOM, [-5., -5.])
        .show(ctx, |ui| {
            ui.add(egui::Label::new(RichText::new(hint).small()));
        });
}
