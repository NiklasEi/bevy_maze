use bevy::prelude::*;

use crate::UnevenSlotCoordinate;
use rand::{random, thread_rng, Rng};

pub struct Maze {
    pub height: usize,
    pub width: usize,
    pub maze_slots: Vec<Vec<UnevenMazeSlot>>,
    pub slots: Vec<Vec<MazeSlot>>,
}

pub struct MazeSlot {
    pub entity: Entity,
    pub path: bool,
}

impl Maze {
    pub fn untouched_neighbors(
        &self,
        position: &UnevenSlotCoordinate,
    ) -> Vec<UnevenSlotCoordinate> {
        self.neighbors(position)
            .iter()
            .filter(|&coord| {
                self.maze_slots
                    .get(coord.row)
                    .expect("Row out of bounds")
                    .get(coord.column)
                    .expect("Column out of bounds")
                    .state
                    == MazeSlotState::UnTouched
            })
            .map(|coord| coord.clone())
            .collect()
    }

    pub fn get_next_random_untouched_position(
        &self,
        position: &UnevenSlotCoordinate,
        previous: Option<&UnevenSlotCoordinate>,
    ) -> Option<UnevenSlotCoordinate> {
        let mut options = self.untouched_neighbors(position);
        if options.len() < 1 {
            return None;
        } else if options.len() == 1 {
            return Some(options.first().unwrap().clone());
        }
        let going_straight = self.go_straight(position, previous);
        if random::<f32>() < 0.2
            || going_straight.is_none()
            || !options.contains(&going_straight.clone().unwrap())
        {
            let mut rng = thread_rng();
            let index: usize = rng.gen_range(0..options.len());

            return Some(options.remove(index));
        }

        Some(going_straight.unwrap())
    }

    fn go_straight(
        &self,
        position: &UnevenSlotCoordinate,
        previous: Option<&UnevenSlotCoordinate>,
    ) -> Option<UnevenSlotCoordinate> {
        let slot = previous?;
        let going_straight = position.try_walk(
            position.row as i64 - slot.row as i64,
            position.column as i64 - slot.column as i64,
        )?;
        if going_straight.row > self.width - 1 && going_straight.column > self.height - 1 {
            return None;
        }
        Some(going_straight)
    }

    pub fn get_random_slot_in_row(&self, row: usize) -> UnevenSlotCoordinate {
        let mut random = thread_rng();
        let column = random.gen_range(0..self.width);

        UnevenSlotCoordinate { row, column }
    }

    fn neighbors(&self, position: &UnevenSlotCoordinate) -> Vec<UnevenSlotCoordinate> {
        let mut neighbors = vec![];
        match position {
            UnevenSlotCoordinate { row: 0, column: 0 } => {
                neighbors.push(position.walk(1, 0));
                neighbors.push(position.walk(0, 1));
            }
            UnevenSlotCoordinate {
                row: 0,
                column: width,
            } if width == &(self.width - 1) => {
                neighbors.push(position.walk(1, 0));
                neighbors.push(position.walk(0, -1));
            }
            UnevenSlotCoordinate {
                row: height,
                column: width,
            } if width == &(self.width - 1) && height == &(self.height - 1) => {
                neighbors.push(position.walk(-1, 0));
                neighbors.push(position.walk(0, -1));
            }
            UnevenSlotCoordinate {
                row: height,
                column: 0,
            } if height == &(self.height - 1) => {
                neighbors.push(position.walk(-1, 0));
                neighbors.push(position.walk(0, 1));
            }
            UnevenSlotCoordinate { row: 0, column: _ } => {
                neighbors.push(position.walk(0, 1));
                neighbors.push(position.walk(0, -1));
                neighbors.push(position.walk(1, 0));
            }
            UnevenSlotCoordinate { row: _, column: 0 } => {
                neighbors.push(position.walk(1, 0));
                neighbors.push(position.walk(0, 1));
                neighbors.push(position.walk(-1, 0));
            }
            UnevenSlotCoordinate {
                row: height,
                column: _,
            } if height == &(self.height - 1) => {
                neighbors.push(position.walk(0, 1));
                neighbors.push(position.walk(-1, 0));
                neighbors.push(position.walk(0, -1));
            }
            UnevenSlotCoordinate {
                row: _,
                column: width,
            } if width == &(self.width - 1) => {
                neighbors.push(position.walk(-1, 0));
                neighbors.push(position.walk(0, -1));
                neighbors.push(position.walk(1, 0));
            }
            UnevenSlotCoordinate { row: _, column: _ } => {
                neighbors.push(position.walk(1, 0));
                neighbors.push(position.walk(0, 1));
                neighbors.push(position.walk(-1, 0));
                neighbors.push(position.walk(0, -1));
            }
        };
        neighbors
    }
}

#[derive(Clone, Debug)]
pub struct UnevenMazeSlot {
    pub state: MazeSlotState,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MazeSlotState {
    UnTouched,
    Visited,
    Paved,
}

impl MazeSlotState {
    pub fn get_color(&self) -> Color {
        match self {
            &MazeSlotState::UnTouched => Color::DARK_GRAY,
            &MazeSlotState::Visited => Color::ORANGE,
            &MazeSlotState::Paved => Color::DARK_GREEN,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::maze::{Maze, MazeSlotState, UnevenMazeSlot};
    use crate::UnevenSlotCoordinate;

    #[test]
    fn correctly_gives_neighbors() {
        let maze = Maze {
            height: 5,
            width: 5,
            maze_slots: vec![
                vec![
                    UnevenMazeSlot {
                        state: MazeSlotState::UnTouched
                    };
                    5
                ];
                5
            ],
            slots: vec![],
        };
        assert_eq!(
            maze.neighbors(&UnevenSlotCoordinate::new(0, 0)),
            vec![
                UnevenSlotCoordinate { row: 1, column: 0 },
                UnevenSlotCoordinate { row: 0, column: 1 }
            ]
        );
        assert_eq!(
            maze.neighbors(&UnevenSlotCoordinate::new(0, 3)),
            vec![
                UnevenSlotCoordinate { row: 0, column: 4 },
                UnevenSlotCoordinate { row: 0, column: 2 },
                UnevenSlotCoordinate { row: 1, column: 3 }
            ]
        );
        assert_eq!(
            maze.neighbors(&UnevenSlotCoordinate::new(2, 3)),
            vec![
                UnevenSlotCoordinate { row: 3, column: 3 },
                UnevenSlotCoordinate { row: 2, column: 4 },
                UnevenSlotCoordinate { row: 1, column: 3 },
                UnevenSlotCoordinate { row: 2, column: 2 }
            ]
        );
    }

    #[test]
    fn filters_touched_neighbors() {
        let mut maze = Maze {
            height: 5,
            width: 5,
            maze_slots: vec![
                vec![
                    UnevenMazeSlot {
                        state: MazeSlotState::UnTouched
                    };
                    5
                ];
                5
            ],
            slots: vec![],
        };
        maze.maze_slots
            .get_mut(3)
            .unwrap()
            .get_mut(3)
            .unwrap()
            .state = MazeSlotState::Visited;
        maze.maze_slots
            .get_mut(2)
            .unwrap()
            .get_mut(2)
            .unwrap()
            .state = MazeSlotState::Visited;

        assert_eq!(
            maze.untouched_neighbors(&UnevenSlotCoordinate::new(2, 3)),
            vec![
                UnevenSlotCoordinate { row: 2, column: 4 },
                UnevenSlotCoordinate { row: 1, column: 3 }
            ]
        );
    }
}
