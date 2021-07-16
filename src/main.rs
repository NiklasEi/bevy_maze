mod maze;

use crate::maze::{Maze, MazeSlot, MazeSlotState};
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::shapes::Rectangle;
use bevy_prototype_lyon::prelude::*;
use rand::{thread_rng, Rng};
use std::convert::TryFrom;

const TILE_SIZE: f32 = 8.;

fn main() {
    App::build()
        .insert_resource(ClearColor(MazeSlotState::UnTouched.get_color()))
        .insert_resource(WindowDescriptor {
            width: 800.0,
            height: 600.0,
            title: "Maze".to_string(),
            ..WindowDescriptor::default()
        })
        .insert_resource::<Stack>(vec![])
        .add_state(AppState::TriggerGeneration)
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_system_set(
            SystemSet::on_enter(AppState::TriggerGeneration)
                .with_system(trigger_generation.exclusive_system()),
        )
        .add_system_set(
            SystemSet::on_enter(AppState::Generating).with_system(prepare_maze.system()),
        )
        .add_system_set(
            SystemSet::on_update(AppState::Generating).with_system(draw_next_path.system()),
        )
        .add_system_set(SystemSet::on_update(AppState::Done).with_system(wait_for_restart.system()))
        .run();
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum AppState {
    TriggerGeneration,
    Generating,
    Done,
}

type Stack = Vec<UnevenSlotCoordinate>;

#[derive(Clone, PartialEq, Debug)]
pub struct UnevenSlotCoordinate {
    pub row: usize,
    pub column: usize,
}

impl UnevenSlotCoordinate {
    fn walk(&self, row_delta: i64, column_delta: i64) -> Self {
        UnevenSlotCoordinate {
            row: usize::try_from(self.row as i64 + row_delta)
                .expect("Overflow navigating the maze board"),
            column: usize::try_from(self.column as i64 + column_delta)
                .expect("Overflow navigating the maze board"),
        }
    }
}

impl From<UnevenSlotCoordinate> for SlotCoordinate {
    fn from(uneven_slot_coordinate: UnevenSlotCoordinate) -> Self {
        SlotCoordinate {
            row: 1 + 2 * uneven_slot_coordinate.row,
            column: 1 + 2 * uneven_slot_coordinate.column,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SlotCoordinate {
    pub row: usize,
    pub column: usize,
}

impl SlotCoordinate {
    fn walk(&self, row_delta: i64, column_delta: i64) -> Self {
        SlotCoordinate {
            row: usize::try_from(self.row as i64 + row_delta)
                .expect("Overflow navigating the maze board"),
            column: usize::try_from(self.column as i64 + column_delta)
                .expect("Overflow navigating the maze board"),
        }
    }
}

fn get_tile() -> Rectangle {
    Rectangle {
        width: TILE_SIZE,
        height: TILE_SIZE,
        origin: Default::default(),
    }
}

fn trigger_generation(world: &mut World) {
    let stack = world.get_resource_mut::<Stack>();
    if let Some(mut stack) = stack {
        stack.clear()
    }
    let maze = world.get_resource_mut::<Maze>();
    let mut despawn = vec![];
    if let Some(mut maze) = maze {
        for mut row in maze.slots.drain(..) {
            for entity in row.drain(..) {
                despawn.push(entity);
            }
        }
    }
    for entity in despawn.drain(..) {
        world.entity_mut(entity).despawn()
    }

    if let Some(mut state) = world.get_resource_mut::<State<AppState>>() {
        state.set(AppState::Generating).unwrap();
    }
}

fn prepare_maze(mut commands: Commands) {
    let mut maze = Maze {
        height: 37,
        width: 49,
        maze_slots: vec![],
        slots: vec![],
    };
    commands.spawn_bundle(OrthographicCameraBundle {
        transform: Transform::from_translation(Vec3::new(
            maze.width as f32 * TILE_SIZE,
            maze.height as f32 * TILE_SIZE,
            999.9,
        )),
        ..OrthographicCameraBundle::new_2d()
    });
    for row in 0..(2 * maze.height + 1) {
        let mut current_slot_row = vec![];
        let mut current_row = vec![];
        for column in 0..(2 * maze.width + 1) {
            let entity = commands
                .spawn_bundle(GeometryBuilder::build_as(
                    &get_tile(),
                    ShapeColors::new(MazeSlotState::UnTouched.get_color()),
                    DrawMode::Fill(FillOptions::default()),
                    Transform::from_translation(Vec3::new(
                        column as f32 * TILE_SIZE,
                        row as f32 * TILE_SIZE,
                        0.,
                    )),
                ))
                .id();
            current_slot_row.push(entity);
            if column % 2 != 0 && row % 2 != 0 {
                current_row.push(MazeSlot {
                    state: MazeSlotState::UnTouched,
                });
            }
        }
        maze.slots.push(current_slot_row);
        if row % 2 != 0 {
            maze.maze_slots.push(current_row);
        }
    }
    let starting_point = UnevenSlotCoordinate { row: 0, column: 0 };
    commands.insert_resource(starting_point.clone());
    maze.maze_slots
        .get_mut(starting_point.row)
        .unwrap()
        .get_mut(starting_point.column)
        .unwrap()
        .state = MazeSlotState::Visited;
    set_slot_state(
        &mut commands,
        SlotCoordinate::from(starting_point),
        &mut maze,
        &MazeSlotState::Visited,
    );
    commands.insert_resource(maze);
}

fn draw_next_path(
    mut commands: Commands,
    mut position: ResMut<UnevenSlotCoordinate>,
    mut maze: ResMut<Maze>,
    mut stack: ResMut<Stack>,
    mut state: ResMut<State<AppState>>,
) {
    let options = maze.untouched_neighbors(&position);
    if options.len() > 0 {
        let mut rng = thread_rng();
        let index: usize = rng.gen_range(0..options.len());

        visit(
            &mut commands,
            &position,
            options.get(index).unwrap(),
            &mut maze,
        );

        stack.push(position.clone());
        *position = options.get(index).unwrap().clone();
    } else {
        if let Some(last_position) = stack.pop() {
            pave(&mut commands, &position, &last_position, &mut maze);
            *position = last_position;
        } else {
            set_slot_state(
                &mut commands,
                SlotCoordinate::from(position.clone()),
                &mut maze,
                &MazeSlotState::Paved,
            );
            state.set(AppState::Done).unwrap();
        }
    }
}

fn visit(
    mut commands: &mut Commands,
    from: &UnevenSlotCoordinate,
    to: &UnevenSlotCoordinate,
    maze: &mut Maze,
) {
    set_path_state(&mut commands, to, from, maze, MazeSlotState::Visited);
}

fn pave(
    mut commands: &mut Commands,
    from: &UnevenSlotCoordinate,
    to: &UnevenSlotCoordinate,
    maze: &mut Maze,
) {
    set_path_state(&mut commands, from, to, maze, MazeSlotState::Paved);
}

fn set_path_state(
    mut commands: &mut Commands,
    set: &UnevenSlotCoordinate,
    connecting_to: &UnevenSlotCoordinate,
    mut maze: &mut Maze,
    state: MazeSlotState,
) {
    let to_visit = SlotCoordinate::from(set.clone());
    set_slot_state(
        &mut commands,
        SlotCoordinate::from(set.clone()),
        &mut maze,
        &state,
    );
    set_slot_state(
        &mut commands,
        to_visit.walk(
            connecting_to.row as i64 - set.row as i64,
            connecting_to.column as i64 - set.column as i64,
        ),
        &mut maze,
        &state,
    );
    maze.maze_slots
        .get_mut(set.row)
        .unwrap()
        .get_mut(set.column)
        .unwrap()
        .state = state;
}

fn set_slot_state(
    commands: &mut Commands,
    slot: SlotCoordinate,
    maze: &mut Maze,
    state: &MazeSlotState,
) {
    let entity = maze
        .slots
        .get(slot.row)
        .unwrap()
        .get(slot.column)
        .unwrap()
        .clone();
    commands.entity(entity).despawn();
    let entity = commands
        .spawn_bundle(GeometryBuilder::build_as(
            &get_tile(),
            ShapeColors::new(state.get_color()),
            DrawMode::Fill(FillOptions::default()),
            Transform::from_translation(Vec3::new(
                slot.column as f32 * TILE_SIZE,
                slot.row as f32 * TILE_SIZE,
                0.,
            )),
        ))
        .id();
    maze.slots.get_mut(slot.row).unwrap().remove(slot.column);
    maze.slots
        .get_mut(slot.row)
        .unwrap()
        .insert(slot.column, entity.clone());
}

fn wait_for_restart(input: Res<Input<KeyCode>>, mut state: ResMut<State<AppState>>) {
    if input.just_pressed(KeyCode::R) {
        state.set(AppState::TriggerGeneration).unwrap();
    }
}
