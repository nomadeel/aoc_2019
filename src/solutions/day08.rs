use crate::solver::Solver;
use std::{
    fs::File,
    io::Read
};

pub struct Problem;

impl Solver for Problem {
    type Input = Vec<u8>;
    type Output1 = usize;
    type Output2 = String;

    fn parse_input(&self, f: File) -> Self::Input {
        f.bytes().flatten().map(|b| b - b'0').collect()
    }

    fn solve_first(&self, input: &Self::Input) -> Self::Output1 {
        let image = Image::from_array(input, 25, 6);
        // Find the layer with the least zeroes
        let (layer_with_least_zero, _) = image.layers
            .iter()
            .enumerate()
            .map(|(i, v)| (i, v.iter().filter(|x| **x == 0).count()))
            .min_by_key(|(_, c)| *c)
            .unwrap();
        // Get number of ones and twoes
        let ones = image.layers[layer_with_least_zero as usize].iter().filter(|x| **x == 1).count();
        let twos = image.layers[layer_with_least_zero as usize].iter().filter(|x| **x == 2).count();
        ones * twos
    }

    fn solve_second(&self, input: &Self::Input) -> Self::Output2 {
        let image = Image::from_array(input, 25, 6);
        // Rasterize the image so that we can print it easily
        let rasterized_image = image.rasterize();
        for line in rasterized_image {
            println!("{}", line);
        }
        String::from("YLFPJ")
    }
}

struct Image {
    layers: Vec<Vec<u8>>,
    width: usize,
    height: usize
}

impl Image {
    fn from_array(input: &[u8], width: usize, height: usize) -> Self {
        let layer_size = width * height;
        let mut layers: Vec<Vec<u8>> = input
            .chunks(layer_size)
            .map(|l| l.into())
            .collect();
        layers.pop();
        Self { layers, width, height }
    }

    fn rasterize(&self) -> Vec<String> {
        let mut flat_image = String::new();
        for l in 0..(self.width * self.height) {
            if Some(1) == self.layers.iter().map(|v| v[l]).filter(|x| *x != 2).next() {
                flat_image.push('1');
            } else {
                flat_image.push(' ');
            }
        }
        flat_image.as_bytes().chunks(self.width).map(|x| String::from(std::str::from_utf8(x).ok().unwrap())).collect()
    }
}
