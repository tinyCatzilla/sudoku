use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::time::Instant;
use csv::Writer;
mod sudoku;
mod utils;



use crate::sudoku::{Sudoku, BruteForceSolver,
    // CSPSolver,
    RuleBasedSolver, StochasticSolver, CSPSolver, Solver};

fn main() {
    let mut writer = Writer::from_path("./data/output.csv").unwrap();
    writer.write_record(&["Puzzle", "Model", "Time", "Correct"]).unwrap();

    // Get the first line (puzzle) from the file
    let first_line = {
        let file = File::open("./data/easy.txt").unwrap();
        let mut reader = BufReader::new(file);
        reader.lines().next().unwrap().unwrap()
    };

    let first_sudoku = Sudoku::new(Some(&first_line)).unwrap();

    // Instantiate the solvers using the first puzzle
    let mut solvers: Vec<Box<dyn Solver>> = vec![
        // Box::new(BruteForceSolver::new()),
        // Box::new(RuleBasedSolver::new()),
        Box::new(CSPSolver::new()),
        // Box::new(StochasticSolver::new(10000.0, 0.999, first_sudoku.clone())),
    ];

    // Re-instantiate the BufReader
    let file = File::open("./data/large.txt").unwrap();
    let reader = BufReader::new(file);

    // Ensure to process the first line as well
    for line in std::iter::once(first_line).chain(reader.lines().map(|l| l.unwrap())) {
        let sudoku = Sudoku::new(Some(&line)).unwrap();
        
        for solver in &mut solvers {
            // solver.reset(); // reset the solver state for a new puzzle

            let mut sudoku_clone = sudoku.clone();
            let start = Instant::now();
            solver.initialize_candidates(&mut sudoku_clone);
            let result = solver.solve(&mut sudoku_clone);
            let duration = start.elapsed();

            let is_correct = sudoku_clone.is_solved(); // assuming is_solved method exists

            writer.write_record(&[
                &line, 
                &solver.name(), 
                &format!("{:?}", duration), 
                &format!("{}", is_correct)
            ]).unwrap();

            writer.flush().unwrap();
        }
    }
    println!("FINISHED!!");
}
    

    
