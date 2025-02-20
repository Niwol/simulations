use bevy::prelude::*;
use pendulum::{camera::CameraPlugin, pendulum::PendulumPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((CameraPlugin, PendulumPlugin))
        .add_systems(Update, exit_app)
        .run();
}

fn exit_app(input: Res<ButtonInput<KeyCode>>, mut exit_event: EventWriter<AppExit>) {
    if input.just_pressed(KeyCode::Escape) {
        exit_event.send_default();
    }
}
