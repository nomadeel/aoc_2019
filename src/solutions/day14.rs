use crate::solver::Solver;
use std::{
    io::{BufRead, BufReader},
    collections::{HashMap, VecDeque},
    fs::File,
    fmt
};
use regex::Regex;

pub struct Problem;

impl Solver for Problem {
    // No references instead of string since we can't use lifetimes
    type Input = HashMap<String, Recipe>;
    type Output1 = u64;
    type Output2 = u64;

    fn parse_input(&self, f: File) -> Self::Input {
        let mut recipe_map = HashMap::new();
        let r = Regex::new(r"(\d+) (\w+)").unwrap();

        BufReader::new(f)
            .lines()
            .filter_map(|l| l.ok())
            .for_each(|l| {
                let splits: Vec<_> = l.split("=>").collect();
                let product_caps = r.captures(&splits[1].trim()).unwrap();
                let product_quantity: u64 = product_caps.get(1).unwrap().as_str().parse().unwrap();
                let product_string = product_caps.get(2).unwrap().as_str();
                let mut new_recipe = Recipe::new(product_string, product_quantity);
                let input_splits: Vec<_> = splits[0].split(',').collect();
                for i in input_splits {
                    let input_caps = r.captures(i.trim()).unwrap();
                    let input_quantity: u64 = input_caps.get(1).unwrap().as_str().parse().unwrap();
                    let input_string = input_caps.get(2).unwrap().as_str();
                    new_recipe.add_input(input_string, input_quantity);
                }
                recipe_map.insert(String::from(product_string), new_recipe);
            });

        recipe_map
    }

    fn solve_first(&self, input: &Self::Input) -> Self::Output1 {
        let mut factory = Factory::new(input.clone());
        factory.build("FUEL", 1);
        factory.ore_used
    }

    fn solve_second(&self, input: &Self::Input) -> Self::Output2 {
        let mut factory = Factory::new(input.clone());
        let mut fuel_made = 0;
        while factory.build("FUEL", 1) {
            fuel_made += 1;
        }
        fuel_made
    }
}

#[derive(Debug, Clone)]
pub struct Recipe {
    output: String,
    quantity: u64,
    inputs: HashMap<String, u64>,
}

impl fmt::Display for Recipe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "output: {}, quanity: {}, inputs: {:?}", self.output, self.quantity, self.inputs)
    }
}

impl Recipe {
    fn new(output: &str, quantity: u64) -> Self {
        Self {
            output: String::from(output),
            quantity: quantity,
            inputs: HashMap::new()
        }
    }

    fn add_input(&mut self, input: &str, quantity: u64) {
        self.inputs.insert(String::from(input), quantity);
    }
}

struct Factory {
    recipes: HashMap<String, Recipe>,
    inventory: HashMap<String, u64>,
    ore_used: u64,
}

impl Factory {
    fn new(recipes: HashMap<String, Recipe>) -> Self {
        let mut inventory: HashMap<String, u64> = HashMap::new();
        inventory.insert("ORE".to_string(), 1000000000000);
        Self {
            recipes: recipes,
            inventory: inventory,
            ore_used: 0,
        }
    }

    fn build(&mut self, material: &str, quantity: u64) -> bool {
        let mut build_queue: VecDeque<(String, u64)> = VecDeque::new();
        build_queue.push_back((material.to_string(), quantity));

        while !build_queue.is_empty() {
            let (curr_material, curr_needed) = build_queue.pop_front().unwrap();
            let available = self.grab_from_inventory(&curr_material, curr_needed);
            // We're good, we can fulfill it
            if curr_needed <= available {
                continue;
            }
            let to_fulfill = curr_needed - available;

            // Otherwise figure what we need to build and add the order
            if let Some(recipe) = self.recipes.get(&curr_material.to_string()).cloned() {
                let multiple = (to_fulfill as f64 / recipe.quantity as f64).ceil() as u64;
                let produced = multiple * recipe.quantity;
                let surplus = produced - to_fulfill;
                self.put_in_inventory(&curr_material, surplus);
                recipe.inputs.iter().for_each(|(i, q)| {
                    build_queue.push_back((i.clone(), q * multiple));
                });
            } else {
                return false;
            }
        }

        true
    }

    fn put_in_inventory(&mut self, material: &str, quantity: u64) {
        let entry = self.inventory.entry(material.to_string()).or_insert(0);
        *entry += quantity;
    }

    fn grab_from_inventory(&mut self, material: &str, quantity: u64) -> u64 {
        let store = self.inventory.entry(material.to_string()).or_insert(0);
        if *store >= quantity {
            if material == "ORE" {
                self.ore_used += quantity;
            }
            *store -= quantity;
            quantity
        } else {
            let remains = *store;
            *store = 0;
            remains
        }
    }
}
