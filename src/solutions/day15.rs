use crate::{
    solver::Solver,
    grid::Point,
    intcode::{parse_program, AsyncIO, IntCodeMachine}
};
use std::{
    collections::{HashMap, VecDeque},
    fs::File,
    thread,
    error::Error,
    sync::mpsc::{Sender, Receiver}
};

#[derive(Clone, Eq, PartialEq)]
pub enum Elem {
    Empty,
    Wall,
    OxygenSystem,
    Unknown,
}

impl Default for Elem {
    fn default() -> Self {
        Elem::Unknown
    }
}

impl Elem {
    fn to_char(&self) -> char {
        match self {
            Elem::Empty => '.',
            Elem::Wall => '#',
            Elem::OxygenSystem => '@',
            Elem::Unknown => '?',
        }
    }
}

pub struct Problem;

impl Solver for Problem {
    type Input = Navigator;
    type Output1 = u64;
    type Output2 = u64;

    fn parse_input(&self, f: File) -> Self::Input {
        let input = parse_program(f);
        let mut navigator = Navigator::new(&input);
        let _ = navigator.explore_map();
        navigator.print_map();
        navigator
    }

    fn solve_first(&self, input: &Self::Input) -> Self::Output1 {
        let mut curr_position = input.oxygen_position.unwrap();
        let origin = Point { x: 0, y: 0 };
        let mut steps = 0;
        while !curr_position.eq(&origin) {
            curr_position = *input.reachable_from.get(&curr_position).unwrap();
            steps += 1;
        }
        steps
    }

    fn solve_second(&self, input: &Self::Input) -> Self::Output2 {
        // Do BFS to find longest path from oxygen
        let mut distances: HashMap<Point, u64> = HashMap::new();
        let mut to_evaluate: VecDeque<Point> = VecDeque::new();
        let mut parent_map: HashMap<Point, Point>  = HashMap::new();
        let starting_point = input.oxygen_position.unwrap();

        to_evaluate.push_back(starting_point);
        distances.insert(starting_point, 0);
        parent_map.insert(starting_point, starting_point);

        while !to_evaluate.is_empty() {
            let curr_position = to_evaluate.pop_front().unwrap();
            let curr_distance = distances.get(&curr_position).unwrap().clone();
            // Push neighbours that haven't been evaluated yet
            let movements = vec!(Point { x: -1, y: 0 }, Point { x: 1, y: 0 }, Point { x: 0, y: -1 }, Point { x: 0, y: 1 });
            movements.iter().for_each(|m| {
                let neighbour = Point { x: m.x + curr_position.x, y: m.y + curr_position.y };
                match parent_map.get(&neighbour) {
                    Some(_) => (),
                    None => {
                        let neighbour_tile = input.map.get(&neighbour).unwrap_or(&Elem::Unknown);
                        if *neighbour_tile != Elem::Wall && *neighbour_tile != Elem::Unknown {
                            parent_map.insert(neighbour, curr_position);
                            to_evaluate.push_back(neighbour);
                            distances.insert(neighbour, curr_distance + 1);
                        }
                    }
                }
            });
        }

        *distances.values().max().unwrap()
    }
}

pub struct Navigator {
    _handle: thread::JoinHandle<()>,
    robot_position: Point,
    reachable_from: HashMap<Point, Point>,
    map: HashMap<Point, Elem>,
    oxygen_position: Option<Point>,
    tx_chan: Sender<i64>,
    rx_chan: Receiver<i64>,
}

impl Navigator {
    fn new(program: &Vec<i64>) -> Self {
        let (io, tx, rx) = AsyncIO::new();
        let mut machine = IntCodeMachine::new(program, io);
        let handle = thread::spawn(move || machine.run());
        let mut map: HashMap<Point, Elem> = HashMap::new();
        map.insert(Point { x: 0, y: 0 }, Elem::Empty);
        Self {
            _handle: handle,
            robot_position: Point { x: 0, y: 0 },
            reachable_from: HashMap::new(),
            map: map,
            oxygen_position: None,
            tx_chan: tx,
            rx_chan: rx,
        }
    }

    fn explore_map(&mut self) -> Result<(), Box<dyn Error>> {
        self.reachable_from.insert(self.robot_position, self.robot_position);
        let mut to_visit: VecDeque<Point> = VecDeque::new();
        self.add_visitable_neighbours(&mut to_visit);

        while !to_visit.is_empty() {
            let next_position = to_visit.pop_front().unwrap();
            if !is_direct_neighbour(&self.robot_position, &next_position) {
                let point_before = self.reachable_from.get(&next_position).unwrap();
                let _ = self.backtrack(point_before);
                self.robot_position = *self.reachable_from.get(&next_position).unwrap();
            }

            let movement = movement_from(&self.robot_position, &next_position);
            //println!("Moving {} to {:?}", movement, next_position);
            self.tx_chan.send(movement)?;

            let status = self.rx_chan.recv()?;
            //println!("Found {} at {:?}", status, next_position);
            match status {
                0 => { self.map.insert(next_position, Elem::Wall); },
                1 => { self.map.insert(next_position, Elem::Empty); self.robot_position = next_position; },
                2 => {
                    self.map.insert(next_position, Elem::OxygenSystem);
                    self.robot_position = next_position;
                    self.oxygen_position = Some(self.robot_position.clone());
                }
                _ => panic!("Invalid status code!")
            }
            self.add_visitable_neighbours(&mut to_visit);
        }

        Ok(())
    }

    fn backtrack(&self, point: &Point) -> Result<(), Box<dyn Error>> {
        // Simple case
        if is_direct_neighbour(point, &self.robot_position) {
            let movement = movement_from(&self.robot_position, point);
            self.tx_chan.send(movement)?;
            let output = self.rx_chan.recv().ok().unwrap();
            assert!(output == 1, "output = {}", output);
            return Ok(());
        }

        let mut curr_position = &self.robot_position;
        let origin = Point { x: 0, y: 0 };
        let mut movements_to_origin: Vec<i64> = vec!();

        // Backtrack robot to origin
        while !curr_position.eq(&origin) {
            let step_back = self.reachable_from.get(&curr_position).unwrap();
            let movement = movement_from(curr_position, step_back);
            movements_to_origin.push(movement);
            curr_position = step_back;
        }
        movements_to_origin.iter().for_each(|m| { self.tx_chan.send(*m).ok().unwrap();
                                                  let output = self.rx_chan.recv().ok().unwrap();
                                                  assert!(output == 1, "output = {}", output);
        });

        // Build steps from origin to point backwards
        let mut movements_to_point: Vec<i64> = vec!();
        curr_position = point;
        while !curr_position.eq(&origin) {
            let step_back = self.reachable_from.get(&curr_position).unwrap();
            let movement = movement_from(step_back, curr_position);
            movements_to_point.push(movement);
            curr_position = step_back;
        }
        movements_to_point.reverse();
        movements_to_point.iter().for_each(|m| { self.tx_chan.send(*m).ok().unwrap();
                                                 let output = self.rx_chan.recv().ok().unwrap();
                                                 assert!(output == 1, "output = {}", output);
        });

        Ok(())
    }

    fn add_visitable_neighbours(&mut self, to_visit: &mut VecDeque<Point>) {
        let movements = vec!(Point { x: -1, y: 0 }, Point { x: 1, y: 0 }, Point { x: 0, y: -1 }, Point { x: 0, y: 1 });
        let potential_points: Vec<_> = movements
            .iter()
            .map(|m| { Point { x: m.x + self.robot_position.x, y: m.y + self.robot_position.y } })
            .collect();
        potential_points.iter().for_each(|p| {
            match self.reachable_from.get(p) {
                Some(_) => (),
                None => {
                    to_visit.push_back(p.clone());
                    self.reachable_from.insert(p.clone(), self.robot_position.clone());
                }
            }
        })
    }

    fn print_map(&self) {
        let min_x = self.map.keys().min_by_key(|p| p.x).unwrap().x;
        let min_y = self.map.keys().min_by_key(|p| p.y).unwrap().y;
        let max_x = self.map.keys().max_by_key(|p| p.x).unwrap().x;
        let max_y = self.map.keys().max_by_key(|p| p.y).unwrap().y;

        for y in (min_y..=max_y).rev() {
            for x in min_x..=max_x {
                if self.robot_position.x == x && self.robot_position.y == y {
                    print!("R");
                } else if x == 0 && y == 0 {
                    print!("O");
                } else {
                    print!("{}", self.map.get(&Point { x, y }).unwrap_or(&Elem::Unknown).to_char());
                }
            }
            println!();
        }
    }
}


fn is_direct_neighbour(robot_position: &Point, position: &Point) -> bool {
    let vector = (position.x - robot_position.x, position.y - robot_position.y);
    match vector {
        (-1, 0) => true,
        (1, 0) => true,
        (0, -1) => true,
        (0, 1) => true,
        _ => false,
    }
}

fn movement_from(from: &Point, to: &Point) -> i64 {
    let vector = (to.x - from.x, to.y - from.y);
    match vector {
        // Go left
        (-1, 0) => 3,
        // Go right
        (1, 0) => 4,
        // Go down
        (0, -1) => 2,
        // Go up
        (0, 1) => 1,
        _ => panic!("'from' {:?} and 'to' {:?} are not neighbours!", from, to)
    }
}
