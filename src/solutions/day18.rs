use crate::{
    solver::Solver,
    grid::Grid,
};
use std::{
    collections::{HashMap, HashSet, VecDeque, BinaryHeap, BTreeSet},
    cmp::Ordering,
    convert::TryFrom,
    fs::File,
};
use itertools::Itertools;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Tile {
    Wall,
    Empty,
    Node(char), // Could be key or a door
}

impl TryFrom<u8> for Tile {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'#' => Ok(Tile::Wall),
            b'.' => Ok(Tile::Empty),
            _ => {
                let character = char::from_u32(value as u32).unwrap();
                Ok(Tile::Node(character))
            }
        }
    }
}

impl Default for Tile {
    fn default() -> Self {
        Tile::Empty
    }
}

pub struct Problem;

impl Solver for Problem {
    type Input = Grid<Tile>;
    type Output1 = usize;
    type Output2 = usize;

    fn parse_input(&self, f: File) -> Self::Input {
        let grid: Grid<Tile> = Grid::from_reader(f).unwrap();
        grid
    }

    fn solve_first(&self, input: &Self::Input) -> Self::Output1 {
        let maze = MazeGraph::from_grid(&input);
        let result = maze.search(&['@'].to_vec());
        assert!(result != None);
        result.unwrap()
    }

    fn solve_second(&self, input: &Self::Input) -> Self::Output2 {
        let mut grid = input.clone();
        // Close off the centre of the maze into four partitions
        const ROBOT_LOCATION: (usize, usize) = (40, 40);
        assert!(*grid.get(ROBOT_LOCATION).unwrap() == Tile::Node('@'));
        let neighbour_vectors: Vec<(i64, i64)> = vec!((-1, 0), (1, 0), (0, -1), (0, 1));
        let robot_neighbours: Vec<_> = neighbour_vectors.iter().map(|(x, y)| {
            ((x + ROBOT_LOCATION.0 as i64) as usize, (y + ROBOT_LOCATION.1 as i64) as usize)
        }).collect();
        grid.set(ROBOT_LOCATION, Tile::Wall);
        (0..4).for_each(|i| grid.set(robot_neighbours[i], Tile::Wall));
        // Add the new robots in
        let new_robot_nodes = vec!(Tile::Node('@'), Tile::Node('!'), Tile::Node('$'), Tile::Node('%'));
        let new_robot_vectors: Vec<(i64, i64)> = vec!((-1, 1), (-1, -1), (1, 1), (1, -1));
        let new_robots: Vec<_> = new_robot_vectors.iter().map(|(x, y)| {
            ((x + ROBOT_LOCATION.0 as i64) as usize, (y + ROBOT_LOCATION.1 as i64) as usize)
        }).collect();
        (0..4).for_each(|i| grid.set(new_robots[i], new_robot_nodes[i].clone()));
        let maze = MazeGraph::from_grid(&grid);
        let result = maze.search(&['@', '!', '$', '%'].to_vec());
        assert!(result != None);
        result.unwrap()
    }
}

#[derive(PartialEq,Eq)]
struct State {
    curr_nodes: Vec<char>,
    keys_held: BTreeSet<char>,
    steps_taken: usize
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other.steps_taken.cmp(&self.steps_taken).then(self.keys_held.len().cmp(&other.keys_held.len()))
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &State) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(PartialEq,Eq)]
struct DijkstraState {
    curr_node: char,
    cost: usize
}

impl Ord for DijkstraState {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost)
    }
}

impl PartialOrd for DijkstraState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct MazeGraph {
    graph: HashMap<char, HashMap<char, usize>>,
}

impl MazeGraph {
    fn from_grid(grid: &Grid<Tile>) -> Self {
        let mut graph = HashMap::new();
        for (x, y) in (0..grid.w).cartesian_product(0..grid.h) {
            if let Some(Tile::Node(c)) = grid.get((x, y)) {
                let reachable_nodes = reachable_from(grid, (x, y));
                graph.insert(*c, reachable_nodes);
            }
        }
        Self { graph }
    }

    // Dijkstra's algorithm basically
    fn search(&self, start: &Vec<char>) -> Option<usize> {
        const TOTAL_KEY_COUNT: usize = 26;
        let mut to_evaluate = BinaryHeap::new();
        let mut best_distances: HashMap<(Vec<char>, BTreeSet<char>), usize> = HashMap::new();
        let mut search_cached: HashMap<(char, BTreeSet<char>), Vec<(char, usize)>> = HashMap::new();
        let start_state = State { curr_nodes: start.clone(), keys_held: BTreeSet::new(), steps_taken: 0 };
        to_evaluate.push(start_state);

        while let Some(curr_state) = to_evaluate.pop() {
            if curr_state.keys_held.len() == TOTAL_KEY_COUNT {
                return Some(curr_state.steps_taken);
            }

            let curr_map_key = (curr_state.curr_nodes.clone(), curr_state.keys_held.clone());

            // Check if we've seen a better solution already
            let last_best = best_distances.get(&curr_map_key).unwrap_or(&usize::max_value());
            if curr_state.steps_taken > *last_best {
                continue;
            }

            for (i, node) in curr_state.curr_nodes.iter().enumerate() {
                // Now do a search on the keys that we can get
                let search_cache = search_cached.entry((*node, curr_state.keys_held.clone())).or_insert_with(|| {
                    self.reachable_keys(node, &curr_state.keys_held)
                });

                // Evaluate all the keys that we can get
                for (next_key, steps) in search_cache {
                    let mut next_keys = curr_state.keys_held.clone();
                    next_keys.insert(*next_key);
                    let next_steps = *steps + curr_state.steps_taken;
                    let mut next_nodes = curr_state.curr_nodes.clone();
                    next_nodes[i] = *next_key;

                    let best_so_far = best_distances.entry((next_nodes.clone(), next_keys.clone())).or_insert(usize::max_value());

                    if next_steps < *best_so_far {
                        // Replace it with what we have found
                        *best_so_far = next_steps;

                        // Add this to the evaluation queue
                        let new_state = State {
                            curr_nodes: next_nodes,
                            keys_held: next_keys,
                            steps_taken: next_steps
                        };

                        to_evaluate.push(new_state);
                    }
                }
            }
        }

        None
    }

    // Dijkstra's algorithm
    fn reachable_keys(&self, from: &char, keys_held: &BTreeSet<char>) -> Vec<(char, usize)> {
        // Distances from 'from' to the node
        let mut distances: HashMap<char, usize> = HashMap::new();
        for k in self.graph.keys() {
            distances.insert(*k, usize::max_value());
        }
        distances.insert(*from, 0);
        let mut to_visit = BinaryHeap::new();
        let mut reachable_keys = HashSet::new();
        let mut visited = HashSet::new();
        to_visit.push(DijkstraState { curr_node: *from, cost: 0 });

        while let Some(s) = to_visit.pop() {
            visited.insert(s.curr_node);
            let curr_distance = distances[&s.curr_node];
            let to_consider: Vec<_> = self.graph.get(&s.curr_node).unwrap().keys().filter(|k| !visited.contains(k)).collect();
            for n in to_consider {
                // Check if we can pass through the door, if it is one
                if n.is_uppercase() && !keys_held.contains(&n.to_ascii_lowercase()) {
                    continue;
                }

                // Add this to the keys that we can get if we haven't gotten it
                if n.is_lowercase() && !keys_held.contains(n) {
                    reachable_keys.insert(*n);
                }

                let new_distance = curr_distance + self.graph.get(&s.curr_node).unwrap().get(n).unwrap();
                if new_distance < *distances.get(n).unwrap() {
                    distances.insert(*n, new_distance);
                }
                to_visit.push(DijkstraState { curr_node: *n, cost: new_distance });
            }
        }

        let result = reachable_keys.iter().map(|k| (*k, *distances.get(k).unwrap())).collect();
        result
    }
}

fn reachable_from(grid: &Grid<Tile>, (from_x, from_y): (usize, usize)) -> HashMap<char, usize> {
    let mut reachable = HashMap::new();
    let mut visited = HashSet::new();
    let mut to_visit = VecDeque::new();
    visited.insert((from_x, from_y));

    to_visit.push_back((from_x, from_y, 0));

    while let Some((curr_x, curr_y, steps)) = to_visit.pop_front() {
        let neighbours = [(curr_x - 1, curr_y), (curr_x + 1, curr_y), (curr_x, curr_y - 1), (curr_x, curr_y + 1)];
        for n in neighbours {
            if let Some(tile) = grid.get(n) {
                if !visited.contains(&n) {
                    visited.insert(n);
                    match tile {
                        Tile::Wall => (),
                        Tile::Empty => to_visit.push_back((n.0, n.1, steps + 1)),
                        Tile::Node(c) => { reachable.insert(*c, steps + 1); }
                    }
                }
            }
        }
    }

    reachable
}
