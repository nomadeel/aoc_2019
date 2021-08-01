use crate::{
    intcode::{parse_program, IntCodeMachine, AsyncIO},
    solver::Solver,
    grid::Point,
};
use std::{
    collections::HashMap,
    fs::File,
    sync::mpsc::{Sender, Receiver},
    thread,
    error::Error
};

pub struct Problem;

impl Solver for Problem {
    type Input = Vec<i64>;
    type Output1 = usize;
    type Output2 = u64;

    fn parse_input(&self, f: File) -> Self::Input {
        parse_program(f)
    }

    fn solve_first(&self, input: &Self::Input) -> Self::Output1 {
        let mut area = GameArea::new();
        let mut arcade_cabinet = ArcadeCabinet::new(input);

        let _ = arcade_cabinet.only_fill_map(&mut area);

        arcade_cabinet.wait();
        area.print();

        area.tiles.values().filter(|t| **t == Tile::Block).count()
    }

    fn solve_second(&self, input: &Self::Input) -> Self::Output2 {
        let mut area = GameArea::new();
        let mut play_for_free_input = input.clone();
        play_for_free_input[0] = 2;
        let mut arcade_cabinet = ArcadeCabinet::new(&play_for_free_input);

        let _ = arcade_cabinet.play(&mut area);

        let end_score = arcade_cabinet.score;

        arcade_cabinet.wait();

        end_score
    }
}

#[derive(Clone, PartialEq)]
enum Tile {
    Empty,
    Wall,
    Block,
    HorizontalPaddle,
    Ball
}

impl Tile {
    fn from_i64(i: i64) -> Self {
        match i {
            0 => Tile::Empty,
            1 => Tile::Wall,
            2 => Tile::Block,
            3 => Tile::HorizontalPaddle,
            4 => Tile::Ball,
            _ => panic!("Invalid tile code!")
        }
    }

    fn to_char(&self) -> char {
        match self {
            Tile::Empty => ' ',
            Tile::Wall => '#',
            Tile::Block => '@',
            Tile::HorizontalPaddle => '_',
            Tile::Ball => 'o'
        }
    }
}

struct GameArea {
    tiles: HashMap<Point, Tile>
}

impl GameArea {
    fn new() -> Self {
        Self { tiles: HashMap::new() }
    }

    fn get_tile(&self, p: &Point) -> Option<&Tile> {
        self.tiles.get(p)
    }

    fn set_tile(&mut self, p: &Point, t: &Tile) {
        self.tiles.insert(p.clone(), t.clone());
    }

    fn print(&self) {
        let min_x = self.tiles.keys().min_by_key(|p| p.x).unwrap().x;
        let min_y = self.tiles.keys().min_by_key(|p| p.y).unwrap().y;
        let max_x = self.tiles.keys().max_by_key(|p| p.x).unwrap().x;
        let max_y = self.tiles.keys().max_by_key(|p| p.y).unwrap().y;

        // Don't reverse it since Y is from distance from top
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                print!("{}", self.get_tile(&Point{ x: x, y: y }).unwrap().to_char());
            }
            println!();
        }
    }
}

struct ArcadeCabinet {
    score: u64,
    ball: Option<i64>,
    paddle: Option<i64>,
    handle: thread::JoinHandle<()>,
    tx_chan: Sender<i64>,
    rx_chan: Receiver<i64>
}

impl ArcadeCabinet {
    fn new(program: &Vec<i64>) -> Self {
        let (io, tx, rx) = AsyncIO::new();
        let mut machine = IntCodeMachine::new(program, io);
        let handle = thread::spawn(move || machine.run());
        Self {
            score: 0,
            ball: None,
            paddle: None,
            handle: handle,
            tx_chan: tx,
            rx_chan: rx
        }
    }

    fn wait(self) {
        let _ = self.handle.join();
    }

    fn only_fill_map(&mut self, area: &mut GameArea) -> Result<(), Box<dyn Error>> {
        loop {
            let x = self.rx_chan.recv()?;
            let y = self.rx_chan.recv()?;
            let tile_id = self.rx_chan.recv()?;
            let tile = Tile::from_i64(tile_id);
            area.set_tile(&Point{ x, y }, &tile);
        }
    }

    fn play(&mut self, area: &mut GameArea) -> Result<(), Box<dyn Error>> {
        loop {
            let x = self.rx_chan.recv()?;
            let y = self.rx_chan.recv()?;
            let tile_id = self.rx_chan.recv()?;

            if x == -1 && y == 0 {
                self.score = tile_id as u64;
                continue;
            }

            let tile = Tile::from_i64(tile_id);
            area.set_tile(&Point{ x, y }, &tile);

            match &tile {
                Tile::Ball => { self.ball = Some(x) },
                Tile::HorizontalPaddle => {
                    if self.paddle == None {
                        self.paddle = Some(x);
                        self.send_move()?;
                    }
                },
                _ => ()
            }

            if self.ball != None && self.paddle != None {
                if tile == Tile::Ball {
                    self.send_move()?;
                }
            }
        }
    }

    fn send_move(&mut self) -> Result<(), Box <dyn Error>> {
        let paddle_x = self.paddle.unwrap();
        let ball_x = self.ball.unwrap();
        match paddle_x.cmp(&ball_x) {
            std::cmp::Ordering::Less => {
                self.tx_chan.send(1)?;
                self.paddle = Some(paddle_x + 1);
            }
            std::cmp::Ordering::Equal => {
                self.tx_chan.send(0)?;
            }
            std::cmp::Ordering::Greater => {
                self.tx_chan.send(-1)?;
                self.paddle = Some(paddle_x - 1);
            }
        }
        Ok(())
    }
}
