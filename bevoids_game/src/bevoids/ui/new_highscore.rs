use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use crate::bevoids::{
    highscore::{save_highscores, HighScore, HighScoreRepository, Score},
    AssetPath, GameState,
};

pub(crate) fn display_new_highscore_menu(
    egui_context: Res<EguiContext>,
    score: Res<Score>,
    //textures: Res<TextureAssetMap<GeneralTexture>>,
    mut state: ResMut<State<GameState>>,
    mut name: Local<String>,
    mut highscore_repo: ResMut<HighScoreRepository>,
    mut kb: ResMut<Input<KeyCode>>,
    assets_path: Res<AssetPath>,
) {
    let ctx = egui_context.ctx();

    egui::Window::new("NewHighScore Menu")
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, [0., 0.])
        .show(ctx, |ui| {
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    ui.add(
                        egui::Label::new("New Highscore")
                            .heading()
                            .text_color(egui::Color32::WHITE),
                    );
                    ui.add(egui::Separator::default().horizontal().spacing(20.));

                    ui.add(egui::Label::new(score.to_string()).text_color(egui::Color32::GREEN));
                    // TODO: display trophy!
                    // ui.add(egui::Image::new(
                    //     egui::TextureId::Egui,//User(textures.get(GeneralTexture::Trophy).unwrap().texture.id),
                    //     [100., 100.],
                    // ));

                    ui.add(egui::Separator::default().horizontal().spacing(20.));
                    let name_box =
                        ui.add(egui::TextEdit::singleline(&mut *name).hint_text("Enter your name"));
                    name_box.request_focus();

                    if name.len() >= 3 {
                        ui.add(egui::Separator::default().horizontal().spacing(20.));
                        if ui.button("Enter hall of fame").clicked() || name_box.clicked() {
                            kb.reset(KeyCode::Return);
                            highscore_repo
                                .push(HighScore::new(*score, name.clone()))
                                .expect("failed adding highscore");
                            name.clear();

                            save_highscores(&highscore_repo, &assets_path);

                            state.set(GameState::HighScoreMenu).unwrap();
                        }
                    }
                },
            )
        });

    egui::Window::new("NewHighScore Hint")
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::RIGHT_BOTTOM, [-5., -5.])
        .show(ctx, |ui| {
            ui.add(
                egui::Label::new(if name.len() >= 3 {
                    "Hit Enter accept"
                } else {
                    "At least 3 charaters required"
                })
                .small(),
            );
        });
}
