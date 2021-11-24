use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align2, Color32, Label},
    EguiContext,
};

use crate::bevoids::GameState;

pub(crate) fn display_highscore_menu(
    egui_context: Res<EguiContext>,
    mut state: ResMut<State<GameState>>,
) {
    let ctx = egui_context.ctx();
    let mut hint: String = "".to_string();

    egui::Window::new("HighScore Menu")
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::CENTER_CENTER, [0., 0.])
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

                    // TODO: implement highscore menu

                    let mainmenu_button = ui.button("Main Menu");
                    if mainmenu_button.clicked() {
                        state.set(GameState::MainMenu).unwrap();
                    }

                    if mainmenu_button.has_focus() {
                        hint = "Hit Enter for main menu".to_string();
                    } else {
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
