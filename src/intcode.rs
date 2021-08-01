use std::convert::TryInto;
use std::{
    fs::File,
    io::{BufRead, BufReader, ErrorKind, Result},
    sync::mpsc::{channel, Receiver, Sender}
};
use std::io;

pub fn parse_program(f: File) -> Vec<i64> {
    BufReader::new(f)
        .lines()
        .flatten()
        .collect::<String>()
        .split(",")
        .map(String::from)
        .map(|s| s.parse().unwrap())
        .collect()
}

#[derive(Debug)]
enum Instruction {
    Add,
    Multiply,
    Input,
    Output,
    JumpTrue,
    JumpFalse,
    LessThan,
    Equal,
    ChangeRelative,
    Halt,
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
#[derive(Copy)]
enum ParamMode {
    Position,
    Immediate,
    Relative,
}

impl From<i64> for ParamMode {
    fn from(n: i64) -> Self {
        match n {
            0 => ParamMode::Position,
            1 => ParamMode::Immediate,
            2 => ParamMode::Relative,
            _ => panic!("Invalid number for ParamMode!"),
        }
    }
}

pub trait IO {
    fn get(&mut self) -> Result<i64>;
    fn put(&mut self, val: i64) -> Result<()>;
}

pub struct NoIO {}

impl IO for NoIO {
    fn get(&mut self) -> Result<i64> {
        Ok(0)
    }

    fn put(&mut self, _val: i64) -> Result<()> {
        Ok(())
    }
}

pub struct AsyncIO {
    tx: Sender<i64>,
    rx: Receiver<i64>,
}

impl AsyncIO {
    pub fn new() -> (Self, Sender<i64>, Receiver<i64>) {
        let (itx, orx) = channel();
        let (otx, irx) = channel();
        let s = Self { tx: itx, rx: irx };
        (s, otx, orx)
    }
}

impl IO for AsyncIO {
    fn get(&mut self) -> Result<i64> {
        self.rx.recv().map_err(|e| io::Error::new(ErrorKind::BrokenPipe, e))
    }
    
    fn put(&mut self, val: i64) -> Result<()> {
        self.tx.send(val).map_err(|e| io::Error::new(ErrorKind::BrokenPipe, e))
    }
}

pub struct Connector {
    tx: Vec<Sender<i64>>,
    rx: Receiver<i64>,
}

impl Connector {
    pub fn new(tx: Sender<i64>, rx: Receiver<i64>) -> Self {
        Self { tx: vec![tx], rx }
    }

    pub fn multiplexed(tx: Vec<Sender<i64>>, rx: Receiver<i64>) -> Self {
        Self { tx, rx }
    }

    pub fn run(&self) {
        while let Ok(data) = self.rx.recv() {
            for tx in self.tx.iter() {
                let _ = tx.send(data);
            }
        }
    }
}

pub struct IntCodeMachine<T> where T: IO {
    pub program: Vec<i64>,
    pub io: T,
    pc: usize,
    relative_base: i64,
    halted: bool,
}

impl<T> IntCodeMachine<T> where T: IO {
    pub fn new(program: &Vec<i64>, io: T) -> Self {
        Self {
            program: program.to_vec(),
            io: io,
            pc: 0,
            relative_base: 0,
            halted: false,
        }
    }

    fn read_memory(&mut self, pos: usize) -> i64 {
        if pos >= self.program.len() {
            self.program.resize(pos + 1, 0);
        }
        self.program[pos]
    }

    fn write_memory(&mut self, pos: usize, value: i64) {
        if pos >= self.program.len() {
            self.program.resize(pos + 1, 0);
        }
        self.program[pos] = value;
    }

    fn fetch_operand(&mut self, param_index: usize, param_mode: ParamMode) -> i64 {
        let param = self.program[self.pc + param_index + 1];
        match param_mode {
            ParamMode::Position => self.read_memory(param as usize),
            ParamMode::Immediate => param,
            ParamMode::Relative => {
                let pos = param + self.relative_base;
                self.read_memory(pos as usize)
            }
        }
    }

    fn fetch_destination(&mut self, param_index: usize, param_mode: ParamMode) -> i64 {
        let param = self.program[self.pc + param_index + 1];
        match param_mode {
            ParamMode::Position => param,
            ParamMode::Immediate => panic!("Should never get immediate as a position parameter!"),
            ParamMode::Relative => {
                assert!(param + self.relative_base >= 0);
                param + self.relative_base
            }
        }
    }

    fn fetch_operands3(&mut self, param_modes: Vec<ParamMode>) -> (i64, i64, i64) {
        (self.fetch_operand(0, param_modes[0]), self.fetch_operand(1, param_modes[1]), self.fetch_destination(2, param_modes[2]))
    }

    fn fetch_operands2(&mut self, param_modes: Vec<ParamMode>) -> (i64, i64) {
        (self.fetch_operand(0, param_modes[0]), self.fetch_operand(1, param_modes[1]))
    }

    fn exec_arithmetic(&mut self, inst: Instruction, param_modes: Vec<ParamMode>) {
        let (num1, num2, pos) = self.fetch_operands3(param_modes);
        let res: i64 = match inst {
            Instruction::Add => num1 + num2,
            Instruction::Multiply => num1 * num2,
            _ => panic!("Unexpected instruction!")
        };
        self.write_memory(pos as usize, res);
        self.pc += 4
    }

    fn exec_io(&mut self, inst: Instruction, param_modes: Vec<ParamMode>) {
        match inst {
            Instruction::Input => {
                let pos = self.fetch_destination(0, param_modes[0]);
                if let Ok(input) = self.io.get() {
                    self.write_memory(pos as usize, input); 
                    self.pc += 2;
                } else {
                    self.halted = true;
                }
            },
            Instruction::Output => {
                let operand = self.fetch_operand(0, param_modes[0]);
                if self.io.put(operand).is_ok() {
                    self.pc += 2;
                } else {
                    self.halted = true;
                }
            }
            _ => panic!("Unexpected instruction!")
        }
    }

    fn exec_branching(&mut self, inst: Instruction, param_modes: Vec<ParamMode>) {
        let (num1, pos) = self.fetch_operands2(param_modes);
        let check_fn: Box<dyn Fn(i64) -> bool> = match inst {
            Instruction::JumpTrue => { Box::new(|x| x != 0) },
            Instruction::JumpFalse => { Box::new(|x| x == 0) },
            _ => panic!("Unexpected instruction!")
        };

        if check_fn(num1) {
            self.pc = pos as usize;
        } else {
            self.pc += 3
        }
    }

    fn exec_compare(&mut self, inst: Instruction, param_modes: Vec<ParamMode>) {
        let (num1, num2, pos) = self.fetch_operands3(param_modes);
        let check_fn: Box<dyn Fn(i64, i64) -> bool> = match inst {
            Instruction::LessThan => { Box::new(|x, y| x < y) },
            Instruction::Equal => { Box::new(|x, y| x == y) },
            _ => panic!("Unexpected instruction!")
        };

        if check_fn(num1, num2) {
            self.write_memory(pos as usize, 1);
        } else {
            self.write_memory(pos as usize, 0);
        }

        self.pc += 4
    } 

    fn exec_relative(&mut self, param_modes: Vec<ParamMode>) {
        let offset = self.fetch_operand(0, param_modes[0]);
        self.relative_base += offset;
        self.pc += 2;
    }

    fn decode_instruction(&mut self) -> (Instruction, Vec<ParamMode>) {
        let inst = match self.program[self.pc] % 100 {
            1 => Instruction::Add,
            2 => Instruction::Multiply,
            3 => Instruction::Input,
            4 => Instruction::Output,
            5 => Instruction::JumpTrue,
            6 => Instruction::JumpFalse,
            7 => Instruction::LessThan,
            8 => Instruction::Equal,
            9 => Instruction::ChangeRelative,
            99 => Instruction::Halt,
            i => panic!("Invalid opcode {}!", i),
        };
        let param_modes = vec![
            ((self.program[self.pc] / 100) % 10).into(),
            ((self.program[self.pc] / 1000) % 10).into(),
            ((self.program[self.pc] / 10000) % 10).into()
        ];
        (inst, param_modes)
    }

    fn step(&mut self) {
        assert!(self.pc < self.program.len().try_into().unwrap());
        let (inst, param_modes) = self.decode_instruction();
        match inst {
            Instruction::Add => self.exec_arithmetic(Instruction::Add, param_modes),
            Instruction::Multiply => self.exec_arithmetic(Instruction::Multiply, param_modes),
            Instruction::Input => self.exec_io(Instruction::Input, param_modes),
            Instruction::Output => self.exec_io(Instruction::Output, param_modes),
            Instruction::JumpTrue => self.exec_branching(Instruction::JumpTrue, param_modes),
            Instruction::JumpFalse => self.exec_branching(Instruction::JumpFalse, param_modes),
            Instruction::LessThan => self.exec_compare(Instruction::LessThan, param_modes),
            Instruction::Equal => self.exec_compare(Instruction::Equal, param_modes),
            Instruction::ChangeRelative => self.exec_relative(param_modes),
            Instruction::Halt => { self.halted = true },
        }
    }

    pub fn run(&mut self) {
        while !self.halted {
            self.step();
        }
    }
}
