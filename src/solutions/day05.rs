use crate::intcode::{parse_program, IntCodeMachine, IO};
use crate::solver::Solver;
use std::fs::File;
use std::io::Result;

pub struct SimpleIO {
    val: i64,
}

impl IO for SimpleIO {
    fn get(&mut self) -> Result<i64> {
        Ok(self.val)
    }

    fn put(&mut self, val: i64) -> Result<()> {
        self.val = val;
        Ok(())
    }
}

pub struct Problem;

impl Solver for Problem {
    type Input = Vec<i64>;
    type Output1 = i64;
    type Output2 = i64;

    fn parse_input(&self, f: File) -> Self::Input {
        parse_program(f)
    }

    fn solve_first(&self, input: &Self::Input) -> Self::Output1 {
        let mut machine = IntCodeMachine::new(&input, SimpleIO { val: 1 });
        machine.run();
        machine.io.val
    }

    fn solve_second(&self, input: &Self::Input) -> Self::Output2 {
        let mut machine = IntCodeMachine::new(&input, SimpleIO { val: 5 });
        machine.run();
        machine.io.val
    }
}
