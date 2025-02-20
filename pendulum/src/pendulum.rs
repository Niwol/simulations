use bevy::{color::palettes, prelude::*, sprite::Anchor};

pub struct PendulumPlugin;

impl Plugin for PendulumPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpanwPendulumEvent>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    handle_input,
                    spawn_pendulum.run_if(on_event::<SpanwPendulumEvent>()),
                ),
            );
    }
}

#[derive(Event)]
struct SpanwPendulumEvent {
    parent: Entity,
}

#[derive(Resource)]
struct Handles {
    circle_mesh: Handle<Mesh>,
    materials: Vec<Handle<ColorMaterial>>,
}

#[derive(Resource)]
struct RootPendulum {
    root: Entity,
    children: Vec<Pendulum>,
}

struct Pendulum {
    root: Entity,
    children: Vec<Self>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let circle_mesh = meshes.add(Circle::new(50.0));
    let materials: Vec<Handle<ColorMaterial>> = (0..360)
        .map(|i| materials.add(ColorMaterial::from_color(Color::hsv(i as f32, 1.0, 1.0))))
        .collect();

    let handles = Handles {
        circle_mesh,
        materials,
    };

    let root = commands
        .spawn(ColorMesh2dBundle {
            mesh: handles.circle_mesh.clone().into(),
            material: handles.materials[0].clone(),
            ..Default::default()
        })
        .id();

    commands.insert_resource(handles);
    commands.insert_resource(RootPendulum {
        root,
        children: Vec::new(),
    });
}

fn handle_input(
    input: Res<ButtonInput<KeyCode>>,
    mut spawn_event: EventWriter<SpanwPendulumEvent>,
    root_pendulum: Res<RootPendulum>,
) {
    if input.just_pressed(KeyCode::Space) {
        spawn_event.send(SpanwPendulumEvent {
            parent: root_pendulum.root,
        });
    }
}

fn spawn_pendulum(mut commands: Commands, mut spawn_events: EventReader<SpanwPendulumEvent>) {
    for spawn_event in spawn_events.read() {
        let pendulum = commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: palettes::basic::AQUA.into(),
                    custom_size: Some(Vec2 { x: 400.0, y: 3.0 }),
                    anchor: Anchor::CenterLeft,
                    ..Default::default()
                },
                transform: Transform::from_xyz(0.0, 0.0, -1.0),
                ..Default::default()
            })
            .id();

        commands.entity(spawn_event.parent).add_child(pendulum);
    }
}
