use bevy::prelude::*;

use crate::UnevenSlotCoordinate;
use rand::{thread_rng, Rng};

pub struct Maze {
    pub height: usize,
    pub width: usize,
    pub maze_slots: Vec<Vec<MazeSlot>>,
    pub slots: Vec<Vec<Entity>>,
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
pub struct MazeSlot {
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
    use crate::maze::{Maze, MazeSlot, MazeSlotState};
    use crate::UnevenSlotCoordinate;

    #[test]
    fn correctly_gives_neighbors() {
        let maze = Maze {
            height: 5,
            width: 5,
            maze_slots: vec![
                vec![
                    MazeSlot {
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
                    MazeSlot {
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
