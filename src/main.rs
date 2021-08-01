#![feature(slice_group_by)]

mod solutions;
mod solver;
mod intcode;
mod grid;

use crate::solutions::run_day;
use std::env;

fn main() {
    let day = env::args()
        .nth(1)
        .unwrap_or_else(|| { println!("Given no input, running default day 1..."); "1".to_string()})
        .parse()
        .unwrap_or(1);
    run_day(day)
}
