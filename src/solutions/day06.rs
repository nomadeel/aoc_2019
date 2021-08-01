use crate::solver::Solver;
use std::{
    fs::File,
    collections::{HashMap, HashSet},
    io::{BufRead, BufReader},
};

pub struct SpaceObject {
    parent_object: String,
    satellites: Vec<String>
}

pub struct Problem;

impl Solver for Problem {
    type Input = HashMap<String, SpaceObject>;
    type Output1 = u32;
    type Output2 = u32;

    fn parse_input(&self, f: File) -> Self::Input {
        let mut satellite_map: HashMap<String, SpaceObject> = HashMap::new();
        for line in BufReader::new(f).lines() {
            let orbit_string = String::from(&line.unwrap());
            let mut split = orbit_string.split(')');
            let space_obj: String = split.next().unwrap().to_string();
            let orbiter: String = split.next().unwrap().to_string();
            let map_match = satellite_map.get_mut(&orbiter);
            match map_match {
                Some(orbiter_object) => { orbiter_object.parent_object.insert_str(0, &space_obj); },
                None => { satellite_map.insert(String::from(&orbiter), SpaceObject { 
                    parent_object: String::from(&space_obj), satellites: vec!() }); ()
                }
            };
            let new_space_obj = satellite_map.entry(String::from(&space_obj)).or_insert(SpaceObject {
                parent_object: String::new(), satellites: vec!()});
            new_space_obj.satellites.push(String::from(&orbiter));
        }
        satellite_map
    }

    fn solve_first(&self, input: &Self::Input) -> Self::Output1 {
        total_direct_and_indirect_orbits(input, "COM", 0)
    }

    fn solve_second(&self, input: &Self::Input) -> Self::Output2 {
        let mut seen: HashSet<&str> = HashSet::new();
        let mut from: HashMap<&str, &str> = HashMap::new();
        let mut to_visit: Vec<&str> = vec!("YOU");
        seen.insert("YOU");
        from.insert("YOU", "YOU");

        // Run BFS over the tree to find the shortest path
        while !to_visit.is_empty() {
            let curr = to_visit.pop().unwrap();
            if curr == "SAN" {
                break;
            }
            let curr_object = input.get(curr).unwrap();
            curr_object.satellites.iter().for_each(|x| {
                if !seen.contains(x.as_str()) {
                    seen.insert(x.as_str());
                    from.insert(x.as_str(), curr);
                    to_visit.push(x)
                }
            });
            if curr != "COM" && !seen.contains(curr_object.parent_object.as_str()) {
                seen.insert(curr_object.parent_object.as_str());
                from.insert(curr_object.parent_object.as_str(), curr);
                to_visit.push(curr_object.parent_object.as_str());
            }
        }

        let mut curr = "SAN";
        let mut num_nodes = 0;
        loop {
            curr = from.get(curr).unwrap();
            if curr == "YOU" {
                break;
            }
            num_nodes += 1;
        }

        num_nodes - 1 // We count the "SAN" node
    }
}

// Count number of children starting from some object
fn total_direct_and_indirect_orbits(satellite_map: &HashMap<String, SpaceObject>, curr_object: &str, depth: u32) -> u32 {
    let space_object = satellite_map.get(curr_object).unwrap();

    match space_object.satellites.len() {
        0 => depth,
        _ => {
            let mut orbiters = 0;
            for s in space_object.satellites.iter() {
                orbiters += total_direct_and_indirect_orbits(satellite_map, &s, depth + 1);
            }
            orbiters + depth
        },
    }
}
