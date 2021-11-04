use std::f32::consts::PI;

use self::PieceType::*;
use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum PieceColor {
    White,
    Black,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PieceType {
    King,
    Queen,
    Bishop,
    Knight,
    Rook,
    Pawn,
}

const LAYOUT: [PieceType; 8] = [Rook, Knight, Bishop, Queen, King, Bishop, Knight, Rook];

#[derive(Clone, Copy)]
pub struct Piece {
    pub color: PieceColor,
    pub piece_type: PieceType,
    pub has_moved: bool,
    pub pos: IVec2,
}
impl Piece {
    /// Returns the possible_positions that are available
    pub fn is_move_valid(&self, target: IVec2, pieces: &[Piece]) -> bool {
        // If there's a piece of the same color in the same square, it can't move
        let color_of_target = color_of_square(target, pieces);
        if color_of_target == Some(self.color) {
            return false;
        }
        let diff = target - self.pos;
        let signum = diff.signum();

        match self.piece_type {
            King => {
                diff.abs() <= IVec2::ONE
                // Castling
                || !self.has_moved
                    && diff.x == 0
                    && diff.y.abs() == 2
                    && is_path_empty(self.pos, target + signum, pieces)
                    && pieces.iter().any(|&piece| {
                        piece.piece_type == Rook
                        && !piece.has_moved
                        && (piece.pos - self.pos).signum() == signum
                    })
            }
            Queen => {
                diff == signum * diff.abs().max_element() && is_path_empty(self.pos, target, pieces)
            }
            Bishop => diff.x.abs() == diff.y.abs() && is_path_empty(self.pos, target, pieces),
            Knight => diff.abs() == IVec2::new(1, 2) || diff.abs() == IVec2::new(2, 1),
            Rook => signum.x * signum.y == 0 && is_path_empty(self.pos, target, pieces),
            Pawn => {
                let direction = match self.color {
                    PieceColor::White => 1,
                    PieceColor::Black => -1,
                };
                // Normal move
                diff.x == direction
                    && diff.y == 0
                    && color_of_target.is_none()

                    // Double move
                    || !self.has_moved
                    && diff.x == direction * 2
                    && diff.y == 0
                    && color_of_target.is_none()
                    && is_path_empty(self.pos, target, pieces)

                    // Take piece
                    || diff.x == direction
                        && diff.y.abs() == 1
                        && color_of_target.is_some()
            }
        }
    }
}

fn is_path_empty(begin: IVec2, end: IVec2, pieces: &[Piece]) -> bool {
    let diff = end - begin;
    let signum = diff.signum();
    if diff == signum {
        return true;
    };
    let min = (begin + signum).min(end - signum);
    let max = (begin + signum).max(end - signum);

    for piece in pieces {
        if min <= piece.pos && piece.pos <= max && {
            let diff = piece.pos - begin;
            diff == signum * diff.abs().max_element()
        } {
            return false;
        }
    }
    true
}

/// Returns None if square is empty, returns a Some with the color if not
fn color_of_square(pos: IVec2, pieces: &[Piece]) -> Option<PieceColor> {
    pieces
        .iter()
        .find(|piece| piece.pos == pos)
        .map(|piece| piece.color)
}

const MOVE_TIME: f32 = 0.1;

fn move_pieces(time: Res<Time>, mut query: Query<(&mut Transform, &Piece)>) {
    for (mut transform, piece) in query.iter_mut() {
        // Get the direction to move in
        let direction =
            Vec3::new(piece.pos.x as f32, 0., piece.pos.y as f32) - transform.translation;

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

    let mut spawn_piece = |piece_color, x, y, piece_type| {
        let meshes = match piece_type {
            Rook => vec![rook_handle.clone()],
            Knight => vec![knight_base_handle.clone(), knight_head_handle.clone()],
            Bishop => vec![bishop_handle.clone()],
            Queen => vec![queen_handle.clone()],
            King => vec![king_body_handle.clone(), king_cross_handle.clone()],
            Pawn => vec![pawn_handle.clone()],
        };
        commands
            .spawn_bundle(PbrBundle {
                transform: Transform {
                    translation: Vec3::new(x as f32, 0., y as f32),
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
                pos: (x, y).into(),
            })
            .with_children(|parent| {
                for mesh in meshes {
                    parent.spawn_bundle(PbrBundle {
                        mesh: mesh.clone(),
                        material: match piece_color {
                            PieceColor::White => white_material.clone(),
                            PieceColor::Black => black_material.clone(),
                        },
                        ..Default::default()
                    });
                }
            });
    };

    for (&piece_type, y) in LAYOUT.iter().zip(0..) {
        spawn_piece(PieceColor::White, 0, y, piece_type);
        spawn_piece(PieceColor::White, 1, y, Pawn);
        spawn_piece(PieceColor::Black, 7, y, piece_type);
        spawn_piece(PieceColor::Black, 6, y, Pawn);
    }
}

pub struct PiecesPlugin;
impl Plugin for PiecesPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(create_pieces.system())
            .add_system(move_pieces.system());
    }
}
