use std::{
    fmt::Display,
    fs::File,
    io::self,
};

fn input_file(day: i32) -> String {
    format!("input/day{:02}.txt", day)
}

pub trait Solver {
    type Input;
    type Output1: Display;
    type Output2: Display;

    //fn parse_input<R: io::Seek + io::Read>(&self, r: R) -> Self::Input;
    fn parse_input(&self, f: File) -> Self::Input;
    fn solve_first(&self, input: &Self::Input) -> Self::Output1;
    fn solve_second(&self, input: &Self::Input) -> Self::Output2;

    //fn load_input<P: AsRef<Path>>(&self, p: P) -> io::Result<Self::Input> {
    fn load_input(&self, file_path: String) -> io::Result<Self::Input> {
        let f = File::open(file_path)?;
        Ok(self.parse_input(f))
    }

    fn solve(&self, day: i32) {
        let input_file = input_file(day);
        let input = self
            .load_input(input_file)
            .expect(&format!("Unable to open input file input/day{:02}.txt!", day));
        let s1 = self.solve_first(&input);
        let s2 = self.solve_second(&input);
        println!("Solution 1: {}", s1);
        println!("Solution 2: {}", s2);
    }
}

