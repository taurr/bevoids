use bevy::prelude::*;
use derive_more::Display;

use crate::{
    text::{AsText, TextAttr},
    GameState,
};

pub struct GameOverPlugin;

#[derive(Component, Debug, Display)]
#[display(fmt = "Game Over")]
struct GameOverText;

#[derive(Component, Debug, Display)]
#[display(fmt = "Press return to try again")]
struct PressReturnText;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(GameState::GameOver).with_system(init_gameover_texts.system()),
        );

        app.add_system_set(
            SystemSet::on_update(GameState::GameOver).with_system(restart_on_enter.system()),
        );

        app.add_system_set(
            SystemSet::on_exit(GameState::GameOver)
                .with_system(remove_texts_on_exit_gameover.system()),
        );
    }
}

fn init_gameover_texts(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

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
            text: gameover.as_text(&gameover_textattr),
            transform: Transform {
                translation: Vec3::new(0., 75., 900.),
                ..Transform::default()
            },
            ..Default::default()
        })
        .insert(gameover)
        .insert(gameover_textattr);

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
            text: pressreturn.as_text(&pressreturn_textattr),
            transform: Transform {
                translation: Vec3::new(0., -75., 900.),
                ..Transform::default()
            },
            ..Default::default()
        })
        .insert(pressreturn)
        .insert(pressreturn_textattr);
}

fn remove_texts_on_exit_gameover(
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

fn restart_on_enter(kb: Res<Input<KeyCode>>, mut state: ResMut<State<GameState>>) {
    if kb.pressed(KeyCode::Return) {
        state.set(GameState::InGame).unwrap();
    }
}
