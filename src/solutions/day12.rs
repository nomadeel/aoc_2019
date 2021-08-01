use crate::solver::Solver;
use std::{
    io::{BufReader,BufRead},
    fs::File,
    cmp::Ordering
};
use regex::Regex;
use itertools::Itertools;
use num::Integer;

pub struct Problem;

impl Solver for Problem {
    type Input = JupiterSystem;
    type Output1 = u64;
    type Output2 = u64;

    fn parse_input(&self, f: File) -> Self::Input {
        JupiterSystem::from_file(f)
    }

    fn solve_first(&self, input: &Self::Input) -> Self::Output1 {
        let mut jupiter_copy = input.clone();
        for _ in 0..1000 {
            jupiter_copy.run_simulation_step();
        }
        jupiter_copy.calculate_energy()
    }

    fn solve_second(&self, input: &Self::Input) -> Self::Output2 {
        let x_period = find_position_period(&mut input.moons.iter().map(|m| m.position[0]).collect());
        let y_period = find_position_period(&mut input.moons.iter().map(|m| m.position[1]).collect());
        let z_period = find_position_period(&mut input.moons.iter().map(|m| m.position[2]).collect());
        x_period.lcm(&y_period.lcm(&z_period))
    }
}

#[derive(Debug,Clone,PartialEq)]
struct Moon {
    position: [i32; 3],
    velocity: [i32; 3],
}

impl Moon {
    fn apply_gravity(&mut self, other: &mut Self) {
        for dim in 0..3 {
            let cmp_res = self.position[dim].cmp(&other.position[dim]);
            match cmp_res {
                Ordering::Less => { self.velocity[dim] += 1; other.velocity[dim] -= 1 },
                Ordering::Greater => { self.velocity[dim] -= 1; other.velocity[dim] += 1 },
                _ => ()
            }
        }
    }
}

#[derive(Clone)]
pub struct JupiterSystem {
    moons: Vec<Moon>,
}

impl JupiterSystem {
    fn from_file(f: File) -> Self {
        let r = Regex::new(r"<x=(-?\d+), y=(-?\d+), z=(-?\d+)").unwrap();
        let moons = BufReader::new(f)
            .lines()
            .map(|l| {
                let line = l.unwrap();
                let caps = r.captures(&line).unwrap();
                let x = caps.get(1).unwrap().as_str().parse().unwrap();
                let y = caps.get(2).unwrap().as_str().parse().unwrap();
                let z = caps.get(3).unwrap().as_str().parse().unwrap();
                Moon { position: [x, y, z], velocity: [0, 0, 0] }
            })
            .collect();
        JupiterSystem { moons }
    }

    fn run_simulation_step(&mut self) {
        let pairs: Vec<Vec<_>> = (0..4).combinations(2).collect();
        // Apply gravity
        for p in &pairs {
            // Just rust things
            if p[0] == 0 {
                let (moons1, moons2) = self.moons.split_at_mut(1);
                moons1[0].apply_gravity(&mut moons2[p[1] - 1]);
            } else {
                let (moons1, moons2) = self.moons.split_at_mut(p[0] + 1);
                moons1[p[0]].apply_gravity(&mut moons2[p[1] - p[0] - 1]);
            }
        }

        // Apply velocity
        for moon in self.moons.iter_mut() {
            for dim in 0..3 {
                moon.position[dim] += moon.velocity[dim]
            }
        }
    }

    fn calculate_energy(&self) -> u64 {
        self.moons.iter().fold(0, |s,moon| {
            s + moon.position.iter().fold(0, |s,v| s + v.abs()) as u64 * moon.velocity.iter().fold(0, |s,v| s + v.abs()) as u64
        })
    }
}

fn apply_gravity(positions: &mut [i32], velocities: &mut [i32]) {
    let pairs: Vec<Vec<_>> = (0..4).combinations(2).collect();
    for p in &pairs {
        let cmp_res = positions[p[0]].cmp(&positions[p[1]]);
        match cmp_res {
            std::cmp::Ordering::Less => { velocities[p[0]] += 1; velocities[p[1]] -= 1 },
            std::cmp::Ordering::Greater => { velocities[p[0]] -= 1; velocities[p[1]] += 1 },
            _ => ()
        }
    }

    for i in 0..4 {
        positions[i] += velocities[i];
    }
}

fn find_position_period(positions: &mut Vec<i32>) -> u64 {
    let mut velocities = vec![0; 4];
    let initial_velocities = velocities.clone();
    let mut steps = 0;

    loop {
        apply_gravity(positions, &mut velocities);
        steps += 1;
        if initial_velocities == velocities {
            break;
        }
    }

    // Steps is half the period, e.g. half of sine wave
    steps * 2
}
