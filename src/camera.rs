use std::f32::consts::PI;

use bevy::{prelude::*, render::camera::PerspectiveProjection};
use bevy_mod_picking::PickingCameraBundle;

use crate::{board::GameStatus, pieces::PieceColor};

const ACCELERATION: f32 = PI * 2.;
const INITIAL_SPEED: f32 = 0.5;
struct CameraPosition {
    yaw: f32,
}

impl Default for CameraPosition {
    fn default() -> Self {
        CameraPosition { yaw: -PI / 2. }
    }
}

impl CameraPosition {
    fn get_rotation(&self) -> Quat {
        Quat::from_rotation_ypr(self.yaw, -1.3, 0.)
    }

    fn get_translation(&self) -> Vec3 {
        Vec3::new(
            3.5 * (self.yaw.sin() + 1.),
            13.,
            3.5 * (self.yaw.cos() + 1.),
        )
    }

    fn get_speed(&self) -> f32 {
        let theta = PI / 2. - self.yaw.abs();
        f32::sqrt(2. * ACCELERATION * theta) + INITIAL_SPEED
    }
}

fn setup(mut commands: Commands, camera_yaw: Res<CameraPosition>) {
    commands
        // Camera
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_matrix(Mat4::from_rotation_translation(
                camera_yaw.get_rotation(),
                camera_yaw.get_translation(),
            )),
            ..Default::default()
        })
        .insert_bundle(PickingCameraBundle::default())
        // Light
        .commands()
        .spawn_bundle(LightBundle {
            transform: Transform::from_translation(Vec3::new(3.5, 10., 3.5)),
            ..Default::default()
        });
}

fn reposition_camera(
    time: Res<Time>,
    game_status: Res<GameStatus>,
    mut camera_position: ResMut<CameraPosition>,
    mut camera_query: Query<&mut Transform, With<PerspectiveProjection>>,
) {
    let target_yaw = match game_status.color {
        PieceColor::White => -PI / 2.,
        PieceColor::Black => PI / 2.,
    };
    let remaining = target_yaw - camera_position.yaw;
    if remaining.abs() <= f32::EPSILON {
        return;
    }
    let delta = camera_position.get_speed() * remaining.signum() * time.delta_seconds();
    camera_position.yaw += if remaining.abs() > delta.abs() {
        delta
    } else {
        remaining
    };
    if let Some(mut transform) = camera_query.iter_mut().next() {
        transform.rotation = camera_position.get_rotation();
        transform.translation = camera_position.get_translation();
    }
}

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<CameraPosition>()
            .add_startup_system(setup.system())
            .add_system(reposition_camera.system());
    }
}
