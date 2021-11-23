use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use crate::bevoids::{GameState, highscore::Score};

pub(crate) fn display_gameover_menu(
    egui_context: Res<EguiContext>,
    score: Res<Score>,
    mut state: ResMut<State<GameState>>,
) {
    let ctx = egui_context.ctx();
    let score = score.to_string();

    egui::Window::new("GameOver Menu")
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, [0., 0.])
        .show(ctx, |ui| {
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    ui.add(
                        egui::Label::new("Game Over")
                            .heading()
                            .text_color(egui::Color32::WHITE),
                    );
                    ui.add(egui::Separator::default().horizontal().spacing(20.));

                    ui.add(egui::Label::new(score).text_color(egui::Color32::WHITE));

                    if ui.button("Start").clicked() {
                        state.set(GameState::Playing).unwrap();
                    }
                    if ui.button("Main Menu").clicked() {
                        state.set(GameState::MainMenu).unwrap();
                    }
                },
            )
        });

    egui::Window::new("GameOver Hint")
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::RIGHT_BOTTOM, [-5., -5.])
        .show(ctx, |ui| {
            ui.add(egui::Label::new("Hit Enter to try again").small());
        });
}

pub(crate) fn restart_on_enter(
    mut kb: ResMut<Input<KeyCode>>,
    mut state: ResMut<State<GameState>>,
) {
    if kb.just_pressed(KeyCode::Return) {
        kb.reset(KeyCode::Return);
        state.set(GameState::Playing).unwrap();
    }
}
