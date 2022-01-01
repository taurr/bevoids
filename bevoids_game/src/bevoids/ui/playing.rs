use bevy::prelude::*;
use bevy_egui::{
    egui::{self, RichText},
    EguiContext,
};

use crate::bevoids::highscore::Score;

pub(crate) fn display_playing_ui(egui_context: Res<EguiContext>, score: Res<Score>) {
    let ctx = egui_context.ctx();

    let score = score.to_string();

    egui::Window::new("Playing Score")
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::RIGHT_TOP, [-20., 10.])
        .show(ctx, |ui| {
            ui.add(egui::Label::new(
                RichText::new(score).color(egui::Color32::WHITE),
            ));
        });

    egui::Window::new("Playing Hint")
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::RIGHT_BOTTOM, [-5., -5.])
        .show(ctx, |ui| {
            ui.add(egui::Label::new(
                RichText::new("Hit Escape to pause").small(),
            ));
        });
}
