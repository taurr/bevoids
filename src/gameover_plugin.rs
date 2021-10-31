use bevy::prelude::*;
use bevy_kira_audio::Audio;
use derive_more::Display;

use crate::{
    assets::LoadRelative,
    text::{AsText, TextAttr},
    Args, GameState,
};

pub(crate) struct GameoverPlugin;

#[derive(Component, Debug, Display)]
#[display(fmt = "Game Over")]
struct GameOver;

#[derive(Component, Debug, Display)]
#[display(fmt = "Press return to try again")]
struct PressReturn;

impl Plugin for GameoverPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(GameState::GameOver).with_system(init_gameover.system()),
        )
        .add_system_set(
            SystemSet::on_update(GameState::GameOver).with_system(restart_on_enter.system()),
        )
        .add_system_set(
            SystemSet::on_exit(GameState::GameOver).with_system(exit_gameover.system()),
        );
    }
}

fn init_gameover(mut commands: Commands, asset_server: Res<AssetServer>, args: Res<Args>) {
    let font = asset_server
        .load_relative(&"fonts/FiraSans-Bold.ttf", &*args)
        .expect("missing font");

    let gameover = GameOver;
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

    let pressreturn = PressReturn;
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

fn exit_gameover(
    mut commands: Commands,
    gameover_query: Query<Entity, With<GameOver>>,
    pressreturn_query: Query<Entity, With<PressReturn>>,
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
