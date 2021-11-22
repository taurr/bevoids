use bevy::{log, prelude::*};
use bevy_egui::{egui, EguiContext};
use derive_more::{AsMut, AsRef, Display};

#[derive(Debug)]
pub(crate) struct AddScoreEvent(pub u32);

#[derive(Debug, Clone, AsRef, AsMut, Display)]
#[display(fmt = "Score: {}", score)]
pub(crate) struct ScoreBoardComponent {
    #[as_ref]
    score: u32,
}

pub(crate) fn display_playing_scoreboard(
    egui_context: Res<EguiContext>,
    query: Query<&ScoreBoardComponent>,
) {
    let ctx = egui_context.ctx();

    let scoreboard = query.iter().next().unwrap();
    let score = scoreboard.to_string();

    egui::Window::new("Menu")
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::RIGHT_TOP, [-20., 10.])
        .show(ctx, |ui| {
            ui.add(egui::Label::new(score).text_color(egui::Color32::WHITE));
        });

    egui::Window::new("Note")
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::RIGHT_BOTTOM, [-5., -5.])
        .show(ctx, |ui| {
            ui.add(egui::Label::new("Hit Escape to pause").small());
        });
}

pub(crate) fn reset_scoreboard(
    mut commands: Commands,
    query: Query<Entity, With<ScoreBoardComponent>>,
) {
    // remove any old remnants
    query.iter().for_each(|e| commands.entity(e).despawn());

    // create a fresh scoreboard
    let board = ScoreBoardComponent { score: 0 };
    commands.spawn().insert(board);
}

pub(crate) fn update_scoreboard(
    mut addscore_events: EventReader<AddScoreEvent>,
    mut query: Query<&mut ScoreBoardComponent>,
) {
    let score: u32 = addscore_events.iter().map(|e| e.0).sum();
    if score > 0 {
        let mut scoreboard = query.iter_mut().next().expect("couldn't get query result");
        scoreboard.score += score;
        log::info!(score, total = scoreboard.score, "update score");
    }
}
