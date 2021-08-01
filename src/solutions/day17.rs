use crate::{
    solver::Solver,
    grid::Point,
    intcode::{parse_program, AsyncIO, IntCodeMachine}
};
use std::{
    collections::HashMap,
    fs::File,
    thread,
    error::Error,
    sync::mpsc::{Sender, Receiver}
};

pub struct Problem;

impl Solver for Problem {
    type Input = Vec<i64>;
    type Output1 = u64;
    type Output2 = i64;

    fn parse_input(&self, f: File) -> Self::Input {
        parse_program(f)
    }

    fn solve_first(&self, input: &Self::Input) -> Self::Output1 {
        let mut scaffolding = Scaffolding::new(input);
        let _ = scaffolding.populate_grid();
        //scaffolding.print();

        let max_x = scaffolding.grid.keys().max_by_key(|p| p.x).unwrap().x;
        let max_y = scaffolding.grid.keys().max_by_key(|p| p.y).unwrap().y;
        let scaffold_points: Vec<_> = scaffolding.grid.iter().filter(|(_, e)| **e == Elem::Scaffold).map(|(p, _)| p).collect();
        let intersections: Vec<Point> = scaffold_points.iter().filter(|p| {
            let neighbours = vec!(Point { x: p.x - 1, y: p.y }, Point { x: p.x + 1, y: p.y }, Point { x: p.x, y: p.y - 1 }, Point { x: p.x, y: p.y + 1 });
            neighbours.iter().map(|n| scaffolding.grid.get(n).unwrap_or(&Elem::Empty)).all(|e| *e == Elem::Scaffold)
        }).map(|p| **p).collect();
        let alignment_sum = intersections.iter().fold(0, |acc, p: &Point| acc + (p.x * p.y) );
        let _ = scaffolding.handle.join();
        alignment_sum as u64
    }

    fn solve_second(&self, input: &Self::Input) -> Self::Output2 {
        let mut scaffolding = Scaffolding::new(&input);
        let _ = scaffolding.populate_grid();

        let path = scaffolding.find_path();
        // Compact the string and then find the three patterns TODO, it looks like a dynamic programming problem
        let compacted_path: String = path.group_by(|a, b| a == b).map(|g| {
            if g.len() > 1 {
                g.len().to_string()
            } else {
                String::from(g[0])
            }
        }).collect();
        // Found by hand
        let patterns = vec!("L,6,L,4,R,8\n", "R,8,L,6,L,4,L,10,R,8\n", "L,4,R,4,L,4,R,8\n");
        let movement_routine = "A,B,A,C,B,C,B,C,A,B\n";
        // Modify input so that we can move the robot
        let mut modified_input = input.clone();
        modified_input[0] = 2;
        let mut scaffolding = Scaffolding::new(&modified_input);
        let _ = scaffolding.print_map();
        let _ = scaffolding.input_routines(&patterns, &movement_routine);
        let _ = scaffolding.print_map();
        let _ = scaffolding.rx_chan.recv().unwrap();
        let output = scaffolding.rx_chan.recv().unwrap();
        output
    }
}

#[derive(Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right
}

impl Direction {
    fn to_movement(&self) -> Point {
        match self {
            Direction::Up => Point { x: 0, y: -1 },
            Direction::Down => Point { x: 0, y: 1 },
            Direction::Left => Point { x: -1, y: 0 },
            Direction::Right => Point { x: 1, y: 0 },
        }
    }

    /* Gets the 'movement vectors' of a turn to the left and right, the ups and
     * downs for the left and right turns should be the other way around
     */
    fn turns(&self) -> (Point, Point) {
        match self {
            Direction::Up => (Point { x: -1, y: 0 }, Point { x: 1, y: 0 }),
            Direction::Down => (Point { x: 1, y: 0 }, Point { x: -1, y: 0 }),
            Direction::Left => (Point { x: 0, y: 1 }, Point { x: 0, y: -1 }),
            Direction::Right => (Point { x: 0, y: -1 }, Point { x: 0, y: 1 }),
        }
    }

    fn turn(&self, turn: &Direction) -> Self {
        match self {
            Direction::Up => match turn {
                Direction::Left => Direction::Left,
                Direction::Right => Direction::Right,
                _ => panic!("Invalid turn!")
            }
            Direction::Down => match turn {
                Direction::Left => Direction::Right,
                Direction::Right => Direction::Left,
                _ => panic!("Invalid turn!")
            }
            Direction::Left => match turn {
                Direction::Left => Direction::Down,
                Direction::Right => Direction::Up,
                _ => panic!("Invalid turn!")
            }
            Direction::Right => match turn {
                Direction::Left => Direction::Up,
                Direction::Right => Direction::Down,
                _ => panic!("Invalid turn!")
            }
        }
    }
}

#[derive(Clone, PartialEq)]
enum Elem {
    Empty,
    Scaffold,
    Robot
}

impl Elem {
    fn from_char(c: char) -> Self {
        match c {
            '.' => Elem::Empty,
            '#' => Elem::Scaffold,
            '^' => Elem::Robot,
            '<' => Elem::Robot,
            'v' => Elem::Robot,
            '>' => Elem::Robot,
            _ => panic!("Invalid character!"),
        }
    }

    fn to_char(&self, d: &Direction) -> char {
        match self {
            Elem::Empty => '.',
            Elem::Scaffold => '#',
            Elem::Robot => match d {
                Direction::Up => '^',
                Direction::Down => 'v',
                Direction::Left => '<',
                Direction::Right => '>',
            }
        }
    }
}

struct Scaffolding {
    handle: thread::JoinHandle<()>,
    robot_position: Point,
    robot_direction: Direction,
    grid: HashMap<Point, Elem>,
    tx_chan: Sender<i64>,
    rx_chan: Receiver<i64>,
}

impl Scaffolding {
    fn new(program: &Vec<i64>) -> Self {
        let (io, tx, rx) = AsyncIO::new();
        let mut machine = IntCodeMachine::new(program, io);
        let handle = thread::spawn(move || machine.run());
        let grid: HashMap<Point, Elem>  = HashMap::new();
        Self {
            handle: handle,
            robot_position: Point { x: 0, y: 0 },
            robot_direction: Direction::Up,
            grid: grid,
            tx_chan: tx,
            rx_chan: rx,
        }
    }

    fn populate_grid(&mut self) -> Result<(), Box<dyn Error>> {
        let mut curr_point = Point { x: 0, y: 0 };
        loop {
            let output = self.rx_chan.recv()?;
            if output == 10 {
                curr_point.x = 0;
                curr_point.y += 1;
                continue;
            }
            let character = char::from_u32(output as u32).unwrap();
            match character {
                '^' => { self.robot_position = curr_point; self.robot_direction = Direction::Up },
                'v' => { self.robot_position = curr_point; self.robot_direction = Direction::Down },
                '<' => { self.robot_position = curr_point; self.robot_direction = Direction::Left },
                '>' => { self.robot_position = curr_point; self.robot_direction = Direction::Right },
                _ => ()
            }
            let elem = Elem::from_char(character);
            self.grid.insert(curr_point, elem);
            curr_point.x += 1;
        }
    }

    fn print_map(&self) -> Result<(), Box<dyn Error>> {
        const WIDTH: usize = 43 + 1; // for 'newline'
        const HEIGHT: usize = 39;
        let map_size = WIDTH * HEIGHT;
        for _ in 0..map_size {
            let output = self.rx_chan.recv()?;
            let character = char::from_u32(output as u32).unwrap();
            print!("{}", character);
        }
        Ok(())
    }

    /*
     * Tricks: It seems that you can basically ignore intersections and just
     * move forward on a straight line until you have to make a turn. At this
     * point, there should only be one turn available to you.
     */
    fn find_path(&mut self) -> Vec<char> {
        let mut output = vec!();
        let mut is_deadend = false;

        while !is_deadend {
            // Move forward if we can
            let movement = self.robot_direction.to_movement();
            let forward_position = Point { x: self.robot_position.x + movement.x, y: self.robot_position.y + movement.y };
            let forward_elem = self.grid.get(&forward_position);
            let turn_movements = self.robot_direction.turns();
            let left_position = Point { x: self.robot_position.x + turn_movements.0.x,
                                        y: self.robot_position.y + turn_movements.0.y };
            let right_position = Point { x: self.robot_position.x + turn_movements.1.x,
                                         y: self.robot_position.y + turn_movements.1.y };
            match forward_elem {
                Some(Elem::Scaffold) => { self.robot_position = forward_position.clone(); output.push('F') },
                // Either empty space or 'None'
                _ => {
                    // So we have to turn either left or right
                    if self.grid.get(&left_position) == Some(&Elem::Scaffold) {
                        self.robot_direction = self.robot_direction.turn(&Direction::Left);
                        output.push('L');
                    } else if self.grid.get(&right_position) == Some(&Elem::Scaffold) {
                        self.robot_direction = self.robot_direction.turn(&Direction::Right);
                        output.push('R')
                    }
                }
            }
            // Check if we're at a dead end now, remember no backtracking
            let neighbours = vec!(forward_position, left_position, right_position);
            is_deadend = !neighbours.iter().map(|p| self.grid.get(&p).unwrap_or(&Elem::Empty)).any(|e| *e == Elem::Scaffold);
        }

        output
    }

    fn input_routines(&mut self, functions: &Vec<&str>, routine: &str) -> Result<(), Box<dyn Error>> {
        // I could be using 'for_each()' but these can fail
        // Send the routine
        for c in routine.chars() {
            self.tx_chan.send(c as i64)?;
        }
        // Now send the fucntions
        for f in functions {
            for c in f.chars() {
                self.tx_chan.send(c as i64)?;
            }
        }
        // Tell the robot we don't want continuous video feed
        self.tx_chan.send('n' as i64)?;
        self.tx_chan.send('\n' as i64)?;
        // Print out the input prompts and the last newline
        for _ in 0..7 {
            let mut output = 0;
            while output != 10 {
                output = self.rx_chan.recv()?;
                let character = char::from_u32(output as u32).unwrap();
                print!("{}", character);
            }
        }
        Ok(())
    }

    fn print(&self) {
        let min_x = self.grid.keys().min_by_key(|p| p.x).unwrap().x;
        let min_y = self.grid.keys().min_by_key(|p| p.y).unwrap().y;
        let max_x = self.grid.keys().max_by_key(|p| p.x).unwrap().x;
        let max_y = self.grid.keys().max_by_key(|p| p.y).unwrap().y;

        // Don't reverse it since Y is from distance from top
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                print!("{}", self.grid.get(&Point{ x: x, y: y }).unwrap().to_char(&self.robot_direction));
            }
            println!();
        }
    }
}
