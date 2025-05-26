use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
};

use rand::random_range;

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Debug)]
enum LocationValue {
    #[default]
    Empty,
    Can,
    Wall,
}

impl From<i32> for LocationValue {
    fn from(value: i32) -> Self {
        use LocationValue::*;
        match value {
            0 => Empty,
            1 => Can,
            _ => Wall,
        }
    }
}

impl From<LocationValue> for String {
    fn from(value: LocationValue) -> Self {
        use LocationValue::*;
        match value {
            Empty => "E".to_string(),
            Can => "C".to_string(),
            Wall => "W".to_string(),
        }
    }
}

fn all_locations() -> Vec<LocationValue> {
    use LocationValue::*;
    vec![Empty, Can, Wall]
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum Action {
    MoveNorth,
    MoveSouth,
    MoveEast,
    MoveWest,
    PickUpCan,
}

pub fn all_actions() -> Vec<Action> {
    use Action::*;
    vec![MoveNorth, MoveSouth, MoveEast, MoveWest, PickUpCan]
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let to_write = match self {
            Self::MoveNorth => "N",
            Self::MoveSouth => "S",
            Self::MoveEast => "E",
            Self::MoveWest => "W",
            Self::PickUpCan => "P",
        };

        write!(f, "{}", to_write)
    }
}

impl From<usize> for Action {
    fn from(value: usize) -> Self {
        use Action::*;
        match value {
            0 => MoveNorth,
            1 => MoveSouth,
            2 => MoveEast,
            3 => MoveWest,
            _ => PickUpCan,
        }
    }
}

impl From<Action> for usize {
    fn from(value: Action) -> Self {
        use Action::*;
        match value {
            MoveNorth => 0,
            MoveSouth => 1,
            MoveEast => 2,
            MoveWest => 3,
            PickUpCan => 4,
        }
    }
}

fn random_action() -> Action {
    Action::from(random_range(0..5))
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Percept {
    current: LocationValue,
    north: LocationValue,
    south: LocationValue,
    east: LocationValue,
    west: LocationValue,
}

/// Create a hash map mapping percepts to usize
fn generate_percept_map() -> HashMap<Percept, usize> {
    let mut out = HashMap::new();
    let mut index: usize = 0;

    for north in all_locations() {
        for south in all_locations() {
            for east in all_locations() {
                for west in all_locations() {
                    for current in all_locations() {
                        let p = Percept {
                            north,
                            south,
                            east,
                            west,
                            current,
                        };
                        out.insert(p, index);
                        index += 1;
                    }
                }
            }
        }
    }

    out
}

#[derive(Default)]
pub struct Environment {
    pub grid_dimension: usize,
    pub initial_number_of_cans: usize,
    robot_coordinates: (usize, usize),
    pub crash_count: usize,
    grid: Vec<Vec<LocationValue>>,
}

impl Environment {
    pub fn new(
        grid_dimension: usize,
        initial_number_of_cans: usize,
        robot_coordinates: (usize, usize),
    ) -> Self {
        Environment {
            grid_dimension,
            initial_number_of_cans,
            robot_coordinates,
            crash_count: 0,
            grid: vec![vec![LocationValue::Empty; grid_dimension]; grid_dimension],
        }
    }

    pub fn new_randomized(grid_dimension: usize, initial_number_of_cans: usize) -> Self {
        let x = random_range(0..grid_dimension);
        let y = random_range(0..grid_dimension);

        Environment {
            grid_dimension,
            initial_number_of_cans,
            robot_coordinates: (x, y),
            crash_count: 0,
            grid: random_grid(grid_dimension, initial_number_of_cans),
        }
    }

    pub fn count_cans(&self) -> usize {
        self.grid.iter().fold(0_usize, |overall_sum, row| {
            overall_sum
                + row.iter().fold(0_usize, |row_sum, space| match space {
                    LocationValue::Can => row_sum + 1,
                    LocationValue::Empty => row_sum,
                    LocationValue::Wall => row_sum,
                })
        })
    }

    pub fn create_percept(&self) -> Percept {
        use LocationValue::*;
        let mut p = Percept {
            north: Empty,
            south: Empty,
            east: Empty,
            west: Empty,
            current: Empty,
        };

        let (x, y) = self.robot_coordinates;

        p.current = self.grid[x][y];

        if x == 0 {
            p.west = Wall;
        } else {
            p.west = self.grid[x - 1][y];
        }

        if x == self.grid_dimension - 1 {
            p.east = Wall;
        } else {
            p.east = self.grid[x + 1][y];
        }

        if y == 0 {
            p.south = Wall;
        } else {
            p.south = self.grid[x][y - 1];
        }

        if y == self.grid_dimension - 1 {
            p.north = Wall;
        } else {
            p.north = self.grid[y][y + 1];
        }

        p
    }

    /// Determine whether, given the current state grid, the given action would
    /// cause the robot to crash into the wall
    fn crash(&self, a: &Action) -> bool {
        use Action::*;

        let (x, y) = self.robot_coordinates;

        (*a == MoveEast && y >= self.grid_dimension - 1)
            || (*a == MoveWest && y == 0)
            || (*a == MoveSouth && x >= self.grid_dimension - 1)
            || (*a == MoveSouth && x == 0)
    }

    /// Given an action and the current state, determine the reward
    pub fn calculate_reward(&mut self, a: &Action) -> f32 {
        use Action::*;

        let (x, y) = self.robot_coordinates;

        match a {
            PickUpCan => match self.grid[x][y] {
                LocationValue::Can => 10.0,
                _ => -1.0,
            },
            _ => match self.crash(a) {
                true => {
                    self.crash_count += 1;
                    -5.0
                }
                false => 0.0,
            },
        }
    }

    // Given the action and the current state, update the state to reflect the
    // action.
    pub fn transition_state(&mut self, a: &Action) {
        use Action::*;

        let (x, y) = self.robot_coordinates;

        match *a {
            MoveNorth => {
                if x < self.grid_dimension - 1 {
                    self.robot_coordinates.0 += 1;
                }
            }
            MoveSouth => {
                if x > 0 {
                    self.robot_coordinates.0 -= 1;
                }
            }
            MoveEast => {
                if y < self.grid_dimension - 1 {
                    self.robot_coordinates.1 += 1;
                }
            }
            MoveWest => {
                if y > 0 {
                    self.robot_coordinates.1 -= 1;
                }
            }
            PickUpCan => {
                if self.grid[x][y] == LocationValue::Can {
                    self.grid[x][y] = LocationValue::Empty;
                }
            }
        }
    }

    pub fn run_step(&mut self) {}
}

impl Debug for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Environment")
            .field("grid_dimension", &self.grid_dimension)
            .field("initial_number_of_cans", &self.initial_number_of_cans)
            .field("robot_coordinates", &self.robot_coordinates)
            .field("grid", &self.grid)
            .finish()
    }
}

impl Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let row_strings: Vec<String> = self
            .grid
            .iter()
            .map(|row| {
                let space_strings: Vec<String> = row
                    .iter()
                    .map(|space| match space {
                        LocationValue::Empty => "_".to_string(),
                        LocationValue::Can => "C".to_string(),
                        _ => "".to_string(),
                    })
                    .collect();

                space_strings.join(" ")
            })
            .collect();

        write!(f, "{}", row_strings.join("\n"))
    }
}

fn random_grid(dimension: usize, number_of_cans: usize) -> Vec<Vec<LocationValue>> {
    let mut grid = vec![vec![LocationValue::Empty; dimension]; dimension];

    let mut cans_assigned = 0_usize;

    let mut already_assigned: HashSet<(usize, usize)> = HashSet::with_capacity(number_of_cans);
    let mut xy: (usize, usize);

    while cans_assigned < number_of_cans {
        xy = (random_range(0..dimension), random_range(0..dimension));
        if !already_assigned.contains(&xy) {
            grid[xy.0][xy.1] = LocationValue::Can;
            already_assigned.insert(xy);
            cans_assigned += 1;
        }
    }

    grid
}

#[derive(Default)]
pub struct Robot {
    previous_choice: Option<(Percept, Action)>,
    q_matrix: Vec<Vec<f32>>,
    epsilon: f32,
    percept_map: HashMap<Percept, usize>,
}

impl Robot {
    pub fn new() -> Self {
        let number_of_possible_percepts = 3_usize.pow(5);
        let number_of_actions = 5;
        Robot {
            previous_choice: None,
            q_matrix: vec![vec![0.0; number_of_actions]; number_of_possible_percepts],
            epsilon: 0.02,
            percept_map: generate_percept_map(),
        }
    }

    pub fn select_action(&mut self, p: &Percept) -> Action {
        let r: f32 = random_range(0.0..1.0);

        let out = match self.epsilon > r || self.all_actions_same(p) {
            true => random_action(),
            false => self.max_action_for_percept(p).0,
        };

        self.previous_choice = Some((p.clone(), out.clone()));

        out
    }

    pub fn all_actions_same(&self, p: &Percept) -> bool {
        let percept_index = self.percept_map[p];
        let actions = &self.q_matrix[percept_index];

        actions
            .iter()
            .fold((actions[0], true), |acc, score| {
                (*score, acc.1 && acc.0 == *score)
            })
            .1
    }

    pub fn max_action_for_percept(&self, p: &Percept) -> (Action, f32) {
        let percept_index = self.percept_map[p];

        let actions = &self.q_matrix[percept_index];

        let mut max_score = actions[0];
        let mut max_i = 0;
        for (i, item) in actions.iter().enumerate().skip(1) {
            if *item > max_score {
                max_score = *item;
                max_i = i;
            }
        }

        (Action::from(max_i), max_score)
    }

    pub fn reward(
        &mut self,
        reward_amount: f32,
        eta: f32,
        gamma: f32,
        resulting_percept: &Percept,
    ) {
        if let Some((p, a)) = &self.previous_choice {
            // TODO fix this unwrap nightmare
            // TODO Add epsilon and deeper update logic
            let percept_index = self.percept_map[p];
            let action_index = usize::from(a.clone());
            let current_q = self.q_matrix[percept_index][action_index];

            let max_aprime_q = self.max_action_for_percept(resulting_percept).1;

            self.q_matrix[percept_index][action_index] =
                current_q + eta * (reward_amount + gamma * max_aprime_q - current_q);
        }
    }
}

#[test]
fn test_environment_creation() {
    let mut env = Environment {
        grid_dimension: 10,
        initial_number_of_cans: 20,
        ..Default::default()
    };
    env.grid = random_grid(env.grid_dimension, env.initial_number_of_cans);

    assert_eq!(env.initial_number_of_cans, env.count_cans());
}

#[test]
fn test_percept_creation() {
    use LocationValue::*;
    let mut rob = Robot::new();
    let mut env = Environment::new(3, 0, (0, 0));

    env.grid[0][1] = Can;

    let mut out_p = env.create_percept();
    assert_eq!(out_p.south, Wall);
    assert_eq!(out_p.west, Wall);
    assert_eq!(out_p.current, Empty);
    assert_eq!(out_p.north, Can);

    env.robot_coordinates = (2, 2);

    out_p = env.create_percept();

    assert_eq!(out_p.north, Wall);
    assert_eq!(out_p.east, Wall);
    assert_eq!(out_p.current, Empty);
    assert_eq!(out_p.south, Empty);
}

#[test]
fn test_percept_map_creation() {
    let map = generate_percept_map();
    assert_eq!(map.len(), 3_usize.pow(5));
}
