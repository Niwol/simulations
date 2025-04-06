use bevy::{
    color::palettes,
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};
use bevy_egui::{
    egui::{self, DragValue},
    EguiContexts, EguiPlugin,
};

const CELL_SIZE: usize = 32;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                canvas: Some(String::from("#game-canvas")),
                ..Default::default()
            }),
            ..Default::default()
        }))
        // .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_event::<ResetMazeEvent>()
        .init_resource::<MazeConfig>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                exit_on_escape,
                pan_and_zoom,
                ui,
                toggle_pause,
                update.run_if(resource_exists::<Maze>),
                reset_maze.run_if(on_event::<ResetMazeEvent>),
            ),
        )
        .run();
}

fn exit_on_escape(input: Res<ButtonInput<KeyCode>>, mut exit_event: EventWriter<AppExit>) {
    if input.just_pressed(KeyCode::Escape) {
        exit_event.send_default();
    }
}

#[derive(Event, Default)]
struct ResetMazeEvent;

#[derive(Clone, Copy, PartialEq, Eq)]
enum SolvingMode {
    Paused,
    Running,
    Stepping,
}

#[derive(Resource)]
struct MazeConfig {
    width: usize,
    height: usize,
    solving_mode: SolvingMode,
}

fn ui(
    mut ctx: EguiContexts,
    mut maze_config: ResMut<MazeConfig>,
    mut reset_event: EventWriter<ResetMazeEvent>,
) {
    egui::SidePanel::left("Sied panel").show(ctx.ctx_mut(), |ui| {
        ui.heading("Maze Generation");

        ui.horizontal(|ui| {
            ui.add(DragValue::new(&mut maze_config.width));
            ui.add(DragValue::new(&mut maze_config.height));

            maze_config.width = maze_config.width.max(2);
            maze_config.height = maze_config.height.max(2)
        });

        if ui.button("Reset Maze").clicked() {
            reset_event.send_default();
        }

        ui.separator();

        ui.selectable_value(&mut maze_config.solving_mode, SolvingMode::Paused, "Pause");
        ui.selectable_value(&mut maze_config.solving_mode, SolvingMode::Running, "Solve");
        ui.selectable_value(
            &mut maze_config.solving_mode,
            SolvingMode::Stepping,
            "Stepping",
        );
    });
}

#[derive(Resource)]
struct Maze {
    cells: Vec<Cell>,
    width: usize,
    height: usize,
    stack: Vec<UVec2>,
}

#[derive(Clone, Copy)]
struct Cell {
    entity: Entity,
    walls: Walls,
    visited: bool,
}

#[derive(Clone, Copy)]
struct Walls {
    up: Option<Entity>,
    down: Option<Entity>,
    left: Option<Entity>,
    right: Option<Entity>,
}

#[derive(Clone, Copy)]
struct Step {
    from_coord: UVec2,
    to_coord: UVec2,
    opend_walls: bool,
}

impl Step {
    fn direction(&self) -> Direction {
        Direction::from_coords(self.from_coord, self.to_coord)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Default for MazeConfig {
    fn default() -> Self {
        Self {
            width: 30,
            height: 30,
            solving_mode: SolvingMode::Paused,
        }
    }
}

impl Direction {
    fn to_coord(&self) -> IVec2 {
        match self {
            Direction::Up => IVec2::Y,
            Direction::Down => IVec2::NEG_Y,
            Direction::Left => IVec2::NEG_X,
            Direction::Right => IVec2::X,
        }
    }

    fn from_coords(from: UVec2, to: UVec2) -> Self {
        let vec = to.as_ivec2() - from.as_ivec2();

        match vec {
            IVec2::Y => Direction::Up,
            IVec2::NEG_Y => Direction::Down,
            IVec2::NEG_X => Direction::Left,
            IVec2::X => Direction::Right,

            _ => panic!("coord vectors were not adjacent"),
        }
    }
}

impl Maze {
    fn step(&mut self) -> Step {
        self.mark_current_cell();

        let current_coord = *self.stack.last().unwrap();

        let directions = [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ]
        .into_iter()
        .filter(|direction| {
            let dir = direction.to_coord();

            let new_coord = dir + current_coord.as_ivec2();
            let outside_maze = new_coord.x < 0
                || new_coord.x >= self.width as i32
                || new_coord.y < 0
                || new_coord.y >= self.height as i32;

            if outside_maze {
                return false;
            }

            let idx = coord_to_idx(new_coord.as_uvec2(), self.width);
            let next_cell = self.cells[idx];
            if next_cell.visited {
                return false;
            }

            true
        })
        .collect::<Vec<Direction>>();

        if directions.is_empty() {
            self.stack.pop();
            let new_coord = *self.stack.last().unwrap();
            return Step {
                from_coord: current_coord,
                to_coord: new_coord,
                opend_walls: false,
            };
        }

        let r = rand::random_range(0..directions.len());
        let direction = directions[r];

        let new_coord = (current_coord.as_ivec2() + direction.to_coord()).as_uvec2();

        self.stack.push(new_coord);

        Step {
            from_coord: current_coord,
            to_coord: new_coord,
            opend_walls: true,
        }
    }

    fn mark_current_cell(&mut self) {
        let current_coord = self.stack.last().unwrap();
        let idx = coord_to_idx(*current_coord, self.width);
        self.cells[idx].visited = true;
    }

    fn open_walls(&mut self, step: Step) {
        let width = self.width;
        let direction = step.direction();
        match direction {
            Direction::Up => {
                self.cells
                    .get_mut(coord_to_idx(step.from_coord, width))
                    .as_mut()
                    .unwrap()
                    .walls
                    .up
                    .take();
                self.cells
                    .get_mut(coord_to_idx(step.to_coord, width))
                    .as_mut()
                    .unwrap()
                    .walls
                    .down
                    .take();
            }

            Direction::Down => {
                self.cells
                    .get_mut(coord_to_idx(step.from_coord, width))
                    .as_mut()
                    .unwrap()
                    .walls
                    .down
                    .take();
                self.cells
                    .get_mut(coord_to_idx(step.to_coord, width))
                    .as_mut()
                    .unwrap()
                    .walls
                    .up
                    .take();
            }

            Direction::Left => {
                self.cells
                    .get_mut(coord_to_idx(step.from_coord, width))
                    .as_mut()
                    .unwrap()
                    .walls
                    .left
                    .take();
                self.cells
                    .get_mut(coord_to_idx(step.to_coord, width))
                    .as_mut()
                    .unwrap()
                    .walls
                    .right
                    .take();
            }

            Direction::Right => {
                self.cells
                    .get_mut(coord_to_idx(step.from_coord, width))
                    .as_mut()
                    .unwrap()
                    .walls
                    .right
                    .take();
                self.cells
                    .get_mut(coord_to_idx(step.to_coord, width))
                    .as_mut()
                    .unwrap()
                    .walls
                    .left
                    .take();
            }
        }
    }

    fn complete(&self) -> bool {
        self.cells[0].visited && self.stack.len() == 1
    }
}

fn setup(mut commands: Commands, mut reset_event: EventWriter<ResetMazeEvent>) {
    commands.spawn(Camera2d);
    reset_event.send_default();
}

fn reset_maze(mut commands: Commands, maze_config: Res<MazeConfig>, maze: Option<Res<Maze>>) {
    if let Some(maze) = maze {
        for cell in &maze.cells {
            commands.entity(cell.entity).despawn();
            if let Some(e) = cell.walls.up {
                commands.entity(e).despawn();
            }
            if let Some(e) = cell.walls.down {
                commands.entity(e).despawn();
            }
            if let Some(e) = cell.walls.left {
                commands.entity(e).despawn();
            }
            if let Some(e) = cell.walls.right {
                commands.entity(e).despawn();
            }
        }
    }

    let x_offset = CELL_SIZE * maze_config.width / 2;
    let y_offset = CELL_SIZE * maze_config.height / 2;

    let mut maze = Maze {
        cells: Vec::new(),
        width: maze_config.width,
        height: maze_config.height,
        stack: vec![UVec2::ZERO],
    };

    for y in 0..maze_config.height {
        for x in 0..maze_config.width {
            let x = (CELL_SIZE * x) as f32 - x_offset as f32;
            let y = (CELL_SIZE * y) as f32 - y_offset as f32;

            let cell = spawn_cell(&mut commands, x, y);
            maze.cells.push(cell);
        }
    }

    commands.insert_resource(maze);
}

fn spawn_cell(commands: &mut Commands, x: f32, y: f32) -> Cell {
    let up = commands
        .spawn((
            Sprite {
                color: palettes::basic::BLACK.into(),
                custom_size: Some(Vec2 {
                    x: CELL_SIZE as f32,
                    y: 2.0,
                }),
                ..Default::default()
            },
            Transform::from_translation(Vec3 {
                x,
                y: y + CELL_SIZE as f32 / 2.0,
                z: 1.0,
            }),
        ))
        .id();

    let down = commands
        .spawn((
            Sprite {
                color: palettes::basic::BLACK.into(),
                custom_size: Some(Vec2 {
                    x: CELL_SIZE as f32,
                    y: 2.0,
                }),
                ..Default::default()
            },
            Transform::from_translation(Vec3 {
                x,
                y: y - CELL_SIZE as f32 / 2.0,
                z: 1.0,
            }),
        ))
        .id();

    let left = commands
        .spawn((
            Sprite {
                color: palettes::basic::BLACK.into(),
                custom_size: Some(Vec2 {
                    x: 2.0,
                    y: CELL_SIZE as f32,
                }),
                ..Default::default()
            },
            Transform::from_translation(Vec3 {
                x: x - CELL_SIZE as f32 / 2.0,
                y,
                z: 1.0,
            }),
        ))
        .id();

    let right = commands
        .spawn((
            Sprite {
                color: palettes::basic::BLACK.into(),
                custom_size: Some(Vec2 {
                    x: 2.0,
                    y: CELL_SIZE as f32,
                }),
                ..Default::default()
            },
            Transform::from_translation(Vec3 {
                x: x + CELL_SIZE as f32 / 2.0,
                y,
                z: 1.0,
            }),
        ))
        .id();

    let entity = commands
        .spawn((
            Sprite {
                color: palettes::basic::AQUA.into(),
                custom_size: Some(Vec2::splat(CELL_SIZE as f32)),
                ..Default::default()
            },
            Transform::from_translation(Vec3 { x, y, z: 0.0 }),
        ))
        .id();

    Cell {
        entity,
        walls: Walls {
            up: Some(up),
            down: Some(down),
            left: Some(left),
            right: Some(right),
        },
        visited: false,
    }
}

fn update(
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut maze: ResMut<Maze>,
    maze_config: Res<MazeConfig>,
    mut cells: Query<&mut Sprite>,
) {
    if maze.complete() {
        return;
    }

    let should_step = match maze_config.solving_mode {
        SolvingMode::Paused => false,
        SolvingMode::Running => true,
        SolvingMode::Stepping => input.just_pressed(KeyCode::Space),
    };

    // let previous_cell = maze.current_cell();

    let step = if should_step {
        maze.step()
    } else {
        return;
    };

    // let current_cell = maze.current_cell();

    let width = maze.width;
    let previous_cell = maze.cells[coord_to_idx(step.from_coord, width)];
    let current_cell = maze.cells[coord_to_idx(step.to_coord, width)];

    if step.opend_walls {
        #[rustfmt::skip]
        match step.direction() {
            Direction::Up => {
                commands.entity(previous_cell.walls.up.unwrap()).despawn();
                commands.entity(current_cell.walls.down.unwrap()).despawn();
            }
            Direction::Down => {
                commands.entity(previous_cell.walls.down.unwrap()).despawn();
                commands.entity(current_cell.walls.up.unwrap()).despawn();
            }
            Direction::Left => {
                commands.entity(previous_cell.walls.left.unwrap()).despawn();
                commands.entity(current_cell.walls.right.unwrap()).despawn();
            }
            Direction::Right => {
                commands.entity(previous_cell.walls.right.unwrap()).despawn();
                commands.entity(current_cell.walls.left.unwrap()).despawn();
            }
        };

        maze.open_walls(step);
    }

    cells.get_mut(previous_cell.entity).unwrap().color = palettes::basic::FUCHSIA.into();
    cells.get_mut(current_cell.entity).unwrap().color = palettes::basic::BLUE.into();
}

fn pan_and_zoom(
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut camera: Single<&mut Transform, With<Camera2d>>,
    mut projection: Single<&mut OrthographicProjection, With<Camera2d>>,
    mut mouse_motion_event: EventReader<MouseMotion>,
    mut mouse_wheel: EventReader<MouseWheel>,
) {
    if mouse_input.pressed(MouseButton::Middle) {
        for mouse_motion in mouse_motion_event.read() {
            let mut delta = mouse_motion.delta;
            delta.x *= -1.0;

            delta *= projection.scale;

            camera.translation += Vec3::from((delta, 0.0));
        }
    }

    for mouse_wheel in mouse_wheel.read() {
        if mouse_wheel.y > 0.0 {
            projection.scale *= 0.5;
        } else if mouse_wheel.y < 0.0 {
            projection.scale *= 2.0;
        }
    }
}

fn toggle_pause(input: Res<ButtonInput<KeyCode>>, mut maze_config: ResMut<MazeConfig>) {
    if input.just_pressed(KeyCode::Space) {
        maze_config.solving_mode = match maze_config.solving_mode {
            SolvingMode::Paused => SolvingMode::Running,
            SolvingMode::Running => SolvingMode::Paused,
            SolvingMode::Stepping => SolvingMode::Stepping,
        };
    }
}

fn coord_to_idx(coord: UVec2, width: usize) -> usize {
    coord.y as usize * width + coord.x as usize
}

fn _idx_to_coord(idx: usize, width: usize) -> UVec2 {
    UVec2 {
        x: (idx / width) as u32,
        y: (idx % width) as u32,
    }
}
