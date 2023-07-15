use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::time::Instant;
use csv::Writer;
mod sudoku;
mod utils;

use crate::sudoku::{Sudoku, BruteForceSolver,
    // CSPSolver,
    RuleBasedSolver, Stochastic, Solver};

fn main() {
    let file = File::open("/data/easy.txt").unwrap();
    let reader = BufReader::new(file);

    let mut writer = Writer::from_path("/data/output.csv").unwrap();
    writer.write_record(&["Puzzle", "Model", "Time", "Correct"]).unwrap();

    for line in reader.lines() {
        let puzzle = line.unwrap();
        let sudoku = Sudoku::new();
        sudoku.from_string(&puzzle).unwrap();

        let solvers: Vec<Box<dyn Solver>> = vec![
            Box::new(BruteForceSolver::new(sudoku.clone())),
            // Box::new(CSPSolver::new(sudoku.clone())),
            Box::new(RuleBasedSolver::new(sudoku.clone())),
            Box::new(Stochastic::new(1000.0, 0.99)),
        ];

        for solver in solvers {
            let start = Instant::now();
            let result = solver.solve(&mut sudoku);
            let duration = start.elapsed();

            let is_correct = match &result {
                Some(res) => res.is_solved(), // no is_solved yet
                None => false,
            };

            writer.write_record(&[
                &puzzle, 
                solver.name(), 
                &format!("{:?}", duration), 
                &format!("{}", is_correct)
            ]).unwrap();

            writer.flush().unwrap();
        }
    }
}
