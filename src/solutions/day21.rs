use crate::{
    solver::Solver,
    intcode::{parse_program, AsyncIO, IntCodeMachine}
};
use std::{
    fs::File,
    thread,
    error::Error,
    sync::mpsc::{Sender, Receiver}
};

pub struct Problem;

impl Solver for Problem {
    type Input = Vec<i64>;
    type Output1 = i64;
    type Output2 = i64;

    fn parse_input(&self, f: File) -> Self::Input {
        parse_program(f)
    }

    fn solve_first(&self, input: &Self::Input) -> Self::Output1 {
        let mut spring_droid = SpringDroid::new(input);
        // Basically, jump if there's a hole in front of us or there's a hole
        // three tiles away that we need to jump early for and there's ground
        // four tiles away, the jump range is four tiles
        let script = "NOT A J\nNOT C T\nOR T J\nAND D J\nWALK\n";
        let _ = spring_droid.input_script(script);
        let output = spring_droid.run_script();
        let _ = spring_droid.handle.join();
        match output {
            Ok(o) => match o {
                0 => println!("Springdroid fell through the hull and into space..."),
                _ => return o,
            }
            _ => println!("Springdroid failed somewhere"),
        }
        0
    }

    fn solve_second(&self, input: &Self::Input) -> Self::Output2 {
        let mut spring_droid = SpringDroid::new(input);
        // Basically jump if there's space 4 and 8 ahead
        let script = "NOT C J\nAND D J\nAND H J\nNOT B T\nAND D T\nOR T J\nNOT A T\nOR T J\nRUN\n";
        let _ = spring_droid.input_script(script);
        let output = spring_droid.run_script();
        let _ = spring_droid.handle.join();
        match output {
            Ok(o) => match o {
                0 => println!("Springdroid fell through the hull and into space..."),
                _ => return o,
            }
            _ => println!("Springdroid failed somewhere"),
        }
        0
    }
}

struct SpringDroid {
    handle: thread::JoinHandle<()>,
    tx_chan: Sender<i64>,
    rx_chan: Receiver<i64>,
}

impl SpringDroid {
    fn new(program: &Vec<i64>) -> Self {
        let (io, tx, rx) = AsyncIO::new();
        let mut machine = IntCodeMachine::new(program, io);
        let handle = thread::spawn(move || machine.run());
        Self {
            handle: handle,
            tx_chan: tx,
            rx_chan: rx,
        }
    }

    fn input_script(&mut self, script: &str) -> Result<(), Box<dyn Error>> {
        // Print out the prompt
        for _ in 0..20 {
            let output = self.rx_chan.recv()?;
            let character = char::from_u32(output as u32).unwrap();
            print!("{}", character);
        }
        // Input the script
        for c in script.chars() {
            print!("{}", c);
            self.tx_chan.send(c as i64)?;
        }
        Ok(())
    }

    fn run_script(&mut self) -> Result<i64, Box<dyn Error>> {
        while let Ok(c) = self.rx_chan.recv() {
            // This isn't a printable character and is the damage reported by the springdroid
            if c > 127 {
                return Ok(c);
            }
            let character = char::from_u32(c as u32).unwrap();
            print!("{}", character);
        }
        Ok(0)
    }
}
