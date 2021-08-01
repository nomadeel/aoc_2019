use crate::{
    solver::Solver,
    grid::{Grid, GridPoint, Vector2D}
};
use std::{
    collections::HashMap,
    convert::TryFrom,
    fs::File,
};

#[derive(Clone, Eq, PartialEq)]
pub enum Elem {
    Empty,
    Asteroid,
}

impl TryFrom<u8> for Elem {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'.' => Ok(Elem::Empty),
            b'#' => Ok(Elem::Asteroid),
            v => Err(format!("Invalid cell: {}", v))
        }
    }
}

impl Default for Elem {
    fn default() -> Self {
        Elem::Empty
    }
}

pub struct Problem;

impl Solver for Problem {
    type Input = Grid<Elem>;
    type Output1 = usize;
    type Output2 = u64;

    fn parse_input(&self, f: File) -> Self::Input {
        Grid::from_reader(f).unwrap()
    }

    fn solve_first(&self, input: &Self::Input) -> Self::Output1 {
        let (_, number) = find_best_location(input);
        number
    }

    fn solve_second(&self, input: &Self::Input) -> Self::Output2 {
        let (point, _) = find_best_location(input);
        let vaporisation_order = calculate_vaporisation_order(&point, input);
        let target = &vaporisation_order[199];
        (target.x as u64 * 100) + target.y as u64
    }
}

fn find_best_location(grid: &Grid<Elem>) -> (GridPoint, usize) {
    let mut visibles = vec!();
    for y in 0..grid.h {
        for x in 0..grid.w {
            if grid.get((x, y)) == Some(&Elem::Empty) {
                continue;
            }

            // Find all the visible asteroids from this point
            let point = GridPoint{ x, y };
            let visibles_from_point = find_all_visible(&point, grid);
            visibles.push((point, visibles_from_point));
        }
    }

    let (point, visibles_at_point) = visibles.iter().max_by_key(|(_, v)| v.len()).unwrap();
    (point.clone(), visibles_at_point.len())
}

fn find_all_visible(from: &GridPoint, grid: &Grid<Elem>) -> Vec<GridPoint> {
    let mut closest_map: HashMap<Vector2D, GridPoint> = HashMap::new();
    for y in 0..grid.h {
        for x in 0..grid.w {
            // Don't check ourselves
            if x == from.x && y == from.y {
                continue;
            }

            // Don't check empty space
            if grid.get((x, y)) == Some(&Elem::Empty) {
                continue;
            }

            let point = GridPoint { x, y };
            let point_vector = point.vector(&from);
            let map_object = closest_map.get(&point_vector);
            match map_object {
                Some(existing_point) => {
                    if existing_point.distance(from) > point.distance(from) {
                        closest_map.insert(point_vector, point);
                    }
                },
                None => { closest_map.insert(point_vector, point); }
            }
        }
    }
    closest_map.values().map(|x| x.clone()).collect()
}

fn calculate_vaporisation_order(from: &GridPoint, grid: &Grid<Elem>) -> Vec<GridPoint> {
    let mut vaporisation_order = vec!();
    let mut working_set = grid.clone();

    loop {
        let mut visibles_from_point = find_all_visible(from, &working_set);
        if visibles_from_point.is_empty() {
            break;
        }
        visibles_from_point.sort_by(|a, b| from.vector(a).degrees().partial_cmp(&from.vector(b).degrees()).unwrap());
        // Now remove each of those from the working set now
        for p in &visibles_from_point {
            working_set.set((p.x, p.y), Elem::Empty);
        }
        vaporisation_order.extend(visibles_from_point.iter().map(|p| p.clone()));
    }

    vaporisation_order
}
