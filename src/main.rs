use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::time::Instant;
use csv::Writer;
mod sudoku;
mod utils;

use crate::sudoku::{Sudoku, BruteForceSolver,
    // CSPSolver,
    RuleBasedSolver, StochasticSolver, Solver};

    fn main() {
        let file = File::open("./data/easy.txt").unwrap();
        let reader = BufReader::new(file);
    
        let mut writer = Writer::from_path("./data/output.csv").unwrap();
        writer.write_record(&["Puzzle", "Model", "Time", "Correct"]).unwrap();
    
        for line in reader.lines() {
            let puzzle = line.unwrap();
            let sudoku = Sudoku::new(Some(&puzzle)).unwrap();
            let solvers: Vec<Box<dyn Solver>> = vec![
                // Box::new(BruteForceSolver::new()),
                // Box::new(CSPSolver::new(sudoku.clone())),
                Box::new(RuleBasedSolver::new()),
                // Box::new(StochasticSolver::new(1000.0, 0.99, &sudoku.clone())),
            ];
    
            for mut solver in solvers {
                let mut sudoku_clone = sudoku.clone();
                let start = Instant::now();
                let result = solver.solve(&mut sudoku_clone);
                let duration = start.elapsed();
    
                let is_correct = sudoku_clone.is_solved(); // assuming is_solved method exists
    
                writer.write_record(&[
                    &puzzle, 
                    &solver.name(), 
                    &format!("{:?}", duration), 
                    &format!("{}", is_correct)
                ]).unwrap();
    
                writer.flush().unwrap();
            }
        }
    }
    
