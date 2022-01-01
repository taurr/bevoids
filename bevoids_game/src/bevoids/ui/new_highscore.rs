use bevy::prelude::*;
use bevy_egui::{
    egui::{self, RichText},
    EguiContext,
};

use crate::bevoids::{
    highscore::{save_highscores, HighScore, HighScoreRepository, Score},
    GameState,
};

const TROPHY_TEXTURE_ID: u64 = 0;

pub(crate) fn display_new_highscore_menu(
    mut egui_context: ResMut<EguiContext>,
    score: Res<Score>,
    //textures: Res<TextureAssetMap<GeneralTexture>>,
    mut state: ResMut<State<GameState>>,
    mut name: Local<String>,
    mut highscore_repo: ResMut<HighScoreRepository>,
    mut kb: ResMut<Input<KeyCode>>,
    mut started: Local<bool>,
    assets: Res<AssetServer>,
) {
    let mut hint: String = "".to_string();

    if !*started {
        let texture_handle = assets.load("gfx/trophy.png");
        egui_context.set_egui_texture(TROPHY_TEXTURE_ID, texture_handle);
    }

    egui::Window::new("NewHighScore Menu")
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, [0., 0.])
        .show(egui_context.ctx(), |ui| {
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    ui.add(egui::Label::new(
                        RichText::new("New Highscore")
                            .heading()
                            .color(egui::Color32::WHITE),
                    ));
                    ui.add(egui::Separator::default().horizontal().spacing(20.));
                    ui.add(egui::widgets::Image::new(
                        egui::TextureId::User(TROPHY_TEXTURE_ID),
                        [77., 100.],
                    ));
                    ui.add(egui::Separator::default().horizontal().spacing(20.));
                    ui.add(egui::Label::new(
                        RichText::new(score.to_string()).color(egui::Color32::GREEN),
                    ));

                    ui.add(egui::Separator::default().horizontal().spacing(20.));
                    let name_box =
                        ui.add(egui::TextEdit::singleline(&mut *name).hint_text("Enter your name"));

                    let trimmed_name = name.trim();
                    if trimmed_name.len() >= 3 {
                        hint = "Hit Enter accept".to_string();
                        ui.add(egui::Separator::default().horizontal().spacing(20.));
                        if ui.button("Enter hall of fame").clicked() || name_box.clicked() {
                            kb.reset(KeyCode::Return);
                            highscore_repo
                                .push(HighScore::new(*score, trimmed_name))
                                .expect("failed adding highscore");
                            name.clear();

                            save_highscores(&highscore_repo);

                            *started = false;
                            state.set(GameState::HighScoreMenu).unwrap();
                        }
                    } else {
                        if !*started {
                            *started = true;
                            name_box.request_focus();
                        }
                        hint = "At least 3 charaters required".to_string();
                    }
                },
            )
        });

    egui::Window::new("NewHighScore Hint")
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::RIGHT_BOTTOM, [-5., -5.])
        .show(egui_context.ctx(), |ui| {
            ui.add(egui::Label::new(RichText::new(hint).small()));
        });
}
