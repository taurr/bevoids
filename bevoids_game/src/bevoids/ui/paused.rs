use bevy::prelude::*;
use bevy_egui::{
    egui::{self, RichText},
    EguiContext,
};

use crate::bevoids::{highscore::Score, GameState};

pub(crate) fn display_paused_menu(
    egui_context: Res<EguiContext>,
    score: Res<Score>,
    mut started: Local<bool>,
    mut state: ResMut<State<GameState>>,
) {
    let ctx = egui_context.ctx();
    let score = score.to_string();
    let mut hint: String = "".to_string();

    egui::Window::new("Paused Score")
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, [0., 0.])
        .show(ctx, |ui| {
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    ui.add(egui::Label::new(
                        RichText::new("Paused")
                            .heading()
                            .color(egui::Color32::WHITE),
                    ));
                    ui.add(egui::Separator::default().horizontal().spacing(20.));

                    ui.add(egui::Label::new(
                        RichText::new(score).color(egui::Color32::WHITE),
                    ));

                    let continue_button = ui.button("Continue");
                    if continue_button.clicked() {
                        *started = false;
                        state.pop().unwrap();
                    }

                    if !*started {
                        *started = true;
                        continue_button.request_focus();
                    }
                    hint = "Hit Escape or Enter to continue".to_string();
                },
            );
        });

    egui::Window::new("Paused Hint")
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::RIGHT_BOTTOM, [-5., -5.])
        .show(ctx, |ui| {
            ui.add(egui::Label::new(RichText::new(hint).small()));
        });
}
