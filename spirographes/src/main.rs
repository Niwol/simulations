use std::f32::consts::PI;

use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
    sprite::Mesh2dHandle,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_event::<ResetEvent>()
        .add_systems(Startup, setup)
        .add_systems(Update, exit_app)
        .add_systems(
            Update,
            (
                reset.run_if(on_event::<ResetEvent>()),
                handle_input,
                handle_spinning,
                update_spirographe_mesh,
                toggle_circle_visibility,
            ),
        )
        .add_systems(Last, update_transforms)
        .run();
}

fn exit_app(input: Res<ButtonInput<KeyCode>>, mut exit_event: EventWriter<AppExit>) {
    if input.just_pressed(KeyCode::Escape) {
        exit_event.send_default();
    }
}

const MAIN_CIRCLE_RADIUS: f32 = 500.0;
const SPINNING_CIRCLE_SPEED: f32 = 1.0;
const HUE_CHANGIN_SPEED: f32 = 10.0;

#[derive(Resource)]
struct Spirographe {
    spinning_circle_radius: f32,
    spinning_circle_angle: f32,
    spinning_circle_pos: Vec2,
    pencil_dist: f32,
    pencil_angle: f32,
    pencil_position: Vec2,
    hue: f32,

    points: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,

    needs_update: bool,
}

impl Spirographe {
    fn new() -> Self {
        let mut spirographe = Self {
            spinning_circle_radius: 50.0,
            spinning_circle_angle: 0.0,
            spinning_circle_pos: Vec2::default(),
            pencil_dist: 25.0,
            pencil_angle: 0.0,
            pencil_position: Vec2::default(),
            hue: 0.0,

            points: Vec::new(),
            colors: Vec::new(),
            indices: Vec::new(),

            needs_update: false,
        };

        spirographe.update_positions();

        spirographe
    }

    fn update_positions(&mut self) {
        let dist = MAIN_CIRCLE_RADIUS - self.spinning_circle_radius;
        let spinning_circle_pos = Vec2 {
            x: f32::cos(self.spinning_circle_angle) * dist,
            y: f32::sin(self.spinning_circle_angle) * dist,
        };

        let pencil_pos = Vec2 {
            x: spinning_circle_pos.x + f32::cos(self.pencil_angle) * self.pencil_dist,
            y: spinning_circle_pos.y + f32::sin(self.pencil_angle) * self.pencil_dist,
        };

        self.spinning_circle_pos = spinning_circle_pos;
        self.pencil_position = pencil_pos;

        self.points.push([pencil_pos.x, pencil_pos.y, 0.0]);

        let next_color = Color::hsv(self.hue, 1.0, 1.0).to_srgba();

        let next_color = [next_color.red, next_color.green, next_color.blue, 1.0];

        self.colors.push(next_color);

        let new_index = self.indices.len() as u32;
        self.indices.push(new_index);

        self.needs_update = true;
    }
}

#[derive(Component)]
struct MainCircle;

#[derive(Component)]
struct SpinningCircle;

#[derive(Component)]
struct Pencil;

#[derive(Component)]
struct SpirographeMesh;

#[derive(Event, Default)]
struct ResetEvent;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Main circle
    commands
        .spawn(ColorMesh2dBundle {
            mesh: meshes.add(create_circle_mesh()).into(),
            material: materials.add(Color::srgb(1.0, 1.0, 1.0)),
            transform: Transform::from_scale(Vec3::ONE * MAIN_CIRCLE_RADIUS),
            ..Default::default()
        })
        .insert(MainCircle);

    // Spinning circle
    commands
        .spawn(ColorMesh2dBundle {
            mesh: meshes.add(create_circle_mesh()).into(),
            material: materials.add(Color::srgb(1.0, 1.0, 1.0)),
            ..Default::default()
        })
        .insert(SpinningCircle);

    // Pencil
    commands
        .spawn(ColorMesh2dBundle {
            mesh: meshes.add(Circle { radius: 5.0 }).into(),
            material: materials.add(Color::srgb(1.0, 1.0, 1.0)),
            ..Default::default()
        })
        .insert(Pencil);

    let spirographe = Spirographe::new();

    // Spirographe mesh
    let spirographe_mesh = create_spirographe_mesh();
    commands
        .spawn(ColorMesh2dBundle {
            mesh: meshes.add(spirographe_mesh).into(),
            material: materials.add(Color::srgb(1.0, 1.0, 1.0)),
            ..Default::default()
        })
        .insert(SpirographeMesh);

    commands.insert_resource(spirographe);
}

fn reset(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    spirographe_mesh: Query<&Mesh2dHandle, With<SpirographeMesh>>,
) {
    let new_spirographe = Spirographe::new();

    let handle = spirographe_mesh.single();
    let mesh = meshes.get_mut(&handle.0).unwrap();
    *mesh = create_spirographe_mesh();

    commands.insert_resource(new_spirographe);
}

fn create_circle_mesh() -> Mesh {
    let nb_vertices = 100;

    let v_pos: Vec<[f32; 3]> = (0..nb_vertices)
        .map(|i| {
            let step = (i as f32 / (nb_vertices - 1) as f32) * 2.0 * PI;
            let x = f32::cos(step);
            let y = f32::sin(step);

            [x, y, 0.0]
        })
        .collect();

    let mut indices: Vec<u32> = (0..(nb_vertices - 1)).collect();
    indices.push(0);

    Mesh::new(
        PrimitiveTopology::LineStrip,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, v_pos)
    .with_inserted_indices(Indices::U32(indices))
}

fn create_spirographe_mesh() -> Mesh {
    Mesh::new(
        PrimitiveTopology::LineStrip,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
}

fn handle_input(
    input: Res<ButtonInput<KeyCode>>,
    mut spirographe: ResMut<Spirographe>,
    mut reset_event: EventWriter<ResetEvent>,
) {
    if input.just_pressed(KeyCode::ArrowUp) {
        spirographe.spinning_circle_radius += 10.0;
        spirographe.update_positions();
    }

    if input.just_pressed(KeyCode::ArrowDown) {
        spirographe.spinning_circle_radius -= 10.0;
        spirographe.update_positions();
    }

    if input.just_pressed(KeyCode::ArrowLeft) {
        spirographe.pencil_dist -= 10.0;
        spirographe.update_positions();
    }

    if input.just_pressed(KeyCode::ArrowRight) {
        spirographe.pencil_dist += 10.0;
        spirographe.update_positions();
    }

    if input.just_pressed(KeyCode::KeyR) {
        reset_event.send_default();
    }
}

fn handle_spinning(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut spirographe: ResMut<Spirographe>,
) {
    if !input.pressed(KeyCode::Space) {
        return;
    }

    let dt = time.delta().as_secs_f32();
    spirographe.spinning_circle_angle -= SPINNING_CIRCLE_SPEED * dt;

    if spirographe.spinning_circle_angle <= 0.0 {
        spirographe.spinning_circle_angle = 2.0 * PI;
    }

    spirographe.pencil_angle +=
        (MAIN_CIRCLE_RADIUS / spirographe.spinning_circle_radius) * SPINNING_CIRCLE_SPEED * dt;

    if spirographe.pencil_angle <= 0.0 {
        spirographe.pencil_angle = 2.0 * PI;
    }

    spirographe.hue += HUE_CHANGIN_SPEED * dt;
    if spirographe.hue > 360.0 {
        spirographe.hue = 0.0;
    }

    spirographe.update_positions();
}

fn update_spirographe_mesh(
    spirographe: Res<Spirographe>,
    mut meshes: ResMut<Assets<Mesh>>,
    spirographe_mesh: Query<&Mesh2dHandle, With<SpirographeMesh>>,
) {
    if !spirographe.needs_update {
        return;
    }

    let handle = spirographe_mesh.single();

    let spirographe_mesh = meshes.get_mut(&handle.0).unwrap();

    spirographe_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, spirographe.points.clone());
    spirographe_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, spirographe.colors.clone());
    spirographe_mesh.insert_indices(Indices::U32(spirographe.indices.clone()));
}

fn update_transforms(
    spirographe: Res<Spirographe>,
    mut spinning_circle: Query<&mut Transform, (With<SpinningCircle>, Without<Pencil>)>,
    mut pencil: Query<&mut Transform, With<Pencil>>,
) {
    let mut transform = spinning_circle.single_mut();
    transform.scale = Vec3::ONE * spirographe.spinning_circle_radius;

    transform.translation = Vec3 {
        x: spirographe.spinning_circle_pos.x,
        y: spirographe.spinning_circle_pos.y,
        z: 0.0,
    };

    let mut pencil_transform = pencil.single_mut();
    pencil_transform.translation = Vec3 {
        x: spirographe.pencil_position.x,
        y: spirographe.pencil_position.y,
        z: 0.0,
    };
}

fn toggle_circle_visibility(
    input: Res<ButtonInput<KeyCode>>,
    mut circles: Query<&mut Visibility, Or<(With<MainCircle>, With<SpinningCircle>)>>,
) {
    if input.just_pressed(KeyCode::KeyV) {
        for mut circle_vis in &mut circles {
            if *circle_vis == Visibility::Inherited {
                *circle_vis = Visibility::Hidden;
            } else {
                *circle_vis = Visibility::Inherited;
            }
        }
    }
}
