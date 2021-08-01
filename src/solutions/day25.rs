use crate::{
    solver::Solver,
    intcode::{parse_program, AsyncIO, IntCodeMachine}
};
use std::{
    fs::File,
    thread,
    error::Error,
    sync::mpsc::{Sender, Receiver, TryRecvError},
    io::{stdin, stdout, Write}
};

pub struct Problem;

const INTERACTIVE: bool = false;

impl Solver for Problem {
    type Input = Vec<i64>;
    type Output1 = i64;
    type Output2 = i64;

    fn parse_input(&self, f: File) -> Self::Input {
        parse_program(f)
    }

    fn solve_first(&self, input: &Self::Input) -> Self::Output1 {
        let mut navigator_droid = NavigatorDroid::new(input);
        if INTERACTIVE {
            loop {
                navigator_droid.print_output().expect("Failed to print out output from the navigator droid!");
                navigator_droid.prompt_command().expect("Failed to input command for the navigator droid!");
            }
        } else {
            // Found the solution interactively
            let mut instructions = vec!();
            instructions.push("south\n");
            instructions.push("west\n");
            instructions.push("south\n");
            instructions.push("take shell\n");
            instructions.push("north\n");
            instructions.push("north\n");
            instructions.push("take weather machine\n");
            instructions.push("west\n");
            instructions.push("south\n");
            instructions.push("east\n");
            instructions.push("take candy cane\n");
            instructions.push("west\n");
            instructions.push("north\n");
            instructions.push("east\n");
            instructions.push("south\n");
            instructions.push("east\n");
            instructions.push("east\n");
            instructions.push("south\n");
            instructions.push("take hypercube\n");
            instructions.push("south\n");
            instructions.push("south\n");
            instructions.push("east\n");
            let _ = navigator_droid.input_script(&instructions);
        }
        0
    }

    fn solve_second(&self, _input: &Self::Input) -> Self::Output2 {
        0
    }
}

struct NavigatorDroid {
    handle: thread::JoinHandle<()>,
    tx_chan: Sender<i64>,
    rx_chan: Receiver<i64>,
}

impl NavigatorDroid {
    fn new(program: &Vec<i64>) -> Self {
        let (io, tx, rx) = AsyncIO::new();
        let mut machine = IntCodeMachine::new(program, io);
        let handle = thread::spawn(move || machine.run());
        Self {
            handle,
            tx_chan: tx,
            rx_chan: rx
        }
    }

    fn print_output(&mut self) -> Result<(), Box<dyn Error>> {
        let mut output = String::new();
        loop {
            let c = self.rx_chan.recv()?;
            let character = char::from_u32(c as u32).unwrap();
            output.push(character);
            if character == '\n' {
                print!("{}", output);
                if output == "Command?\n" {
                    break;
                }
                output.clear();
            }
        }
        Ok(())
    }

    fn prompt_command(&mut self) -> Result<(), Box<dyn Error>> {
        let mut command = String::new();
        stdin().read_line(&mut command).expect("Failed to read in command for navigator!");
        for c in command.trim().chars() {
            self.tx_chan.send(c as i64)?;
        }
        self.tx_chan.send(10)?;
        Ok(())
    }

    fn input_script(&mut self, instructions: &Vec<&str>) -> Result<(), Box<dyn Error>> {
        for i in instructions {
            for c in i.chars() {
                self.tx_chan.send(c as i64)?;
            }
        }

        // Drain the output now
        loop {
            let c = self.rx_chan.recv()?;
            let character = char::from_u32(c as u32).unwrap();
            print!("{}", character);
        }
    }
}
