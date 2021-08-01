use crate::solver::Solver;
use std::{
    io::{BufRead, BufReader},
    fs::File,
};

pub struct Problem;

impl Solver for Problem {
    type Input = Vec<Technique>;
    type Output1 = usize;
    type Output2 = usize;

    fn parse_input(&self, f: File) -> Self::Input {
        BufReader::new(f)
            .lines()
            .filter_map(|l| l.ok())
            .map(|l| Technique::new(&l))
            .collect()
    }

    fn solve_first(&self, input: &Self::Input) -> Self::Output1 {
        let mut shuffler = Shuffler::new(10007);
        shuffler.shuffle(&input);
        shuffler.cards.iter().enumerate().find(|(_, x)| **x == 2019).unwrap().0
    }

    fn solve_second(&self, input: &Self::Input) -> Self::Output2 {
        let shuffler = Shuffler::new(0);
        let size = 119315717514047;
        let iterations = 101741582076661;
        let pos = 2020;
        shuffler.shuffle_long(&input, size, iterations, pos)
    }
}

#[derive(Clone,Debug)]
enum TechniqueType {
    Cut,
    DealWith,
    DealStack
}

#[derive(Clone,Debug)]
pub struct Technique {
    technique_type: TechniqueType,
    argument: i64,
}

impl Technique {
    fn new(desc: &String) -> Self {
        if desc.contains("cut") {
            let argument = desc.split(" ").last().unwrap().parse().unwrap();
            Self { technique_type: TechniqueType::Cut, argument: argument }
        } else if desc.contains("deal with") {
            let argument = desc.split(" ").last().unwrap().parse().unwrap();
            Self { technique_type: TechniqueType::DealWith, argument: argument }
        } else if desc.contains("deal into") {
            Self { technique_type: TechniqueType::DealStack, argument: 0 }
        } else {
            panic!("Invalid technique string!");
        }
    }
}

struct Shuffler {
    cards: Vec<usize>,
}

impl Shuffler {
    fn new(deck_size: usize) -> Self {
        let mut cards = vec!();
        (0..deck_size).for_each(|i| cards.push(i));
        Self { cards }
    }

    fn shuffle(&mut self, techniques: &Vec<Technique>) {
        techniques.iter().for_each(|t| {
            match t.technique_type {
                TechniqueType::Cut => self.cut(t.argument),
                TechniqueType::DealWith => self.deal_with(t.argument),
                TechniqueType::DealStack => self.deal_stack(),
            }
        })
    }

    fn cut(&mut self, argument: i64) {
        if argument >= 0 {
            self.cards.rotate_left(argument as usize);
        } else {
            self.cards.rotate_right(argument.abs() as usize);
        }
    }

    fn deal_with(&mut self, argument: i64) {
        let deck_size = self.cards.len();
        let mut output = vec![0; deck_size];
        let mut index = 0;
        for c in self.cards.iter() {
            output[index] = *c;
            index = (index + argument as usize) % deck_size;
        }
        self.cards = output;
    }

    fn deal_stack(&mut self) {
        self.cards.reverse();
    }

    /*
     * Inspiration from this post:
     * https://www.reddit.com/r/adventofcode/comments/ee0rqi/2019_day_22_solutions/fbwauzi?utm_source=share&utm_medium=web2x&context=3
     * Inspirations from people on Reddit, the problem is essentially this polynomial:
     *  ax + b mod L, where L is the size of the deck
     * - Cut increases or decreases 'b'
     * - Deal with increment increases 'a'
     * - Dealing into new stack reverses the position, i.e. a = -1 * a, b = L - 1 - b
     */
    fn shuffle_long(&self, techniques: &Vec<Technique>, size: usize, iterations: usize, pos: usize) -> usize {
        let mut reversed = techniques.to_vec();
        reversed.reverse();

        let mut a: i128 = 1;
        let mut b: i128 = 0;
        let l = size as i128;

        // Compose the polynomial functions now
        for t in reversed {
            match t.technique_type {
                TechniqueType::Cut => {
                    b = b + t.argument as i128;
                    b = b.rem_euclid(l);
                },
                TechniqueType::DealStack => { a *= -1; b = l - b - 1; },
                TechniqueType::DealWith => {
                    let z = mod_pow(t.argument as i128, l - 2, l); // modinv(t.argument, l), only when l is prime though
                    a *= z;
                    a = a.rem_euclid(l);
                    b *= z;
                    b = b.rem_euclid(l);
                }
            }
        }

        // Raise our function to the power of iterations, i.e. running the same function 'iterations' time
        let (a, b) = polypow(a, b, iterations as i128, l);


        (a * pos as i128 + b).rem_euclid(l) as usize
    }
}

// From: https://rosettacode.org/wiki/Modular_exponentiation#Rust
fn mod_pow(b: i128, e: i128, m: i128) -> i128 {
    if e == 0 {
        return 1;
    }

    // Now do the modular exponentiation algorithm:
    let mut result: i128 = 1;
    let mut base = b % m;
    let mut exp = e;
    let modulus = m;

    // Loop until we can return out result:
    loop {
        if &exp % 2 == 1 {
            result *= &base;
            result %= &modulus;
        }

        if exp == 1 {
            return result;
        }

        exp /= 2;
        base *= base.clone();
        base %= &modulus;
    }
}

/*
 * Inspiration from this post:
 * https://www.reddit.com/r/adventofcode/comments/ee0rqi/2019_day_22_solutions/fbwauzi?utm_source=share&utm_medium=web2x&context=3
 */
fn polypow(a: i128, b: i128, m: i128, n: i128) -> (i128, i128) {
    if m == 0 {
        (1, 0)
    } else if m % 2 == 0 {
        polypow((a * a).rem_euclid(n), (a * b + b).rem_euclid(n), m / 2, n)
    } else {
        let (c, d) = polypow(a, b, m - 1, n);
        ((a * c).rem_euclid(n), (a * d + b).rem_euclid(n))
    }
}
