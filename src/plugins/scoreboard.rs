use bevy::prelude::*;
use derive_more::{AsMut, AsRef, Display};

use crate::{
    text::{AsTextWithAttr, TextAttr},
    GameState,
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
    asset_server: Res<AssetServer>,
    query: Query<Entity, With<ScoreBoard>>,
) {
    // remove any old remnants
    query.iter().for_each(|e| commands.entity(e).despawn());

    // create a fresh scoreboard
    let font = asset_server.load("fonts/FiraMono-Medium.ttf");
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
            text: board.as_text_with_attr(textattr.clone()),
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
