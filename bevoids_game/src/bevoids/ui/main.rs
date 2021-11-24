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
    mut started: Local<bool>,
) {
    let ctx = egui_context.ctx();
    let mut hint: String = "".to_string();

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

                    let start_button = ui.button("Play");
                    if start_button.clicked() {
                        *started = false;
                        state.set(GameState::Playing).unwrap();
                    }
                    let highscore_button = ui.button("Highscores");
                    if highscore_button.clicked() {
                        *started = false;
                        state.set(GameState::HighScoreMenu).unwrap();
                    }
                    let exit_button = ui.button("Exit");
                    if exit_button.clicked() {
                        exit.send(AppExit);
                    }

                    if start_button.has_focus() {
                        hint = "Hit Enter to play".to_string();
                    } else if highscore_button.has_focus() {
                        hint = "Hit Enter to view highscores".to_string();
                    } else if exit_button.has_focus() {
                        hint = "Hit Enter to exit".to_string();
                    } else if !*started {
                        *started = true;
                        start_button.request_focus();
                    }
                },
            );
        });

    egui::Window::new("MainMenu Hint")
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::RIGHT_BOTTOM, [-5., -5.])
        .show(ctx, |ui| {
            ui.add(egui::Label::new(hint).small());
        });
}
