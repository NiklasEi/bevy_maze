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
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_startup_system(paint_starter.system())
        .add_system(generate.system())
        .run();
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

fn paint_starter(mut commands: Commands) {
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
                .insert(SlotCoordinate { column, row })
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
    set_maze_slot_state(
        &mut commands,
        SlotCoordinate::from(starting_point),
        &mut maze,
        MazeSlotState::Visited,
    );
    commands.insert_resource(maze);
}

fn generate(
    mut commands: Commands,
    mut position: ResMut<UnevenSlotCoordinate>,
    mut maze: ResMut<Maze>,
    mut stack: ResMut<Stack>,
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
        }
    }
}

fn visit(
    mut commands: &mut Commands,
    from: &UnevenSlotCoordinate,
    to: &UnevenSlotCoordinate,
    mut maze: &mut Maze,
) {
    let to_visit = SlotCoordinate::from(to.clone());
    set_maze_slot_state(
        &mut commands,
        to_visit.clone(),
        &mut maze,
        MazeSlotState::Visited,
    );
    set_maze_slot_state(
        &mut commands,
        to_visit.walk(
            from.row as i64 - to.row as i64,
            from.column as i64 - to.column as i64,
        ),
        &mut maze,
        MazeSlotState::Visited,
    );
    maze.maze_slots
        .get_mut(to.row)
        .unwrap()
        .get_mut(to.column)
        .unwrap()
        .state = MazeSlotState::Visited;
}

fn pave(
    mut commands: &mut Commands,
    from: &UnevenSlotCoordinate,
    to: &UnevenSlotCoordinate,
    mut maze: &mut Maze,
) {
    let to_pave = SlotCoordinate::from(to.clone());
    set_maze_slot_state(
        &mut commands,
        SlotCoordinate::from(from.clone()),
        &mut maze,
        MazeSlotState::Paved,
    );
    set_maze_slot_state(
        &mut commands,
        to_pave.walk(
            from.row as i64 - to.row as i64,
            from.column as i64 - to.column as i64,
        ),
        &mut maze,
        MazeSlotState::Paved,
    );
    maze.maze_slots
        .get_mut(to.row)
        .unwrap()
        .get_mut(to.column)
        .unwrap()
        .state = MazeSlotState::Paved;
}

fn set_maze_slot_state(
    commands: &mut Commands,
    to_visit: SlotCoordinate,
    maze: &mut Maze,
    state: MazeSlotState,
) {
    let entity = maze
        .slots
        .get(to_visit.row)
        .unwrap()
        .get(to_visit.column)
        .unwrap()
        .clone();
    commands.entity(entity).despawn();
    let entity = commands
        .spawn_bundle(GeometryBuilder::build_as(
            &get_tile(),
            ShapeColors::new(state.get_color()),
            DrawMode::Fill(FillOptions::default()),
            Transform::from_translation(Vec3::new(
                to_visit.column as f32 * TILE_SIZE,
                to_visit.row as f32 * TILE_SIZE,
                0.,
            )),
        ))
        .insert(to_visit.clone())
        .id();
    maze.slots
        .get_mut(to_visit.row)
        .unwrap()
        .remove(to_visit.column);
    maze.slots
        .get_mut(to_visit.row)
        .unwrap()
        .insert(to_visit.column, entity.clone());
}
