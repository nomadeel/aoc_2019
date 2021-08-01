use std::convert::TryFrom;
use std::{
    io::{BufRead, BufReader, Read},
    iter::{repeat, FromIterator},
    f64::consts::PI,
};
use num::integer::gcd;

#[derive(Clone)]
pub struct Grid<T> {
    cells: Vec<T>,
    pub w: usize,
    pub h: usize,
}

impl<T> Grid<T>
where
    T: Clone + Default + TryFrom<u8>,
{
    pub fn new(w: usize, h: usize) -> Self {
        Self::new_with(w, h, Default::default())
    }

    pub fn new_with(w: usize, h: usize, val: T) -> Self {
        Self {
            cells: Vec::from_iter(repeat(val).take(w * h)),
            w,
            h,
        }
    }

    pub fn from_reader<R: Read>(r: R) -> Result<Self, T::Error> {
        let cells = BufReader::new(r)
            .lines()
            .filter_map(|l| l.ok())
            .map(|l| l.bytes().map(T::try_from).collect::<Result<Vec<_>, _>>())
            .collect::<Result<Vec<_>, _>>()?;
        let h = cells.len();
        let w = cells.first().map_or(0, |c| c.len());

        Ok(Self {
            cells: cells.into_iter().flatten().collect(),
            w,
            h,
        })
    }

    pub fn set(&mut self, c: impl Coord, value: T) {
        if let Some(e) = self.cells.get_mut(c.x() + c.y() * self.w) {
            *e = value;
        }
    }

    pub fn get(&self, c: impl Coord) -> Option<&T> {
        self.cells.get(c.x() + c.y() * self.w)
    }
}

pub trait Coord {
    fn x(&self) -> usize;
    fn y(&self) -> usize;
    fn coords(&self) -> (usize, usize) {
        (self.x(), self.y())
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Copy, Debug)]
pub struct Point {
    pub x: i64,
    pub y: i64,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct GridPoint {
    pub x: usize,
    pub y: usize,
}

impl GridPoint {
    pub fn distance(&self, other: &Self) -> usize {
        ((self.x as isize - other.x as isize).abs() + (self.y as isize - other.y as isize).abs()) as usize
    }

    pub fn vector(&self, other: &Self) -> Vector2D {
        Vector2D::new(
            other.x as isize - self.x as isize,
            other.y as isize - self.y as isize
        )
    }
}

impl Coord for GridPoint {
    fn x(&self) -> usize {
        self.x
    }

    fn y(&self) -> usize {
        self.y
    }
}

impl Coord for (usize, usize) {
    fn x(&self) -> usize {
        self.0
    }

    fn y(&self) -> usize {
        self.1
    }

    fn coords(&self) -> (usize, usize) {
        *self
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Vector2D {
    dx: isize,
    dy: isize,
}

impl Vector2D {
    pub fn new(dx: isize, dy: isize) -> Self {
        let gcd = gcd(dx, dy);
        Self { dx: dx / gcd, dy: dy / gcd }
    }

    pub fn degrees(&self) -> f64 {
        let angle = (self.dy as f64).atan2(self.dx as f64); // returns in terms of radians
        let d = 180.0 * angle / PI + 90.0; // calculate degrees, then add 90.0 as origin is up instead of right
        if d < 0.0 {
            d + 360.0 // don't want negative degrees so add a full revolution
        } else {
            d
        }
    }
}
