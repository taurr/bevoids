use bevy::{app::AppExit, prelude::*};
use bevy_egui::{
    egui::{self, Align2, Color32, Label},
    EguiContext,
};

use crate::bevoids::GameState;

pub(crate) fn display_main_menu(
    egui_context: Res<EguiContext>,
    mut state: ResMut<State<GameState>>,
    mut exit: EventWriter<AppExit>,
) {
    let ctx = egui_context.ctx();

    egui::Window::new("MainMenu Menu")
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::CENTER_CENTER, [0., 0.])
        .show(ctx, |ui| {
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    ui.add(Label::new("Game Menu").heading().text_color(Color32::WHITE));
                    ui.add(egui::Separator::default().horizontal().spacing(20.));

                    if ui.button("Start").clicked() {
                        state.set(GameState::Playing).unwrap();
                    }
                    if ui.button("Highscores").clicked() {
                        state.set(GameState::HighScoreMenu).unwrap();
                    }
                    if ui.button("Exit").clicked() {
                        exit.send(AppExit);
                    }
                },
            );
        });

    egui::Window::new("MainMenu Hint")
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::RIGHT_BOTTOM, [-5., -5.])
        .show(ctx, |ui| {
            ui.add(egui::Label::new("Hit Enter to start").small());
        });
}
