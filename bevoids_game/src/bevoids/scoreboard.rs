use bevy::{log, prelude::*};
use bevy_asset_map::{FontAssetMap, GfxBounds};
use derive_more::{AsMut, AsRef, Display};

use super::GameFont;
use crate::text::{AsTextWithAttr, TextAttr};

#[derive(Debug)]
pub(crate) struct AddScoreEvent(pub u32);

#[derive(Debug, Clone, AsRef, AsMut, Display)]
#[display(fmt = "Score: {}", score)]
pub(crate) struct ScoreBoardComponent {
    #[as_ref]
    score: u32,
}

pub(crate) fn setup_ingame_scoreboard(
    mut commands: Commands,
    query: Query<Entity, With<ScoreBoardComponent>>,
    font_asset_map: Res<FontAssetMap<GameFont>>,
    win_bounds: Res<GfxBounds>,
) {
    // remove any old remnants
    query.iter().for_each(|e| commands.entity(e).despawn());

    // create a fresh scoreboard
    let board = ScoreBoardComponent { score: 0 };

    let font = font_asset_map
        .get(&GameFont::ScoreBoard)
        .expect("unable to get font for ScoreBoard");
    let color = Color::BEIGE;
    let textattr = TextAttr {
        alignment: TextAlignment {
            horizontal: HorizontalAlign::Left,
            vertical: VerticalAlign::Bottom,
        },
        style: TextStyle {
            font,
            font_size: 24.0,
            color,
        },
    };
    commands
        .spawn_bundle(Text2dBundle {
            text: board.as_text_with_attr(textattr.clone()),
            transform: Transform {
                translation: Vec3::new(
                    win_bounds.width() / 2. - 20.,
                    win_bounds.height() / 2. - 10.,
                    1.,
                ),
                ..Transform::default()
            },
            ..Default::default()
        })
        .insert(board)
        .insert(textattr);
}

pub(crate) fn setup_gameover_scoreboard(
    mut commands: Commands,
    mut query: Query<(Entity, &ScoreBoardComponent, &mut Transform, &mut TextAttr)>,
) {
    for (e, board, mut tf, mut textattr) in query.iter_mut() {
        tf.translation = Vec3::new(0., 0., 800.);
        textattr.style.color = Color::WHITE;
        textattr.style.font_size = 48.;
        textattr.alignment = TextAlignment {
            horizontal: HorizontalAlign::Center,
            vertical: VerticalAlign::Center,
        };
        commands
            .entity(e)
            .insert(board.as_text_with_attr(textattr.clone()));
    }
}

pub(crate) fn update_scoreboard(
    mut addscore_events: EventReader<AddScoreEvent>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut ScoreBoardComponent, &TextAttr)>,
) {
    let (scoreboard_entity, mut scoreboard, textattr) =
        query.iter_mut().next().expect("couldn't get query result");

    let score: u32 = addscore_events.iter().map(|e| e.0).sum();
    if score > 0 {
        scoreboard.score += score;
        log::info!(score, total = scoreboard.score, "update score");
        commands
            .entity(scoreboard_entity)
            .insert(scoreboard.as_text_with_attr(textattr.clone()));
    }
}
