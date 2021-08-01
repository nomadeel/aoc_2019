use crate::{
    solver::Solver,
    grid::Grid,
};
use std::{
    collections::{HashMap, HashSet, VecDeque, BinaryHeap, BTreeSet},
    cmp::Ordering,
    io::{BufRead, BufReader},
    fs::File,
};
use itertools::Itertools;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Tile {
    Wall,
    Empty,
    Void,
    Portal(String)
}

impl Tile {
    fn from_char(c: &char) -> Self {
        match c {
            '#' => Tile::Wall,
            '.' => Tile::Empty,
            // Technically there's portal, but this is an intermediate parsing function
            _ => Tile::Void,
        }
    }

    fn to_char(&self) -> char {
        match self {
            Tile::Wall => '#',
            Tile::Empty => '.',
            Tile::Void => ' ',
            Tile::Portal(_) => '@',
        }
    }
}

pub const MAZE_OFFSET_X: usize = 2;
pub const MAZE_OFFSET_Y: usize = 2;
pub const MAZE_WIDTH: usize = 113;
pub const MAZE_HEIGHT: usize = 111;

pub struct Problem;

impl Solver for Problem {
    type Input = PortalGraph;
    type Output1 = usize;
    type Output2 = usize;

    fn parse_input(&self, f: File) -> Self::Input {
        let lines_vec: Vec<String> = BufReader::new(f)
            .lines()
            .filter_map(|l| l.ok())
            .collect();

        // Insert characters from the file into a HashMap
        let mut char_grid: HashMap<(usize, usize), char> = HashMap::new();
        lines_vec.iter().enumerate().for_each(|(y, l)| {
            l.char_indices().for_each(|(x, c)| {
                char_grid.insert((x, y), c);
            });
        });

        // Parse the actual maze
        let mut grid: HashMap<(usize, usize), Tile> = HashMap::new();
        (0..MAZE_WIDTH).cartesian_product(0..MAZE_HEIGHT).for_each(|(x, y)| {
            grid.insert((x, y), Tile::from_char(char_grid.get(&(x + MAZE_OFFSET_X, y + MAZE_OFFSET_Y)).unwrap()));
        });

        // Populate the portals now
        populate_portals(&mut grid, &char_grid, (0, MAZE_WIDTH - 1, 0, MAZE_HEIGHT - 1), true);
        populate_portals(&mut grid, &char_grid, (28, 84, 28, 82), false);

        PortalGraph::from_grid(&grid)
    }

    fn solve_first(&self, input: &Self::Input) -> Self::Output1 {
        input.shortest_distance("AA0", "ZZ0")
    }

    fn solve_second(&self, input: &Self::Input) -> Self::Output2 {
        input.find_shortest_recursive_distance().unwrap()
    }
}

fn populate_portals(grid: &mut HashMap<(usize, usize), Tile>,
                    char_grid: &HashMap<(usize, usize), char>,
                    (left_x, right_x, top_y, bottom_y): (usize, usize, usize, usize),
                    outer: bool) {
    let mut portal_string = String::new();

    // Do the top line and bottom line
    for x in left_x..=right_x {
        if grid.get(&(x, top_y)) == Some(&Tile::Empty) {
            if outer == true {
                portal_string.push(*char_grid.get(&(MAZE_OFFSET_X + x, MAZE_OFFSET_Y + top_y - 2)).unwrap());
                portal_string.push(*char_grid.get(&(MAZE_OFFSET_X + x, MAZE_OFFSET_Y + top_y - 1)).unwrap());
                portal_string.push('0');
            } else {
                portal_string.push(*char_grid.get(&(MAZE_OFFSET_X + x, MAZE_OFFSET_Y + top_y + 1)).unwrap());
                portal_string.push(*char_grid.get(&(MAZE_OFFSET_X + x, MAZE_OFFSET_Y + top_y + 2)).unwrap());
                portal_string.push('1');
            }
            grid.insert((x, top_y), Tile::Portal(portal_string.clone()));
            portal_string.clear();
        }

        if grid.get(&(x, bottom_y)) == Some(&Tile::Empty) {
            if outer == true {
                portal_string.push(*char_grid.get(&(MAZE_OFFSET_X + x, MAZE_OFFSET_Y + bottom_y + 1)).unwrap());
                portal_string.push(*char_grid.get(&(MAZE_OFFSET_X + x, MAZE_OFFSET_Y + bottom_y + 2)).unwrap());
                portal_string.push('0');
            } else {
                portal_string.push(*char_grid.get(&(MAZE_OFFSET_X + x, MAZE_OFFSET_Y + bottom_y - 2)).unwrap());
                portal_string.push(*char_grid.get(&(MAZE_OFFSET_X + x, MAZE_OFFSET_Y + bottom_y - 1)).unwrap());
                portal_string.push('1');
            }
            grid.insert((x, bottom_y), Tile::Portal(portal_string.clone()));
            portal_string.clear();
        }
    }

    // Do the left line and the right line
    for y in top_y..=bottom_y {
        if grid.get(&(left_x, y)) == Some(&Tile::Empty) {
            if outer == true {
                portal_string.push(*char_grid.get(&(MAZE_OFFSET_X + left_x - 2, MAZE_OFFSET_Y + y)).unwrap());
                portal_string.push(*char_grid.get(&(MAZE_OFFSET_X + left_x - 1, MAZE_OFFSET_Y + y)).unwrap());
                portal_string.push('0');
            } else {
                portal_string.push(*char_grid.get(&(MAZE_OFFSET_X + left_x + 1, MAZE_OFFSET_Y + y)).unwrap());
                portal_string.push(*char_grid.get(&(MAZE_OFFSET_X + left_x + 2, MAZE_OFFSET_Y + y)).unwrap());
                portal_string.push('1');
            }
            grid.insert((left_x, y), Tile::Portal(portal_string.clone()));
            portal_string.clear();
        }

        if grid.get(&(right_x, y)) == Some(&Tile::Empty) {
            if outer == true {
                portal_string.push(*char_grid.get(&(MAZE_OFFSET_X + right_x + 1, MAZE_OFFSET_Y + y)).unwrap());
                portal_string.push(*char_grid.get(&(MAZE_OFFSET_X + right_x + 2, MAZE_OFFSET_Y + y)).unwrap());
                portal_string.push('0');
            } else {
                portal_string.push(*char_grid.get(&(MAZE_OFFSET_X + right_x - 2, MAZE_OFFSET_Y + y)).unwrap());
                portal_string.push(*char_grid.get(&(MAZE_OFFSET_X + right_x - 1, MAZE_OFFSET_Y + y)).unwrap());
                portal_string.push('1');
            }
            grid.insert((right_x, y), Tile::Portal(portal_string.clone()));
            portal_string.clear();
        }
    }
}

#[derive(PartialEq,Eq)]
struct DijkstraState {
    curr_node: String,
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

#[derive(PartialEq,Eq)]
struct RecursiveState {
    curr_node: String,
    cost: usize,
    depth: usize,
}

impl Ord for RecursiveState {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost)
    }
}

impl PartialOrd for RecursiveState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct PortalGraph {
    graph: HashMap<String, HashMap<String, usize>>,
}

impl PortalGraph {
    fn from_grid(grid: &HashMap<(usize, usize), Tile>) -> Self {
        let mut graph = HashMap::new();
        for (x, y) in (0..MAZE_WIDTH).cartesian_product(0..MAZE_HEIGHT) {
            if let Some(Tile::Portal(s)) = grid.get(&(x, y)) {
                let reachable_nodes = reachable_from(grid, (x, y));
                graph.insert(String::from(s), reachable_nodes);
            }
        }
        Self { graph: graph }
    }

    fn shortest_distance(&self, from: &str, to: &str) -> usize {
        let mut distances: HashMap<String, usize> = HashMap::new();
        for k in self.graph.keys() {
            distances.insert(k.clone(), usize::max_value());
        }
        distances.insert(from.to_string(), 0);
        let mut to_visit = BinaryHeap::new();
        let mut visited = HashSet::new();
        to_visit.push(DijkstraState { curr_node: from.to_string(), cost: 0 });

        while let Some(s) = to_visit.pop() {
            if s.curr_node == to {
                break;
            }
            visited.insert(s.curr_node.clone());
            let curr_distance = distances[&s.curr_node];
            let to_consider: Vec<_> = self.graph.get(&s.curr_node).unwrap().keys().filter(|k| {
                !visited.contains(k.as_str())
            }).collect();
            for n in to_consider {
                let new_distance = curr_distance + self.graph.get(&s.curr_node).unwrap().get(n).unwrap();
                if new_distance < *distances.get(n.as_str()).unwrap() {
                    distances.insert(n.clone(), new_distance);
                }
                // Push the other side of the portal
                if n != "ZZ0" {
                    visited.insert(n.clone());
                    if n.contains("0") {
                        let other_portal = n.replace("0", "1");
                        distances.insert(other_portal.clone(), distances[n.as_str()]);
                        to_visit.push(DijkstraState { curr_node: other_portal, cost: new_distance });
                    } else {
                        let other_portal = n.replace("1", "0");
                        distances.insert(other_portal.clone(), distances[n.as_str()]);
                        to_visit.push(DijkstraState { curr_node: other_portal, cost: new_distance });
                    }
                }
            }
        }

        // We don't need to go through the portal
        distances[to] - 1
    }

    fn find_shortest_recursive_distance(&self) -> Option<usize> {
        let mut to_evaluate = BinaryHeap::new();
        let start_state = RecursiveState { curr_node: String::from("AA0"), cost: 0, depth: 0 };
        to_evaluate.push(start_state);

        while let Some(curr_state) = to_evaluate.pop() {
            if curr_state.curr_node == "ZZ0" {
                // We don't need to go through the portal
                return Some(curr_state.cost - 1);
            }

            // Consider the portals that we can go through now
            let to_consider: Vec<_> = self.graph.get(&curr_state.curr_node).unwrap().keys().filter(|k| {
                match curr_state.depth {
                    0 => *k == "ZZ0"  || !k.contains("0"),
                    _ => !(*k == "AA0" || *k == "ZZ0"),
                }
            }).collect();

            for n in to_consider {
                let new_cost = curr_state.cost + *self.graph.get(&curr_state.curr_node).unwrap().get(n).unwrap();
                if *n == "ZZ0" {
                    to_evaluate.push(RecursiveState { curr_node: String::from("ZZ0"), cost: new_cost, depth: 0 });
                } else {
                    if n.contains("0") {
                        to_evaluate.push(RecursiveState { curr_node: n.replace("0", "1"), cost: new_cost, depth: curr_state.depth - 1 });
                    } else {
                        to_evaluate.push(RecursiveState { curr_node: n.replace("1", "0"), cost: new_cost, depth: curr_state.depth + 1 });
                    }
                }
            }
        }

        None
    }
}

fn reachable_from(grid: &HashMap<(usize, usize), Tile>, (from_x, from_y): (usize, usize)) -> HashMap<String, usize> {
    let mut reachable = HashMap::new();
    let mut visited = HashSet::new();
    let mut to_visit = VecDeque::new();
    visited.insert((from_x, from_y));

    to_visit.push_back((from_x, from_y, 1));

    while let Some((curr_x, curr_y, steps)) = to_visit.pop_front() {
        let neighbours = [(curr_x - 1, curr_y), (curr_x + 1, curr_y), (curr_x, curr_y - 1), (curr_x, curr_y + 1)];
        for n in neighbours {
            if let Some(tile) = grid.get(&n) {
                if !visited.contains(&n) {
                    visited.insert(n);
                    match tile {
                        Tile::Empty => to_visit.push_back((n.0, n.1, steps + 1)),
                        Tile::Portal(s) => { reachable.insert(s.clone(), steps + 1); }
                        _ => (),
                    }
                }
            }
        }
    }

    reachable
}
