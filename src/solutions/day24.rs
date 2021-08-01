use crate::{
    solver::Solver,
};
use std::{
    collections::{HashMap, HashSet},
    io::{BufRead, BufReader},
    fs::File,
};
use itertools::Itertools;

#[derive(Clone, Eq, PartialEq)]
pub enum Tile {
    Empty,
    Bug,
}

const GRID_WIDTH: usize = 5;
const GRID_HEIGHT: usize = 5;
const ITERATIONS: usize = 200;

impl Tile {
    fn from_char(c: char) -> Self {
        match c {
            '.' => Tile::Empty,
            '#' => Tile::Bug,
            v => panic!("Invalid character: {}", v)
        }
    }

    fn to_char(&self) -> char {
        match self {
            Tile::Empty => '.',
            Tile::Bug => '#',
        }
    }
}

pub struct Problem;

impl Solver for Problem {
    type Input = BugGrid;
    type Output1 = u64;
    type Output2 = usize;

    fn parse_input(&self, f: File) -> Self::Input {
        BugGrid::from_file(f)
    }

    fn solve_first(&self, input: &Self::Input) -> Self::Output1 {
        let mut bug_grid = (*input).clone();
        // We can effectively reduce the grid into a number
        let mut seen_grids: HashSet<u64> = HashSet::new();
        seen_grids.insert(bug_grid.biodiversity_rating());
        loop {
            bug_grid.advance();
            let rating = bug_grid.biodiversity_rating();
            if seen_grids.contains(&rating) {
                return rating;
            }
            seen_grids.insert(rating);
        }
    }

    fn solve_second(&self, input: &Self::Input) -> Self::Output2 {
        let mut recursive_grid = RecursiveBugGrid::new(ITERATIONS);
        // Initialise the starting grid
        (0..GRID_HEIGHT).cartesian_product(0..GRID_WIDTH).for_each(|(y, x)| {
            let tile = input.grid.get(&(x as i64, y as i64)).unwrap();
            recursive_grid.grids.get_mut(&(ITERATIONS + 1)).unwrap().insert((x as i64, y as i64), tile.clone());
        });
        for _ in 0..ITERATIONS {
            recursive_grid.advance();
        }
        recursive_grid.grids.values().fold(0, |acc, g| {
            acc + g.values().filter(|t| **t == Tile::Bug).count()
        })
    }
}

struct RecursiveBugGrid {
    grids: HashMap<usize, HashMap<(i64, i64), Tile>>,
    num_grids: usize,
}

impl RecursiveBugGrid {
    fn new(iterations: usize) -> Self {
        let mut grids = HashMap::new();
        (0..iterations * 2 + 3).for_each(|i| {
            let mut grid = HashMap::new();
            (0..GRID_HEIGHT).cartesian_product(0..GRID_HEIGHT).for_each(|(y, x)| {
                grid.insert((x as i64, y as i64), Tile::Empty);
            });
            grids.insert(i, grid);
        });
        Self { grids, num_grids: iterations * 2 + 3 }
    }

    fn advance(&mut self) {
        let mut new_grid = RecursiveBugGrid::new(ITERATIONS);
        for level in 0..self.num_grids - 2 {
            for (y, x) in (0..GRID_HEIGHT).cartesian_product(0..GRID_WIDTH) {
                if (y, x) == (2, 2) {
                    continue;
                }

                let num_neighbour_bugs = self.count_neighbour_bugs(level, (x as i64, y as i64));
                let current_tile = self.grids.get(&level).unwrap().get(&(x as i64, y as i64)).unwrap();
                match current_tile {
                    Tile::Bug => {
                        if num_neighbour_bugs != 1 {
                            new_grid.grids.get_mut(&level).unwrap().insert((x as i64, y as i64), Tile::Empty);
                        } else {
                            new_grid.grids.get_mut(&level).unwrap().insert((x as i64, y as i64), Tile::Bug);
                        }
                    }
                    Tile::Empty => {
                        if 1 <= num_neighbour_bugs && num_neighbour_bugs <= 2 {
                            new_grid.grids.get_mut(&level).unwrap().insert((x as i64, y as i64), Tile::Bug);
                        } else {
                            new_grid.grids.get_mut(&level).unwrap().insert((x as i64, y as i64), Tile::Empty);
                        }
                    }
                }
            }
        }
        self.grids = new_grid.grids;
    }

    /* For reference:
     *
     *      |     |         |     |
     *   1  |  2  |    3    |  4  |  5
     *      |     |         |     |
     * -----+-----+---------+-----+-----
     *      |     |         |     |
     *   6  |  7  |    8    |  9  |  10
     *      |     |         |     |
     * -----+-----+---------+-----+-----
     *      |     |A|B|C|D|E|     |
     *      |     |-+-+-+-+-|     |
     *      |     |F|G|H|I|J|     |
     *      |     |-+-+-+-+-|     |
     *  11  | 12  |K|L|?|N|O|  14 |  15
     *      |     |-+-+-+-+-|     |
     *      |     |P|Q|R|S|T|     |
     *      |     |-+-+-+-+-|     |
     *      |     |U|V|W|X|Y|     |
     * -----+-----+---------+-----+-----
     *      |     |         |     |
     *  16  | 17  |    18   |  19 |  20
     *      |     |         |     |
     * -----+-----+---------+-----+-----
     *      |     |         |     |
     *  21  | 22  |    23   |  24 |  25
     *      |     |         |     |
     */
    fn count_neighbour_bugs(&self, level: usize, pos: (i64, i64)) -> usize {
        let neighbours: Vec<(i64, i64)> = vec!((0, 1), (0, -1), (1, 0), (-1, 0));
        let mut num_bugs = 0;
        for n in neighbours {
            let curr_pos = ((pos.0 + n.0), (pos.1 + n.1));
            match curr_pos {
                (2, 2) => num_bugs += self.count_neighbours_below(level, pos),
                // A's left from diagram
                (-1, _) => {
                    let level_above = self.grids.get(&(level + 1)).unwrap();
                    if *level_above.get(&(1, 2)).unwrap() == Tile::Bug {
                        num_bugs += 1;
                    }
                },
                // A's up from diagram
                (_, -1) => {
                    let level_above = self.grids.get(&(level + 1)).unwrap();
                    if *level_above.get(&(2, 1)).unwrap() == Tile::Bug {
                        num_bugs += 1;
                    }
                },
                // Y's right from diagram
                (5, _) => {
                    let level_above = self.grids.get(&(level + 1)).unwrap();
                    if *level_above.get(&(3, 2)).unwrap() == Tile::Bug {
                        num_bugs += 1;
                    }
                }
                // Y's down from diagram
                (_, 5) => {
                    let level_above = self.grids.get(&(level + 1)).unwrap();
                    if *level_above.get(&(2, 3)).unwrap() == Tile::Bug {
                        num_bugs += 1;
                    }
                }
                (x, y) => {
                    if *self.grids.get(&level).unwrap().get(&(x, y)).unwrap() == Tile::Bug {
                        num_bugs += 1;
                    }
                }
            }
        }
        num_bugs
    }

    fn count_neighbours_below(&self, level: usize, (x, y): (i64, i64)) -> usize {
        let level_below = match level {
            0 => self.grids.get(&(self.num_grids - 1)).unwrap(),
            _ => self.grids.get(&(level - 1)).unwrap()
        };

        match (x, y) {
            // 8 from diagram
            (2, 1) => (0..5).fold(0, |acc, x| {
                if *level_below.get(&(x, 0)).unwrap() == Tile::Bug {
                    acc + 1
                } else {
                    acc
                }
            }),
            // 12 from diagram
            (1, 2) => (0..5).fold(0, |acc, y| {
                if *level_below.get(&(0, y)).unwrap() == Tile::Bug {
                    acc + 1
                } else {
                    acc
                }
            }),
            // 14 from diagram
            (3, 2) => (0..5).fold(0, |acc, y| {
                if *level_below.get(&(4, y)).unwrap() == Tile::Bug {
                    acc + 1
                } else {
                    acc
                }
            }),
            // 18 from diagram
            (2, 3) => (0..5).fold(0, |acc, x| {
                if *level_below.get(&(x, 4)).unwrap() == Tile::Bug {
                    acc + 1
                } else {
                    acc
                }
            }),
            _ => 0,
        }
    }
}

#[derive (Clone)]
pub struct BugGrid {
    grid: HashMap<(i64, i64), Tile>,
}

impl BugGrid {
    fn from_file(f: File) -> Self {
        let mut output: HashMap<(i64, i64), Tile> = HashMap::new();
        BufReader::new(f)
            .lines()
            .filter_map(|l| l.ok())
            .enumerate()
            .for_each(|(y, l)| {
                l.char_indices().for_each(|(x, c)| {
                    output.insert((x as i64, y as i64), Tile::from_char(c));
                });
            });
        Self { grid: output }
    }

    fn biodiversity_rating(&self) -> u64 {
        let mut rating = 0;
        (0..GRID_HEIGHT).cartesian_product(0..GRID_WIDTH).enumerate().for_each(|(i, (y, x))| {
            match self.grid.get(&(x as i64, y as i64)).unwrap() {
                Tile::Bug => {
                    rating += 1 << i;
                },
                Tile::Empty => (),
            }
        });
        rating
    }

    fn advance(&mut self) {
        let mut new_grid = HashMap::new();
        for (y, x) in (0..GRID_HEIGHT).cartesian_product(0..GRID_WIDTH) {
            let neighbours: Vec<(i64, i64)> = vec!((0, 1), (0, -1), (1, 0), (-1, 0));
            let bugs_count = neighbours.iter().map(|n| {
                self.grid.get(&(x as i64 + n.0, y as i64 + n.1)).unwrap_or(&Tile::Empty)
            }).filter(|t| **t == Tile::Bug).count();
            let current_tile = self.grid.get(&(x as i64, y as i64)).unwrap();
            match current_tile {
                Tile::Bug => {
                    if bugs_count != 1 {
                        new_grid.insert((x as i64, y as i64), Tile::Empty);
                    } else {
                        new_grid.insert((x as i64, y as i64), Tile::Bug);
                    }
                }
                Tile::Empty => {
                    if 1 <= bugs_count && bugs_count <= 2 {
                        new_grid.insert((x as i64, y as i64), Tile::Bug);
                    } else {
                        new_grid.insert((x as i64, y as i64), Tile::Empty);
                    }
                }
            }
        }
        self.grid = new_grid;
    }
}

