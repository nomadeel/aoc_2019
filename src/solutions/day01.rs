use crate::solver::Solver;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

pub struct Problem;

impl Solver for Problem {
    type Input = Vec<u64>;
    type Output1 = u64;
    type Output2 = u64;

    fn parse_input(&self, f: File) -> Self::Input {
        let f = BufReader::new(f);
        f.lines().flatten().flat_map(|l| l.parse()).collect()
    }

    fn solve_first(&self, input: &Self::Input) -> Self::Output1 {
        input.iter().cloned().map(calc_fuel_needed).sum()
    }

    fn solve_second(&self, input: &Self::Input) -> Self::Output2 {
        input.iter().cloned().map(total_fuel_mass).sum()
    }
}

fn calc_fuel_needed(mass: u64) -> u64 {
    // Don't underflow
    if (mass / 3) >= 2 {
        (mass / 3) - 2
    } else {
        0
    }
}

fn total_fuel_mass(mass: u64) -> u64 {
    let mut fuel_required = calc_fuel_needed(mass);
    let mut extra_fuel_for_fuel = calc_fuel_needed(fuel_required);
    while extra_fuel_for_fuel > 0 {
        fuel_required += extra_fuel_for_fuel;
        extra_fuel_for_fuel = calc_fuel_needed(extra_fuel_for_fuel);
    }
    fuel_required
}
