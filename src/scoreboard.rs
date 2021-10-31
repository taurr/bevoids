use std::path::PathBuf;

use bevy::{prelude::*, reflect::List};
use derive_more::{AsMut, AsRef, Display};

use crate::{Args, GameState};

pub(crate) struct ScoreBoardPlugin;

#[derive(Component, Debug, Clone, AsRef, AsMut, Display)]
#[display(fmt = "Score: {}", score)]
pub(crate) struct ScoreBoard {
    #[as_ref]
    score: u32,
    alignment: TextAlignment,
    style: TextStyle,
}

impl From<&ScoreBoard> for Text {
    fn from(board: &ScoreBoard) -> Self {
        Text::with_section(
            board.to_string(),
            board.style.clone(),
            board.alignment.clone(),
        )
    }
}

impl Plugin for ScoreBoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PostStartup, init_scoreboard.system());
        app.add_system_set(
            SystemSet::on_update(GameState::InGame).with_system(update_scoreboard.system()),
        );
    }
}

fn init_scoreboard(mut commands: Commands, asset_server: Res<AssetServer>, args: Res<Args>) {
    let font = "fonts/FiraSans-Bold.ttf";
    let font = args.assets.as_ref().map(|assets| {
        let mut p = PathBuf::from(assets);
        p.push(font);
        p.display().to_string()
    }).unwrap_or_else(|| font.to_string());

    let board = ScoreBoard {
        score: 0,
        alignment: TextAlignment {
            vertical: VerticalAlign::Center,
            horizontal: HorizontalAlign::Center,
        },
        style: TextStyle {
            font: asset_server.load(font.as_str()),
            font_size: 48.0,
            color: Color::DARK_GRAY,
        },
    };
    commands
        .spawn_bundle(Text2dBundle {
            text: Text::from(&board),
            ..Default::default()
        })
        .insert(board);
}

fn update_scoreboard(
    mut commands: Commands,
    query: Query<(Entity, &ScoreBoard), Changed<ScoreBoard>>,
) {
    for (entity, board) in query.iter() {
        commands.entity(entity).insert(Text::from(board));
    }
}
