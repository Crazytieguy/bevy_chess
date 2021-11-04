use crate::pieces::*;
use bevy::{app::AppExit, prelude::*};
use bevy_mod_picking::*;

pub struct Square {
    pub pos: IVec2,
}
impl Square {
    fn is_white(&self) -> bool {
        (self.pos.x + self.pos.y + 1) % 2 == 0
    }
}

struct KingPositions {
    white: IVec2,
    black: IVec2,
}
impl Default for KingPositions {
    fn default() -> Self {
        Self {
            white: IVec2::new(0, 5),
            black: IVec2::new(7, 5),
        }
    }
}

fn create_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<SquareMaterials>,
) {
    // Add meshes
    let mesh = meshes.add(Mesh::from(shape::Plane { size: 1. }));

    // Spawn 64 squares
    for i in 0..8 {
        for j in 0..8 {
            commands
                .spawn_bundle(PbrBundle {
                    mesh: mesh.clone(),
                    // Change material according to position to get alternating pattern
                    material: if (i + j + 1) % 2 == 0 {
                        materials.white_color.clone()
                    } else {
                        materials.black_color.clone()
                    },
                    transform: Transform::from_translation(Vec3::new(i as f32, 0., j as f32)),
                    ..Default::default()
                })
                .insert_bundle(PickableBundle::default())
                .insert(Square { pos: (i, j).into() });
        }
    }
}

fn color_squares(
    selected_square: Res<SelectedSquare>,
    materials: Res<SquareMaterials>,
    mut query: Query<(Entity, &Square, &mut Handle<StandardMaterial>)>,
    picking_camera_query: Query<&PickingCamera>,
) {
    // Get entity under the cursor, if there is one
    let top_entity = match picking_camera_query.iter().last() {
        Some(picking_camera) => picking_camera
            .intersect_top()
            .map(|(entity, _intersection)| entity),
        None => None,
    };

    for (entity, square, mut material) in query.iter_mut() {
        // Change the material
        *material = if Some(entity) == top_entity {
            materials.highlight_color.clone()
        } else if Some(entity) == selected_square.entity {
            materials.selected_color.clone()
        } else if square.is_white() {
            materials.white_color.clone()
        } else {
            materials.black_color.clone()
        };
    }
}

struct SquareMaterials {
    highlight_color: Handle<StandardMaterial>,
    selected_color: Handle<StandardMaterial>,
    black_color: Handle<StandardMaterial>,
    white_color: Handle<StandardMaterial>,
}

impl FromWorld for SquareMaterials {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let mut materials = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();
        SquareMaterials {
            highlight_color: materials.add(Color::rgb(0.8, 0.3, 0.3).into()),
            selected_color: materials.add(Color::rgb(0.9, 0.1, 0.1).into()),
            black_color: materials.add(Color::rgb(0., 0.1, 0.1).into()),
            white_color: materials.add(Color::rgb(1., 0.9, 0.9).into()),
        }
    }
}

#[derive(Default)]
struct SelectedSquare {
    entity: Option<Entity>,
}
#[derive(Default)]
struct SelectedPiece {
    entity: Option<Entity>,
}
pub struct PlayerTurn(pub PieceColor);
impl Default for PlayerTurn {
    fn default() -> Self {
        Self(PieceColor::White)
    }
}
impl PlayerTurn {
    fn change(&mut self) {
        self.0 = match self.0 {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
        }
    }
}

fn select_square(
    mouse_button_inputs: Res<Input<MouseButton>>,
    mut selected_square: ResMut<SelectedSquare>,
    mut selected_piece: ResMut<SelectedPiece>,
    squares_query: Query<&Square>,
    picking_camera_query: Query<&PickingCamera>,
) {
    // Only run if the left button is pressed
    if !mouse_button_inputs.just_pressed(MouseButton::Left) {
        return;
    }

    // Get the square under the cursor and set it as the selected
    if let Some(picking_camera) = picking_camera_query.iter().last() {
        if let Some((square_entity, _intersection)) = picking_camera.intersect_top() {
            if let Ok(_square) = squares_query.get(square_entity) {
                // Mark it as selected
                selected_square.entity = Some(square_entity);
            }
        } else {
            // Player clicked outside the board, deselect everything
            selected_square.entity = None;
            selected_piece.entity = None;
        }
    }
}

fn select_piece(
    selected_square: Res<SelectedSquare>,
    mut selected_piece: ResMut<SelectedPiece>,
    turn: Res<PlayerTurn>,
    squares_query: Query<&Square>,
    pieces_query: Query<(Entity, &Piece)>,
) {
    if !selected_square.is_changed() {
        return;
    }

    let square_entity = if let Some(entity) = selected_square.entity {
        entity
    } else {
        return;
    };

    let square = if let Ok(square) = squares_query.get(square_entity) {
        square
    } else {
        return;
    };

    if selected_piece.entity.is_none() {
        // Select the piece in the currently selected square
        for (piece_entity, piece) in pieces_query.iter() {
            if piece.pos == square.pos && piece.color == turn.0 {
                // piece_entity is now the entity in the same square
                selected_piece.entity = Some(piece_entity);
                break;
            }
        }
    }
}

fn move_piece(
    mut commands: Commands,
    selected_square: Res<SelectedSquare>,
    selected_piece: Res<SelectedPiece>,
    mut king_positions: ResMut<KingPositions>,
    mut turn: ResMut<PlayerTurn>,
    squares_query: Query<&Square>,
    mut pieces_query: Query<(Entity, &mut Piece)>,
    mut reset_selected_event: EventWriter<ResetSelectedEvent>,
) {
    if !selected_square.is_changed() {
        return;
    }

    let square_entity = match selected_square.entity {
        Some(v) => v,
        _ => return,
    };

    let square = match squares_query.get(square_entity) {
        Ok(v) => v,
        _ => return,
    };

    let selected_piece_entity = match selected_piece.entity {
        Some(v) => v,
        _ => return,
    };

    let pieces_before_move: Vec<_> = pieces_query.iter_mut().map(|(_, piece)| *piece).collect();

    let mut piece = match pieces_query.get_mut(selected_piece_entity) {
        Ok((_, piece)) => piece,
        _ => return,
    };

    reset_selected_event.send(ResetSelectedEvent);

    let piece_color = piece.color;
    if !piece.is_move_valid(square.pos, &pieces_before_move) {
        return;
    }

    // Check for resulting check
    let ally_king_position = if piece.piece_type == PieceType::King {
        square.pos
    } else {
        match piece_color {
            PieceColor::White => king_positions.white,
            PieceColor::Black => king_positions.black,
        }
    };
    let pieces_after_move: Vec<_> = pieces_before_move
        .iter()
        .map(|&p| {
            if p.pos == piece.pos {
                Piece {
                    pos: square.pos,
                    ..p
                }
            } else {
                p
            }
        })
        .collect();
    for attacker in pieces_before_move {
        if attacker.color != piece_color
            && attacker.is_move_valid(ally_king_position, &pieces_after_move)
        {
            return;
        }
    }
    let should_castle =
        piece.piece_type == PieceType::King && (square.pos.y - piece.pos.y).abs() == 2;

    // Move piece
    piece.pos = square.pos;
    piece.has_moved = true;
    if piece.piece_type == PieceType::King {
        match piece_color {
            PieceColor::White => king_positions.white = square.pos,
            PieceColor::Black => king_positions.black = square.pos,
        }
    }
    turn.change();

    // Check if a piece of the opposite color exists in this square and despawn it
    if let Some((entity, _)) = pieces_query
        .iter_mut()
        .find(|(_, other)| other.pos == square.pos && other.color != piece_color)
    {
        commands.entity(entity).insert(Taken);
    };
    // Castle
    if should_castle {
        let (rook_entity, _) = pieces_query
            .iter_mut()
            .find(|(_, candidate)| {
                candidate.piece_type == PieceType::Rook
                    && candidate.pos.y == (if square.pos.y == 6 { 7 } else { 0 })
                    && candidate.color == piece_color
            })
            .unwrap();
        let (_, mut rook) = pieces_query.get_mut(rook_entity).unwrap();
        rook.pos.y = if square.pos.y == 6 { 5 } else { 3 };
        rook.has_moved = true;
    }
}

struct ResetSelectedEvent;

fn reset_selected(
    mut event_reader: EventReader<ResetSelectedEvent>,
    mut selected_square: ResMut<SelectedSquare>,
    mut selected_piece: ResMut<SelectedPiece>,
) {
    for _event in event_reader.iter() {
        selected_square.entity = None;
        selected_piece.entity = None;
    }
}

struct Taken;
fn despawn_taken_pieces(
    mut commands: Commands,
    mut app_exit_events: EventWriter<AppExit>,
    query: Query<(Entity, &Piece, &Taken)>,
) {
    for (entity, piece, _taken) in query.iter() {
        // If the king is taken, we should exit
        if piece.piece_type == PieceType::King {
            println!(
                "{} won! Thanks for playing!",
                match piece.color {
                    PieceColor::White => "Black",
                    PieceColor::Black => "White",
                }
            );
            app_exit_events.send(AppExit);
        }

        // Despawn piece and children
        commands.entity(entity).despawn_recursive();
    }
}

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<SelectedSquare>()
            .init_resource::<SelectedPiece>()
            .init_resource::<SquareMaterials>()
            .init_resource::<PlayerTurn>()
            .init_resource::<KingPositions>()
            .add_event::<ResetSelectedEvent>()
            .add_startup_system(create_board.system())
            .add_system(color_squares.system())
            .add_system(select_square.system().label("select_square"))
            .add_system(
                // move_piece needs to run before select_piece
                move_piece
                    .system()
                    .after("select_square")
                    .before("select_piece"),
            )
            .add_system(
                select_piece
                    .system()
                    .after("select_square")
                    .label("select_piece"),
            )
            .add_system(despawn_taken_pieces.system())
            .add_system(reset_selected.system().after("select_square"));
    }
}
