use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    mem,
    panic::Location,
};

use rand::{distr::slice::Empty, random_range};

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

fn all_locations() -> Vec<LocationValue> {
    use LocationValue::*;
    vec![Empty, Can, Wall]
}

fn empty_location_map() -> HashMap<LocationValue, f32> {
    let mut out: HashMap<LocationValue, f32> = HashMap::new();

    for location in all_locations() {
        out.insert(location, 0.0);
    }

    out
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum Action {
    MoveNorth,
    MoveSouth,
    MoveEast,
    MoveWest,
    PickUpCan,
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

fn empty_action_map() -> HashMap<Action, f32> {
    use Action::*;
    let mut out: HashMap<Action, f32> = HashMap::new();
    let actions = vec![MoveNorth, MoveSouth, MoveEast, MoveWest, PickUpCan];

    for a in actions {
        out.insert(a, 0.0);
    }

    out
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Percept {
    current: LocationValue,
    north: LocationValue,
    south: LocationValue,
    east: LocationValue,
    west: LocationValue,
}

#[derive(Default)]
pub struct Environment {
    pub grid_dimension: usize,
    pub initial_number_of_cans: usize,
    robot_coordinates: (usize, usize),
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
    pub fn calculate_reward(&self, a: &Action) -> f32 {
        use Action::*;

        let (x, y) = self.robot_coordinates;

        match a {
            PickUpCan => match self.grid[x][y] {
                LocationValue::Can => 10.0,
                _ => -1.0,
            },
            _ => match self.crash(a) {
                true => -5.0,
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
      let row_strings: Vec<String> = self.grid.iter().map(|row| {
        let space_strings: Vec<String> = row.iter().map(|space| {
          return match space {
            LocationValue::Empty => "_".to_string(),
            LocationValue::Can => "C".to_string(),
            _ => "".to_string()
          }
        }).collect();

        space_strings.join(" ")
      }).collect();

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
    q_matrix: HashMap<Percept, HashMap<Action, f32>>,
}

impl Robot {
    pub fn new() -> Self {
        Robot {
            previous_choice: None,
            q_matrix: generate_q_matrix(),
        }
    }

    pub fn select_action(&mut self, p: &Percept) -> Action {
        let mut selected_action_pair = (Action::PickUpCan, 0.0);

        for (key, value) in &self.q_matrix[p] {
            if *value >= selected_action_pair.1 {
                println!("Choosing action {} over {}, {} > {}", key, selected_action_pair.0, value, selected_action_pair.1);
                selected_action_pair = (key.clone(), value.clone());
            }
        }

        self.previous_choice = Some((p.clone(), selected_action_pair.0.clone()));

        selected_action_pair.0
    }

    pub fn reward(&mut self, reward_amount: f32) {
        if let Some((p, a)) = &self.previous_choice {
            // TODO fix this unwrap nightmare
            // TODO Add epsilon and deeper update logic
            *self.q_matrix.get_mut(p).unwrap().get_mut(a).unwrap() += reward_amount;
        }
    }
}

fn generate_q_matrix() -> HashMap<Percept, HashMap<Action, f32>> {
    let mut out: HashMap<Percept, HashMap<Action, f32>> = HashMap::new();

    for n in 0..3 {
        for s in 0..3 {
            for e in 0..3 {
                for w in 0..3 {
                    for c in 0..3 {
                        let p = Percept {
                            north: LocationValue::from(n),
                            south: LocationValue::from(s),
                            east: LocationValue::from(e),
                            west: LocationValue::from(w),
                            current: LocationValue::from(c),
                        };

                        out.insert(p, empty_action_map());
                    }
                }
            }
        }
    }

    out
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
