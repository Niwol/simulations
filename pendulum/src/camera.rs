use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, controll_camera);
    }
}

fn setup(mut commands: Commands) {
    // Camrea
    commands.spawn(Camera2dBundle::default());
}

const CAMERA_SPEED: f32 = 700.0;

fn controll_camera(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut weel_scroll: EventReader<MouseWheel>,
    mut camera_grabed: Local<bool>,
    mut camera: Query<(&mut Transform, &mut OrthographicProjection), With<Camera2d>>,
) {
    let Ok((mut camera, mut projection)) = camera.get_single_mut() else {
        return;
    };

    if mouse_input.just_pressed(MouseButton::Middle) {
        *camera_grabed = true;
    }
    if mouse_input.just_released(MouseButton::Middle) {
        *camera_grabed = false;
    }

    if *camera_grabed {
        for mouse_motion in mouse_motion.read() {
            let Vec3 { x, y, z } = camera.translation;
            camera.translation = Vec3 {
                x: x - mouse_motion.delta.x * projection.scale,
                y: y + mouse_motion.delta.y * projection.scale,
                z,
            };
        }
    }

    for mouse_weel in weel_scroll.read() {
        let y = mouse_weel.y;
        projection.scale *= (-y * 0.5) + 1.0;
    }

    let mut pan_dir = Vec2::ZERO;
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        pan_dir.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        pan_dir.y -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        pan_dir.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        pan_dir.x += 1.0;
    }

    if pan_dir == Vec2::ZERO {
        return;
    }

    pan_dir = pan_dir.normalize();

    camera.translation += Vec3::from((pan_dir * CAMERA_SPEED * time.delta_seconds(), 0.0));
}
