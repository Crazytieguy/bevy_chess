use std::f32::consts::PI;

use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum PieceColor {
    White,
    Black,
}

#[derive(Clone, Copy, PartialEq)]
pub enum PieceType {
    King,
    Queen,
    Bishop,
    Knight,
    Rook,
    Pawn,
}

#[derive(Clone, Copy)]
pub struct Piece {
    pub color: PieceColor,
    pub piece_type: PieceType,
    pub has_moved: bool,
    // Current position
    pub x: u8,
    pub y: u8,
}
impl Piece {
    /// Returns the possible_positions that are available
    pub fn is_move_valid(&self, new_position: (u8, u8), pieces: &[Piece]) -> bool {
        // If there's a piece of the same color in the same square, it can't move
        if color_of_square(new_position, pieces) == Some(self.color) {
            return false;
        }

        match self.piece_type {
            PieceType::King => {
                // Vertical
                ((self.x as i8 - new_position.0 as i8).abs() == 1
                    && (self.y == new_position.1))
                // Horizontal
                || ((self.y as i8 - new_position.1 as i8).abs() == 1
                    && (self.x == new_position.0))
                // Diagonal
                || ((self.x as i8 - new_position.0 as i8).abs() == 1
                    && (self.y as i8 - new_position.1 as i8).abs() == 1)
                // Castling
                || (!self.has_moved
                    && (self.x == new_position.0)
                    // Short
                    && ((new_position.1 == 6
                        && is_path_empty((self.x, self.y), new_position, pieces)
                        && pieces.iter().any(|&piece| {
                            piece.piece_type == PieceType::Rook
                            && !piece.has_moved
                            && piece.color == self.color
                            && piece.y == 7
                        }))
                        // Long
                        || (new_position.1 == 2
                        && is_path_empty((self.x, self.y), (self.x, 1), pieces)
                        && pieces.iter().any(|&piece| {
                            piece.piece_type == PieceType::Rook
                            && !piece.has_moved
                            && piece.color == self.color
                            && piece.y == 0
                        }))))
            }
            PieceType::Queen => {
                is_path_empty((self.x, self.y), new_position, pieces)
                    && ((self.x as i8 - new_position.0 as i8).abs()
                        == (self.y as i8 - new_position.1 as i8).abs()
                        || ((self.x == new_position.0 && self.y != new_position.1)
                            || (self.y == new_position.1 && self.x != new_position.0)))
            }
            PieceType::Bishop => {
                is_path_empty((self.x, self.y), new_position, pieces)
                    && (self.x as i8 - new_position.0 as i8).abs()
                        == (self.y as i8 - new_position.1 as i8).abs()
            }
            PieceType::Knight => {
                ((self.x as i8 - new_position.0 as i8).abs() == 2
                    && (self.y as i8 - new_position.1 as i8).abs() == 1)
                    || ((self.x as i8 - new_position.0 as i8).abs() == 1
                        && (self.y as i8 - new_position.1 as i8).abs() == 2)
            }
            PieceType::Rook => {
                is_path_empty((self.x, self.y), new_position, pieces)
                    && ((self.x == new_position.0 && self.y != new_position.1)
                        || (self.y == new_position.1 && self.x != new_position.0))
            }
            PieceType::Pawn => {
                if self.color == PieceColor::White {
                    // Normal move
                    if new_position.0 as i8 - self.x as i8 == 1
                        && (self.y == new_position.1)
                        && color_of_square(new_position, pieces).is_none()
                    {
                        return true;
                    }

                    // Move 2 squares
                    if self.x == 1
                        && new_position.0 as i8 - self.x as i8 == 2
                        && (self.y == new_position.1)
                        && is_path_empty((self.x, self.y), new_position, pieces)
                        && color_of_square(new_position, pieces).is_none()
                    {
                        return true;
                    }

                    // Take piece
                    if new_position.0 as i8 - self.x as i8 == 1
                        && (self.y as i8 - new_position.1 as i8).abs() == 1
                        && color_of_square(new_position, pieces) == Some(PieceColor::Black)
                    {
                        return true;
                    }
                } else {
                    // Normal move
                    if new_position.0 as i8 - self.x as i8 == -1
                        && (self.y == new_position.1)
                        && color_of_square(new_position, pieces).is_none()
                    {
                        return true;
                    }

                    // Move 2 squares
                    if self.x == 6
                        && new_position.0 as i8 - self.x as i8 == -2
                        && (self.y == new_position.1)
                        && is_path_empty((self.x, self.y), new_position, pieces)
                        && color_of_square(new_position, pieces).is_none()
                    {
                        return true;
                    }

                    // Take piece
                    if new_position.0 as i8 - self.x as i8 == -1
                        && (self.y as i8 - new_position.1 as i8).abs() == 1
                        && color_of_square(new_position, pieces) == Some(PieceColor::White)
                    {
                        return true;
                    }
                }

                false
            }
        }
    }
}

fn is_path_empty(begin: (u8, u8), end: (u8, u8), pieces: &[Piece]) -> bool {
    // Same column
    if begin.0 == end.0 {
        for piece in pieces {
            if piece.x == begin.0
                && ((piece.y > begin.1 && piece.y < end.1)
                    || (piece.y > end.1 && piece.y < begin.1))
            {
                return false;
            }
        }
    }
    // Same row
    if begin.1 == end.1 {
        for piece in pieces {
            if piece.y == begin.1
                && ((piece.x > begin.0 && piece.x < end.0)
                    || (piece.x > end.0 && piece.x < begin.0))
            {
                return false;
            }
        }
    }

    // Diagonals
    let x_diff = (begin.0 as i8 - end.0 as i8).abs();
    let y_diff = (begin.1 as i8 - end.1 as i8).abs();
    if x_diff == y_diff {
        for i in 1..x_diff {
            let pos = if begin.0 < end.0 && begin.1 < end.1 {
                // left bottom - right top
                (begin.0 + i as u8, begin.1 + i as u8)
            } else if begin.0 < end.0 && begin.1 > end.1 {
                // left top - right bottom
                (begin.0 + i as u8, begin.1 - i as u8)
            } else if begin.0 > end.0 && begin.1 < end.1 {
                // right bottom - left top
                (begin.0 - i as u8, begin.1 + i as u8)
            } else {
                // begin.0 > end.0 && begin.1 > end.1
                // right top - left bottom
                (begin.0 - i as u8, begin.1 - i as u8)
            };

            if color_of_square(pos, pieces).is_some() {
                return false;
            }
        }
    }

    true
}

/// Returns None if square is empty, returns a Some with the color if not
fn color_of_square(pos: (u8, u8), pieces: &[Piece]) -> Option<PieceColor> {
    for piece in pieces {
        if piece.x == pos.0 && piece.y == pos.1 {
            return Some(piece.color);
        }
    }
    None
}

const MOVE_TIME: f32 = 0.1;

fn move_pieces(time: Res<Time>, mut query: Query<(&mut Transform, &Piece)>) {
    for (mut transform, piece) in query.iter_mut() {
        // Get the direction to move in
        let direction = Vec3::new(piece.x as f32, 0., piece.y as f32) - transform.translation;

        // Only move if the piece isn't already there (distance is big)
        if direction.length() > 0.03 {
            transform.translation += direction * time.delta_seconds() / MOVE_TIME;
        }
    }
}

fn create_pieces(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Load all the meshes
    let bishop_handle = asset_server.load("models/chess_kit/pieces.glb#Mesh0/Primitive0");
    let king_body_handle = asset_server.load("models/chess_kit/pieces.glb#Mesh1/Primitive0");
    let king_cross_handle = asset_server.load("models/chess_kit/pieces.glb#Mesh2/Primitive0");
    let knight_base_handle = asset_server.load("models/chess_kit/pieces.glb#Mesh3/Primitive0");
    let knight_head_handle = asset_server.load("models/chess_kit/pieces.glb#Mesh4/Primitive0");
    let pawn_handle = asset_server.load("models/chess_kit/pieces.glb#Mesh5/Primitive0");
    let queen_handle = asset_server.load("models/chess_kit/pieces.glb#Mesh6/Primitive0");
    let rook_handle = asset_server.load("models/chess_kit/pieces.glb#Mesh7/Primitive0");

    // Add some materials
    let white_material = materials.add(Color::rgb(1., 0.8, 0.8).into());
    let black_material = materials.add(Color::rgb(0.3, 0.3, 0.3).into());

    let mut handle_positions = vec![
        (vec![rook_handle.clone()], (0, 0), PieceType::Rook),
        (
            vec![knight_base_handle.clone(), knight_head_handle.clone()],
            (0, 1),
            PieceType::Knight,
        ),
        (vec![bishop_handle.clone()], (0, 2), PieceType::Bishop),
        (vec![queen_handle], (0, 3), PieceType::Queen),
        (
            vec![king_body_handle, king_cross_handle],
            (0, 4),
            PieceType::King,
        ),
        (vec![bishop_handle], (0, 5), PieceType::Bishop),
        (
            vec![knight_base_handle, knight_head_handle],
            (0, 6),
            PieceType::Knight,
        ),
        (vec![rook_handle], (0, 7), PieceType::Rook),
    ];
    for i in 0..8 {
        handle_positions.push((vec![pawn_handle.clone()], (1, i), PieceType::Pawn))
    }

    for (meshes, position, piece_type) in handle_positions.iter() {
        spawn_piece(
            &mut commands,
            white_material.clone(),
            PieceColor::White,
            meshes,
            *position,
            *piece_type,
        )
    }

    for (meshes, (x, y), piece_type) in handle_positions.iter() {
        spawn_piece(
            &mut commands,
            black_material.clone(),
            PieceColor::Black,
            meshes,
            (7 - *x, *y),
            *piece_type,
        )
    }
}

fn spawn_piece(
    commands: &mut Commands,
    material: Handle<StandardMaterial>,
    piece_color: PieceColor,
    meshes: &[Handle<Mesh>],
    position: (u8, u8),
    piece_type: PieceType,
) {
    commands
        .spawn_bundle(PbrBundle {
            transform: Transform {
                translation: Vec3::new(position.0 as f32, 0., position.1 as f32),
                scale: Vec3::new(0.2, 0.2, 0.2),
                rotation: Quat::from_rotation_y(match piece_color {
                    PieceColor::Black => PI,
                    PieceColor::White => 0.,
                }),
            },
            ..Default::default()
        })
        .insert(Piece {
            color: piece_color,
            piece_type,
            has_moved: false,
            x: position.0,
            y: position.1,
        })
        .with_children(|parent| {
            for mesh in meshes.iter() {
                parent.spawn_bundle(PbrBundle {
                    mesh: mesh.clone(),
                    material: material.clone(),
                    ..Default::default()
                });
            }
        });
}

pub struct PiecesPlugin;
impl Plugin for PiecesPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(create_pieces.system())
            .add_system(move_pieces.system());
    }
}
