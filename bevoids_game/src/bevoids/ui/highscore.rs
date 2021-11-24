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

                    // TODO: implement highscore menu

                    if ui.button("Main Menu").clicked() {
                        state.set(GameState::MainMenu).unwrap();
                    }
                },
            );
        });

    egui::Window::new("HighScore Hint")
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::RIGHT_BOTTOM, [-5., -5.])
        .show(ctx, |ui| {
            ui.add(egui::Label::new("Hit Enter for main menu").small());
        });
}

pub(crate) fn enter_for_mainmenu(
    mut kb: ResMut<Input<KeyCode>>,
    mut state: ResMut<State<GameState>>,
) {
    if kb.just_pressed(KeyCode::Return) {
        kb.reset(KeyCode::Return);
        state.set(GameState::MainMenu).unwrap();
    }
}
