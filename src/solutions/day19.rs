use crate::{
    solver::Solver,
    intcode::{parse_program, AsyncIO, IntCodeMachine},
};
use std::{
    fs::File,
    thread,
    error::Error,
};
use itertools::Itertools;

#[derive(Clone, Eq, PartialEq)]
pub enum Tile {
    Empty,
    Beam,
}

impl Tile {
    fn from_i64(value: i64) -> Self {
        match value {
            0 => Tile::Empty,
            1 => Tile::Beam,
            _ => panic!("Invalid input for Tile!")
        }
    }
}

pub struct Problem;

impl Solver for Problem {
    type Input = Vec<i64>;
    type Output1 = usize;
    type Output2 = usize;

    fn parse_input(&self, f: File) -> Self::Input {
        parse_program(f)
    }

    fn solve_first(&self, input: &Self::Input) -> Self::Output1 {
        let mut tractor_drone = TractorDrone::new(input);
        let num_tiles_affected = (0..50).cartesian_product(0..50).fold(0, |acc, (x, y)| {
            match tractor_drone.check_tile(x, y).unwrap() {
                Tile::Empty => acc,
                Tile::Beam => acc + 1,
            }
        });
        num_tiles_affected
    }

    fn solve_second(&self, input: &Self::Input) -> Self::Output2 {
        let mut tractor_drone = TractorDrone::new(input);
        let mut x = 0;
        let mut y = 0;
        // Find the top-right and bottom-left corners of the box
        while tractor_drone.check_tile(x + 99, y).unwrap() == Tile::Empty {
            y += 1;
            while tractor_drone.check_tile(x, y + 99).unwrap() == Tile::Empty {
                x += 1;
            }
        }
        x * 10000 + y
    }
}

pub struct TractorDrone {
    program: Vec<i64>
}

impl TractorDrone {
    fn new(program: &Vec<i64>) -> Self {
        Self {
            program: program.clone(),
        }
    }

    fn check_tile(&mut self, x: usize, y: usize) -> Result<Tile, Box<dyn Error>> {
        let (io, tx, rx) = AsyncIO::new();
        let mut machine = IntCodeMachine::new(&self.program, io);
        let handle = thread::spawn(move || machine.run());
        // Send x and y as input coordinates
        tx.send(x as i64)?;
        tx.send(y as i64)?;
        let result = rx.recv()?;
        let _ =handle.join();
        Ok(Tile::from_i64(result))
    }
}
