use crate::{
    solver::Solver,
    intcode::{parse_program, IntCodeMachine, IO}
};
use std::{
    fs::File,
    thread,
    error::Error,
    sync::mpsc::{channel, Sender, Receiver, TryRecvError},
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
    time::Duration,
    io::ErrorKind,
};
use std::io;

const NUM_ROUTERS: usize = 50;

pub struct Problem;

impl Solver for Problem {
    type Input = Vec<i64>;
    type Output1 = i64;
    type Output2 = i64;

    fn parse_input(&self, f: File) -> Self::Input {
        parse_program(f)
    }

    fn solve_first(&self, input: &Self::Input) -> Self::Output1 {
        let mut network = Network::new(input, NUM_ROUTERS);
        let result = network.run_network(true);
        result.unwrap()
    }

    fn solve_second(&self, input: &Self::Input) -> Self::Output2 {
        println!("This will take some time...");
        let mut network = Network::new(input, NUM_ROUTERS);
        let result = network.run_network(false);
        result.unwrap()
    }
}

struct NonBlockIO {
    tx: Sender<i64>,
    rx: Receiver<i64>,
    first: bool,
    blocked_status: Arc<AtomicBool>,
}

impl NonBlockIO {
    pub fn new() -> (Self, Sender<i64>, Receiver<i64>, Arc<AtomicBool>) {
        let (itx, orx) = channel();
        let (otx, irx) = channel();
        let blocked_status = Arc::new(AtomicBool::new(false));
        let s = Self { tx: itx, rx: irx, first: true, blocked_status: blocked_status.clone() };
        (s, otx, orx, blocked_status)
    }
}

impl IO for NonBlockIO {
    fn get(&mut self) -> io::Result<i64> {
        if self.first {
            self.first = false;
            self.rx.recv().map_err(|e| io::Error::new(ErrorKind::BrokenPipe, e))
        } else {
            match self.rx.try_recv() {
                Ok(v) => {
                    self.blocked_status.store(false, Ordering::SeqCst);
                    Ok(v)
                },
                _ => {
                    self.blocked_status.store(true, Ordering::SeqCst);
                    Ok(-1)
                }
            }
        }
    }

    fn put(&mut self, val: i64) -> io::Result<()> {
        self.tx.send(val).map_err(|e| io::Error::new(ErrorKind::BrokenPipe, e))
    }
}

struct Network {
    handles: Vec<thread::JoinHandle<()>>,
    tx_chans: Vec<Sender<i64>>,
    rx_chans: Vec<Receiver<i64>>,
    blocked_statuses: Vec<Arc<AtomicBool>>,
    num_routers: usize,
}

impl Network {
    fn new(program: &Vec<i64>, num_routers: usize) -> Self {
        let mut handles = vec!();
        let mut tx_chans = vec!();
        let mut rx_chans = vec!();
        let mut blocked_statuses = vec!();
        for _ in 0..num_routers {
            let (io, tx, rx, blocked_status) = NonBlockIO::new();
            let mut machine = IntCodeMachine::new(program, io);
            let handle = thread::spawn(move || machine.run());
            handles.push(handle);
            tx_chans.push(tx);
            rx_chans.push(rx);
            blocked_statuses.push(blocked_status);
        }
        Self { handles, tx_chans, rx_chans, num_routers, blocked_statuses }
    }

    fn run_network(&mut self, first: bool) -> Result<i64, Box<dyn Error>> {
        // Boot up the routers
        for (i, t) in self.tx_chans.iter().enumerate() {
            t.send(i as i64)?;
        }

        let mut last_nat_sent = None;
        let mut nat = None;
        let mut ticks = 0;
        let mut sent_packets = false;
        loop {
            // Receive packets from each of the routers
            for i in 0..self.num_routers {
                loop {
                    match self.rx_chans[i].try_recv() {
                        Ok(a) => {
                            // Receive the rest of the packet
                            let x = self.rx_chans[i].recv()?;
                            let y = self.rx_chans[i].recv()?;
                            if a == 255 {
                                match first {
                                    true => return Ok(y),
                                    false => {
                                        // Overwrite the nat entry
                                        nat = Some((x, y));
                                    }
                                }
                            } else {
                                // Send it
                                self.tx_chans[a as usize].send(x)?;
                                self.tx_chans[a as usize].send(y)?;
                                sent_packets = true;
                            }
                        }
                        Err(TryRecvError::Disconnected) => return Err(Box::new(TryRecvError::Disconnected)),
                        Err(TryRecvError::Empty) => break,
                    }
                }
            }

            // Add a sleep so that the other routers (threads) can move
            thread::sleep(Duration::from_millis(50));
            // Since we're dealing with concurrency as well, add in some slack
            // time to allow the threads to actually run
            if !sent_packets {
                ticks += 1;
            }

            if !first && !sent_packets && ticks > 15 {
                ticks = 0;
                let routers_idle = self.blocked_statuses.iter().all(|b| b.load(Ordering::SeqCst) == true);
                if routers_idle && nat != None {
                    let nat_entry = nat.unwrap();
                    match last_nat_sent {
                        Some((x, y)) => {
                            if x == nat_entry.0 && y == nat_entry.1 {
                                return Ok(nat_entry.1);
                            }
                        },
                        None => (),
                    }
                    // Send nat_entry entry to router 0
                    self.tx_chans[0].send(nat_entry.0)?;
                    self.tx_chans[0].send(nat_entry.1)?;
                    last_nat_sent = Some(nat_entry);
                }
            }

            sent_packets = false;
        }

        Ok(0)
    }
}
