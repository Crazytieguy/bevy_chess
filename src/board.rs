use bevy::prelude::*;
use bevy_mod_picking::{PickableBundle, PickingCamera};

use crate::pieces::{is_check_mate_on, is_check_on, Piece, PieceColor, PieceType};

pub struct Square {
    pub pos: IVec2,
}
impl Square {
    fn is_white(&self) -> bool {
        (self.pos.x + self.pos.y + 1) % 2 == 0
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
        let make_material = |r, g, b| StandardMaterial {
            roughness: 0.5,
            base_color: Color::rgb(r, g, b),
            ..Default::default()
        };
        SquareMaterials {
            highlight_color: materials.add(make_material(0.8, 0.3, 0.3)),
            selected_color: materials.add(make_material(0.9, 0.1, 0.1)),
            black_color: materials.add(make_material(0., 0.1, 0.1)),
            white_color: materials.add(make_material(1., 0.9, 0.9)),
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

pub enum StatusType {
    Move,
    Win,
}

pub struct GameStatus {
    pub color: PieceColor,
    pub status_type: StatusType,
}
impl Default for GameStatus {
    fn default() -> Self {
        Self {
            color: PieceColor::White,
            status_type: StatusType::Move,
        }
    }
}
impl GameStatus {
    fn update(&mut self, pieces: &[Piece]) {
        match is_check_mate_on(pieces, self.color.other()) {
            false => self.color = self.color.other(),
            true => self.status_type = StatusType::Win,
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
    game_status: Res<GameStatus>,
    squares_query: Query<&Square>,
    pieces_query: Query<(Entity, &Piece)>,
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

    if selected_piece.entity.is_none() {
        // Select the piece in the currently selected square
        for (piece_entity, piece) in pieces_query.iter() {
            if piece.pos == square.pos && piece.color == game_status.color {
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
    mut turn: ResMut<GameStatus>,
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

    let (_, mut piece) = match pieces_query.get_mut(selected_piece_entity) {
        Ok(v) => v,
        _ => return,
    };

    reset_selected_event.send(ResetSelectedEvent);

    let piece_color = piece.color;
    if !piece.is_move_valid(square.pos, &pieces_before_move) {
        return;
    }

    let pieces_after_move: Vec<_> = piece.get_pieces_after_move(square.pos, &pieces_before_move);
    if is_check_on(&pieces_after_move, piece_color) {
        return;
    }
    let should_castle =
        piece.piece_type == PieceType::King && (square.pos.y - piece.pos.y).abs() == 2;

    // Move piece
    piece.pos = square.pos;
    piece.has_moved = true;
    turn.update(&pieces_after_move);

    // Check if a piece of the opposite color exists in this square and despawn it
    if let Some((entity, _)) = pieces_query
        .iter_mut()
        .find(|(_, other)| other.pos == square.pos && other.color != piece_color)
    {
        commands.entity(entity).despawn_recursive();
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

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<SelectedSquare>()
            .init_resource::<SelectedPiece>()
            .init_resource::<SquareMaterials>()
            .init_resource::<GameStatus>()
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
            .add_system(reset_selected.system().after("select_square"));
    }
}
