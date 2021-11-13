use bevy::prelude::*;
use derive_more::{AsMut, AsRef, Display};

use crate::{
    resources::{FontAssetMap, GfxBounds},
    text::{AsTextWithAttr, TextAttr},
    Fonts, GameState,
};

pub struct ScoreBoardPlugin;

#[derive(Component, Debug, Clone, AsRef, AsMut, Display, Reflect)]
#[display(fmt = "Score: {}", score)]
pub struct ScoreBoard {
    #[as_ref]
    score: u32,
}

impl Plugin for ScoreBoardPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ScoreBoard>();

        app.add_system_set(
            SystemSet::on_enter(GameState::InGame).with_system(enter_ingame_scoreboard),
        );
        app.add_system_set(
            SystemSet::on_enter(GameState::GameOver).with_system(enter_gameover_scoreboard),
        );
        app.add_system_set(SystemSet::on_update(GameState::InGame).with_system(update_scoreboard));
    }
}

fn enter_ingame_scoreboard(
    mut commands: Commands,
    query: Query<Entity, With<ScoreBoard>>,
    font_asset_map: Res<FontAssetMap<Fonts>>,
    win_bounds: Res<GfxBounds>,
) {
    // remove any old remnants
    query.iter().for_each(|e| commands.entity(e).despawn());

    // create a fresh scoreboard
    let font = font_asset_map
        .get(&Fonts::ScoreBoard)
        .expect("unable to get font for ScoreBoard");
    let board = ScoreBoard { score: 0 };
    let color = Color::BEIGE;
    let textattr = TextAttr {
        alignment: TextAlignment {
            vertical: VerticalAlign::Top,
            horizontal: HorizontalAlign::Right,
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
                    win_bounds.width() / 2. - 15.,
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

fn enter_gameover_scoreboard(
    mut commands: Commands,
    mut query: Query<(Entity, &ScoreBoard, &mut Transform, &mut TextAttr)>,
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

fn update_scoreboard(
    mut commands: Commands,
    query: Query<(Entity, &ScoreBoard, &TextAttr), Changed<ScoreBoard>>,
) {
    for (e, board, textattr) in query.iter() {
        commands
            .entity(e)
            .insert(board.as_text_with_attr(textattr.clone()));
    }
}
