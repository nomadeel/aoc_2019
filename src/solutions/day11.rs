use crate::{
    intcode::{parse_program, IntCodeMachine, AsyncIO},
    solver::Solver,
    grid::Point,
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
    type Output1 = usize;
    type Output2 = String;

    fn parse_input(&self, f: File) -> Self::Input {
        parse_program(f)
    }

    fn solve_first(&self, input: &Self::Input) -> Self::Output1 {
        let mut board = Board::new();
        let mut painting_robot = PaintingRobot::new(input);

        loop {
            if painting_robot.paint(&mut board).is_err() {
                break;
            }
        }

        painting_robot.wait();
        board.panels.len()
    }

    fn solve_second(&self, input: &Self::Input) -> Self::Output2 {
        let mut board = Board::new();
        let mut painting_robot = PaintingRobot::new(input);
        board.paint(&Point{ x: 0, y: 0 }, &PaintColour::White);

        loop {
            if painting_robot.paint(&mut board).is_err() {
                break;
            }
        }

        painting_robot.wait();
        board.print();

        String::from("URCAFLCP")
    }
}

#[derive(Clone,Copy)]
enum PaintColour {
    Black,
    White
}

impl PaintColour {
    fn from_i64(i: i64) -> Self {
        match i {
            0 => PaintColour::Black,
            1 => PaintColour::White,
            _ => panic!("Invalid color!")
        }
    }

    fn to_i64(&self) -> i64 {
        match self {
            PaintColour::Black => 0,
            PaintColour::White => 1,
        }
    }

    fn to_char(&self) -> char {
        match self {
            PaintColour::Black => ' ',
            PaintColour::White => '#',
        }
    }
}

enum RobotRotation {
    Left,
    Right
}

impl RobotRotation {
    fn from_i64(i: i64) -> Self {
        match i {
            0 => RobotRotation::Left,
            1 => RobotRotation::Right,
            _ => panic!("Invalid rotation!")
        }
    }
}

enum Direction {
    Up,
    Down,
    Left,
    Right
}

struct Board {
    panels: HashMap<Point, PaintColour>
}

impl Board {
    fn new() -> Self {
        Self { panels: HashMap::new() }
    }

    fn paint(&mut self, p: &Point, c: &PaintColour) {
        self.panels.insert(p.clone(), c.clone());
    }

    fn colour(&self, p: &Point) -> PaintColour {
        *self.panels.get(p).unwrap_or(&PaintColour::Black)
    }

    fn print(&self) {
        let min_x = self.panels.keys().min_by_key(|p| p.x).unwrap().x;
        let min_y = self.panels.keys().min_by_key(|p| p.y).unwrap().y;
        let max_x = self.panels.keys().max_by_key(|p| p.x).unwrap().x;
        let max_y = self.panels.keys().max_by_key(|p| p.y).unwrap().y;

        for y in (min_y..=max_y).rev() {
            for x in min_x..=max_x {
                print!("{}", self.colour(&Point{ x: x, y: y }).to_char());
            }
            println!();
        }
    }
}

struct PaintingRobot {
    handle: thread::JoinHandle<()>,
    curr_direction: Direction,
    position: Point,
    tx_chan: Sender<i64>,
    rx_chan: Receiver<i64>,
}

impl PaintingRobot {
    fn new(program: &Vec<i64>) -> Self {
        let (io, tx, rx) = AsyncIO::new();
        let mut machine = IntCodeMachine::new(program, io);
        let handle = thread::spawn(move || machine.run());
        Self {
            handle: handle,
            curr_direction: Direction::Up,
            position: Point{ x: 0, y: 0 },
            tx_chan: tx,
            rx_chan: rx
        }
    }

    fn wait(self) {
        let _ = self.handle.join();
    }

    fn paint(&mut self, board: &mut Board) -> Result<(), Box<dyn Error>>{
        let curr_colour = board.colour(&self.position);

        // Error on send means the channel is closed, i.e. machine has halted
        self.tx_chan.send(curr_colour.to_i64())?;

        let colour = PaintColour::from_i64(self.rx_chan.recv()?);
        let turn_direction = RobotRotation::from_i64(self.rx_chan.recv()?);

        board.paint(&self.position, &colour);

        // Move the robot
        match turn_direction {
            RobotRotation::Left => self.turn_left(),
            RobotRotation::Right => self.turn_right()
        }

        Ok(())
    }

    fn turn_left(&mut self) {
        match self.curr_direction {
            Direction::Up => { self.position.x -= 1; self.curr_direction = Direction::Left },
            Direction::Down => { self.position.x += 1; self.curr_direction = Direction::Right },
            Direction::Left => { self.position.y -= 1; self.curr_direction = Direction::Down },
            Direction::Right => { self.position.y += 1; self.curr_direction = Direction::Up }
        }
    }

    fn turn_right(&mut self) {
        match self.curr_direction {
            Direction::Up => { self.position.x += 1; self.curr_direction = Direction::Right },
            Direction::Down  => { self.position.x -= 1; self.curr_direction = Direction::Left },
            Direction::Left => { self.position.y += 1; self.curr_direction = Direction::Up }
            Direction::Right => { self.position.y -= 1; self.curr_direction = Direction::Down },
        }
    }
}
