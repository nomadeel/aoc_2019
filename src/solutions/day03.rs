use crate::solver::Solver;
use std::convert::TryInto;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Clone, Copy, Debug, Eq)]
pub struct Point {
    x: i32,
    y: i32
}

impl PartialEq for Point {
    fn eq(&self, other: &Point) -> bool {
        self.x == other.x && self.y == other.y
    }
}

pub struct Problem;

impl Solver for Problem {
    type Input = (Vec<Point>, Vec<Point>);
    type Output1 = u32;
    type Output2 = u32;

    fn parse_input(&self, f: File) -> Self::Input {
        let f = BufReader::new(f);
        let mut f = f.lines();
        let wire_movements1: Vec<String> = f.next().unwrap().unwrap().split(',').map(|s| s.to_string()).collect();
        let wire_movements2: Vec<String> = f.next().unwrap().unwrap().split(',').map(|s| s.to_string()).collect();
        (process_wire_movements(&wire_movements1), process_wire_movements(&wire_movements2))
    }

    fn solve_first(&self, (points_set1, points_set2): &Self::Input) -> Self::Output1 {
        let intersections: Vec<&Point> = points_set1.iter().filter(|i| points_set2.contains(&i)).collect();
        intersections.iter().map(|p| calculate_manhattan_distance(&p)).min().unwrap()
    }

    fn solve_second(&self, (points_set1, points_set2): &Self::Input) -> Self::Output2 {
        let intersections: Vec<&Point> = points_set1.iter().filter(|i| points_set2.contains(&i)).collect();
        intersections.iter().map(|p| {
            points_set1.iter().position(|x| *x == **p).unwrap() + points_set2.iter().position(|y| *y == **p).unwrap() + 2
        }).min().unwrap() as Self::Output2
    }
}

fn process_wire_movements(movements: &Vec<String>) -> Vec<Point> {
    let mut points: Vec<Point> = vec![];
    let mut curr_point: Point = Point { x: 0, y: 0 };
    for m in movements {
        let move_vector = match &m[..1] {
            "U" => (0, 1),
            "D" => (0, -1),
            "L" => (-1, 0),
            "R" => (1, 0),
            _ => panic!("Invalid movement!")
        };
        let steps = m[1..].parse().unwrap();
        for _ in 0..steps {
            curr_point.x += move_vector.0;
            curr_point.y += move_vector.1;
            points.push(curr_point);
        }
    }
    points
}

fn calculate_manhattan_distance(p: &Point) -> u32 {
    (p.x.abs() + p.y.abs()).try_into().unwrap()
}
