use bevy::prelude::*;
use derive_more::{AsMut, AsRef, Display};

use crate::{
    assets::LoadRelative,
    text::{AsText, TextAttr},
    Args, GameState,
};

pub(crate) struct ScoreBoardPlugin;

#[derive(Component, Debug, Clone, AsRef, AsMut, Display)]
#[display(fmt = "Score: {}", score)]
pub(crate) struct ScoreBoard {
    #[as_ref]
    score: u32,
}

impl Plugin for ScoreBoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(GameState::InGame).with_system(enter_ingame_scoreboard.system()),
        );
        app.add_system_set(
            SystemSet::on_enter(GameState::GameOver)
                .with_system(enter_gameover_scoreboard.system()),
        );
        app.add_system_set(
            SystemSet::on_update(GameState::InGame).with_system(update_scoreboard.system()),
        );
    }
}

fn enter_ingame_scoreboard(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    args: Res<Args>,
    query: Query<Entity, With<ScoreBoard>>,
) {
    // remove any old remnants
    query.iter().for_each(|e| commands.entity(e).despawn());

    // create a fresh scoreboard
    let font = asset_server
        .load_relative(&"fonts/FiraMono-Medium.ttf", &*args)
        .expect("missing font");
    let board = ScoreBoard { score: 0 };
    let textattr = TextAttr {
        alignment: TextAlignment {
            vertical: VerticalAlign::Center,
            horizontal: HorizontalAlign::Center,
        },
        style: TextStyle {
            font,
            font_size: 48.0,
            color: Color::DARK_GRAY,
        },
    };
    commands
        .spawn_bundle(Text2dBundle {
            text: board.as_text(&textattr),
            transform: Transform {
                translation: Vec3::new(0., 0., 0.),
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
        commands.entity(e).insert(board.as_text(&textattr));
    }
}

fn update_scoreboard(
    mut commands: Commands,
    query: Query<(Entity, &ScoreBoard, &TextAttr), Changed<ScoreBoard>>,
) {
    for (e, board, textattr) in query.iter() {
        commands.entity(e).insert(board.as_text(textattr));
    }
}
