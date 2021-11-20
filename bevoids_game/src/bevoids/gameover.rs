use bevy::prelude::*;
use bevy_asset_map::FontAssetMap;
use derive_more::Display;

use super::{GameFont, GameState};
use crate::text::{AsTextWithAttr, TextAttr};

#[derive(Debug, Display)]
#[display(fmt = "Game Over")]
pub(crate) struct GameOverText;

#[derive(Debug, Display)]
#[display(fmt = "Press return to try again")]
pub(crate) struct PressReturnText;

pub(crate) fn init_gameover_texts(
    mut commands: Commands,
    font_asset_map: Res<FontAssetMap<GameFont>>,
) {
    let font = font_asset_map
        .get(&GameFont::GameOver)
        .expect("unable to get font for gmaeover text");

    let gameover = GameOverText;
    let gameover_textattr = TextAttr {
        alignment: TextAlignment {
            vertical: VerticalAlign::Center,
            horizontal: HorizontalAlign::Center,
        },
        style: TextStyle {
            font: font.clone(),
            font_size: 72.0,
            color: Color::WHITE,
        },
    };
    commands
        .spawn_bundle(Text2dBundle {
            text: gameover.as_text_with_attr(gameover_textattr),
            transform: Transform {
                translation: Vec3::new(0., 75., 900.),
                ..Transform::default()
            },
            ..Default::default()
        })
        .insert(gameover);

    let pressreturn = PressReturnText;
    let pressreturn_textattr = TextAttr {
        alignment: TextAlignment {
            vertical: VerticalAlign::Center,
            horizontal: HorizontalAlign::Center,
        },
        style: TextStyle {
            font,
            font_size: 24.0,
            color: Color::WHITE,
        },
    };
    commands
        .spawn_bundle(Text2dBundle {
            text: pressreturn.as_text_with_attr(pressreturn_textattr),
            transform: Transform {
                translation: Vec3::new(0., -75., 900.),
                ..Transform::default()
            },
            ..Default::default()
        })
        .insert(pressreturn);
}

pub(crate) fn remove_gameover_texts(
    mut commands: Commands,
    gameover_query: Query<Entity, With<GameOverText>>,
    pressreturn_query: Query<Entity, With<PressReturnText>>,
) {
    gameover_query
        .iter()
        .for_each(|e| commands.entity(e).despawn());
    pressreturn_query
        .iter()
        .for_each(|e| commands.entity(e).despawn());
}

pub(crate) fn restart_on_enter(kb: Res<Input<KeyCode>>, mut state: ResMut<State<GameState>>) {
    if kb.pressed(KeyCode::Return) {
        state.set(GameState::StartGame).unwrap();
    }
}
