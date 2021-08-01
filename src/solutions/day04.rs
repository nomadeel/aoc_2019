use crate::solver::Solver;
use std::convert::TryInto;
use std::{
    fs::File,
};

pub struct Problem;

impl Solver for Problem {
    type Input = (u32, u32);
    type Output1 = u32;
    type Output2 = u32;

    fn parse_input(&self, _: File) -> Self::Input {
        let range_start = 357253;
        let range_end = 892942;
        let mut match1 = 0;
        let mut match2 = 0;

        for i in range_start..(range_end + 1) {
            // Extract the digits and throw them into a vector
            let num_vec: Vec<u32> = vec!(0, 0, 0, 0, 0, 0).iter().enumerate().map(|(j, _)| {
                (i / 10u32.pow((5 - j).try_into().unwrap())) % 10
            }).collect();
            let ascending = num_vec.windows(2).all(|w| w[0] <= w[1]);
            if ascending {
                let collapsed_vec: Vec<&[u32]> = num_vec.group_by(|a, b| a == b).collect();
                if collapsed_vec.iter().any(|&i| i.len() >= 2) {
                    match1 += 1;
                }
                if collapsed_vec.iter().any(|&i| i.len() == 2) {
                    match2 += 1
                }
            }
        }

        (match1, match2)
    }

    fn solve_first(&self, (answer, _): &Self::Input) -> Self::Output1 {
        *answer
    }

    fn solve_second(&self, (_, answer): &Self::Input) -> Self::Output2 {
        *answer
    }
}
