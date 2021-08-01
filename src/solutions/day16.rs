use crate::solver::Solver;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

pub struct Problem;

impl Solver for Problem {
    type Input = Vec<i16>;
    type Output1 = String;
    type Output2 = String;

    fn parse_input(&self, f: File) -> Self::Input {
        BufReader::new(f)
            .lines()
            .flatten()
            .collect::<String>()
            .as_bytes()
            .iter()
            .map(|s| (*s - 48) as i16) // '0'
            .collect()
    }

    fn solve_first(&self, input: &Self::Input) -> Self::Output1 {
        let output = fft(input);
        let first_eight = output.iter().take(8).map(|s| s.to_string()).collect();
        first_eight
    }

    fn solve_second(&self, input: &Self::Input) -> Self::Output2 {
        /* Optimisation from
         * https://www.reddit.com/r/adventofcode/comments/ebf5cy/2019_day_16_part_2_understanding_how_to_come_up/fb4bvw4/
         */
        let real_signal = input.repeat(10000);
        let message_offset = real_signal.iter().take(7).map(|s| s.to_string()).collect::<String>().parse().unwrap();
        let mut real_signal = real_signal[message_offset..].to_vec();
        let signal_length = real_signal.len();
        for _ in 0..100 {
            for i in 2..signal_length {
                real_signal[signal_length - i] = (real_signal[signal_length - i] + real_signal[signal_length - i + 1]) % 10;
            }
        }
        let message = real_signal.iter().take(8).map(|s| s.to_string()).collect::<String>();
        message
    }
}

fn fft(input: &Vec<i16>) -> Vec<i16> {
    let mut output = input.clone();
    let sequence_len = input.len();
    for _ in 0..100 {
        let mut new_sequence = vec!();
        for i in 0..sequence_len {
            // Contruct the pattern, i + 1 because i starts from 0
            let pattern = construct_pattern(sequence_len, i + 1);
            let mut digit_output = output.iter().zip(pattern).fold(0, |acc, (i, j)| { acc + i * j } );
            // Get the ones digit from the output
            digit_output %= 10;
            new_sequence.push(digit_output.abs());
        }
        output = new_sequence;
    }
    output
}

fn construct_pattern(sequence_len: usize, repeat_by: usize) -> Vec<i16> {
    let mut output = vec!();
    let base_pattern: Vec<i16> = vec![0, 1, 0, -1];
    let mut pattern_index = 0;
    let mut numbers_filled = 0;
    // Sequence_len + 1 because we remove the first number in the pattern
    while numbers_filled < sequence_len + 1 {
        let mut other = vec![base_pattern[pattern_index]; repeat_by];
        output.append(&mut other);
        numbers_filled += repeat_by;
        pattern_index = (pattern_index + 1) % base_pattern.len();
    }
    output.remove(0);
    output.truncate(sequence_len);
    output
}
