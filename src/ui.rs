use bevy::prelude::*;

use crate::{
    board::{GameStatus, StatusType},
    pieces::PieceColor,
};

// Component to mark the Text entity
struct StatusText;

/// Initialize UiCamera and text
fn init_next_move_text(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let material = color_materials.add(Color::NONE.into());

    commands
        .spawn_bundle(UiCameraBundle::default())
        // root node
        .commands()
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    left: Val::Px(10.),
                    top: Val::Px(10.),
                    ..Default::default()
                },
                ..Default::default()
            },
            material,
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    text: Text::with_section(
                        "Next move: White",
                        TextStyle {
                            font: font.clone(),
                            font_size: 40.0,
                            color: Color::rgb(0.8, 0.8, 0.8),
                        },
                        Default::default(),
                    ),
                    ..Default::default()
                })
                .insert(StatusText);
        });
}

/// Update text with the correct turn
fn update_status(game_status: Res<GameStatus>, mut text_query: Query<&mut Text, With<StatusText>>) {
    if !game_status.is_changed() {
        return;
    }
    let color_text = match game_status.color {
        PieceColor::White => "White",
        PieceColor::Black => "Black",
    };
    let text_value = match game_status.status_type {
        StatusType::Win => format!("{} Wins!", color_text),
        StatusType::Move => format!("Next move: {}", color_text),
    };
    if let Some(mut text) = text_query.iter_mut().next() {
        text.sections[0].value = text_value;
    }
}

/// Demo system to show off Query transformers
fn log_text_changes(query: Query<&Text, Changed<Text>>) {
    for text in query.iter() {
        println!("New text: {}", text.sections[0].value);
    }
}

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(init_next_move_text.system())
            .add_system(update_status.system())
            .add_system(log_text_changes.system());
    }
}
