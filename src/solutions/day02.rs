use crate::intcode::{parse_program, IntCodeMachine, NoIO};
use crate::solver::Solver;
use std::fs::File;

pub struct Problem;

impl Solver for Problem {
    type Input = Vec<i64>;
    type Output1 = i64;
    type Output2 = i64;

    fn parse_input(&self, f: File) -> Self::Input {
        parse_program(f)
    }

    fn solve_first(&self, input: &Self::Input) -> Self::Output1 {
        let mut program = input.clone();
        program[1] = 12;
        program[2] = 2;
        let mut machine = IntCodeMachine::new(&program, NoIO {});
        machine.run();
        machine.program[0]
    }

    fn solve_second(&self, input: &Self::Input) -> Self::Output2 {
        for i in 0..99 {
            for j in 0..99 {
                let mut program = input.clone();
                program[1] = i;
                program[2] = j;
                let mut machine = IntCodeMachine::new(&program, NoIO {});
                machine.run();
                if machine.program[0] == 19690720 {
                    return 100 * i + j;
                }
            }
        }
        panic!("Should not get here.");
    }
}
