use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use crate::bevoids::{highscore::Score, GameState};

pub(crate) fn display_paused_menu(
    egui_context: Res<EguiContext>,
    score: Res<Score>,
    mut state: ResMut<State<GameState>>,
) {
    let ctx = egui_context.ctx();
    let score = score.to_string();

    egui::Window::new("Paused Score")
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, [0., 0.])
        .show(ctx, |ui| {
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    ui.add(
                        egui::Label::new("Paused")
                            .heading()
                            .text_color(egui::Color32::WHITE),
                    );
                    ui.add(egui::Separator::default().horizontal().spacing(20.));

                    ui.add(egui::Label::new(score).text_color(egui::Color32::WHITE));

                    if ui.button("Continue").clicked() {
                        state.pop().unwrap();
                    }
                },
            );
        });

    egui::Window::new("Paused Hint")
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::RIGHT_BOTTOM, [-5., -5.])
        .show(ctx, |ui| {
            ui.add(egui::Label::new("Hit Escape to continue").small());
        });
}
