use std::collections::{HashMap, HashSet};
use std::clone::Clone;
use std::{str, vec};
use rand::Rng;
use crate::utils;
use rand::prelude::SliceRandom;
use rand::thread_rng;

// use std::collections::LinkedList;
// use std::rc::Rc;
// use itertools::Itertools;
// Basic structure of a sudoku board
#[derive(Clone)]
pub struct Sudoku {
    board: [[u8; 9]; 9],
    cells: Vec<String>,
    row_peers: HashMap<String, HashSet<String>>,
    col_peers: HashMap<String, HashSet<String>>,
    box_peers: HashMap<String, HashSet<String>>,
    peers: HashMap<String, HashSet<String>>,
    candidates: HashMap<String, HashSet<usize>>,
}

impl Sudoku {
    // Instantiate
    // cells: A list of strings representing the 81 cells of the Sudoku puzzle.
    // row_peers: A hashmap that maps each cell to the set of its 8 row peers.
    // col_peers: A hashmap that maps each cell to the set of its 8 column peers.
    // box_peers: A hashmap that maps each cell to the set of its 8 box peers.
    // peers: A hashmap that maps each cell to the set of its 20 peers (cells sharing a unit).
    // candidates: A hashmap that maps each cell to the set of its possible values.
    pub fn new(puzzle: Option<&str>) -> Result<Self, &str> {
        let rows = "ABCDEFGHI".chars().collect::<Vec<_>>();
        let cols = "123456789".chars().collect::<Vec<_>>();
        let cells: Vec<String> = utils::cross(&rows, &cols);

        let unitlist = {
            let mut unitlist = Vec::new();
            // Rows
            for c in &cols {
                unitlist.push(utils::cross(&rows, &[*c]));
            }
            // Columns
            for r in &rows {
                unitlist.push(utils::cross(&[*r], &cols));
            }
            // Boxes
            for rs in vec![&rows[0..3], &rows[3..6], &rows[6..9]] {
                for cs in vec![&cols[0..3], &cols[3..6], &cols[6..9]] {
                    unitlist.push(utils::cross(rs, cs));
                }
            }
            unitlist
        };

        let mut row_peers: HashMap<String, HashSet<String>> = HashMap::new();
        let mut col_peers: HashMap<String, HashSet<String>> = HashMap::new();
        let mut box_peers: HashMap<String, HashSet<String>> = HashMap::new();

        let units: HashMap<String, Vec<HashSet<String>>> = cells.iter().map(|s| {
            (s.to_string(), unitlist.iter().filter(|u| u.contains(s)).map(|u| u.clone().into_iter().collect()).collect())
        }).collect();        

        for s in &cells {
            let unit_cells = units.get(s).unwrap();
            let mut row_peers_s = HashSet::new();
            let mut col_peers_s = HashSet::new();
            let mut box_peers_s = HashSet::new();

            for unit in unit_cells {
                for s2 in unit {
                    if s2 != s {
                        if s2.chars().nth(0).unwrap() == s.chars().nth(0).unwrap() {
                            row_peers_s.insert(s2.to_string());
                        }
                        if s2.chars().nth(1).unwrap() == s.chars().nth(1).unwrap() {
                            col_peers_s.insert(s2.to_string());
                        }
                        // convert cell to coordinates and check if they are in the same box
                        let (row, col) = utils::cell_to_coords(s);
                        let (row2, col2) = utils::cell_to_coords(s2);
                        if row / 3 == row2 / 3 && col / 3 == col2 / 3 {
                            box_peers_s.insert(s2.to_string());
                        }
                    }
                }
            }
            row_peers.insert(s.to_string(), row_peers_s);
            col_peers.insert(s.to_string(), col_peers_s);
            box_peers.insert(s.to_string(), box_peers_s);
        }        

        let peers: HashMap<String, HashSet<String>> = cells.iter().map(|s| {
            let units_s = units.get(s).unwrap();
            let mut peers_s = HashSet::new();

            for unit in units_s {
                for s2 in unit {
                    if s2 != s {
                        peers_s.insert(s2.to_string());
                    }
                }
            }

            (s.to_string(), peers_s)
        }).collect();

        let mut board: [[u8; 9]; 9] = [[0; 9]; 9];

        if let Some(puzzle_str) = puzzle {
            // If a puzzle string is provided, use it to populate the board.
            board = Self::from_string(puzzle_str)?; // Change the from_string function to return Result<[[u8; 9]; 9], &str>
        }
    
        let sudoku = Sudoku {
            board,
            cells,
            row_peers,
            col_peers,
            box_peers,
            peers,
            candidates: HashMap::new(),
        };
    
        Ok(sudoku)
    }

    // Creates a new Sudoku puzzle from a string.
    pub fn from_string(s: &str) -> Result<[[u8; 9]; 9], &str> {
        if s.len() != 81 {
            return Err("Input string must be 81 characters long.");
        }
    
        let mut grid: [[u8; 9]; 9] = [[0; 9]; 9]; // Initialise an empty 2D array
    
        for row in 0..9 {
            for col in 0..9 {
                let c = s.chars().nth(9*row + col).unwrap();
                let value = if c == '.' {
                    0
                } else {
                    c.to_digit(10).ok_or("Each character must be a digit from 0 to 9 or a dot.")?
                };
                if value > 9 {
                    return Err("Each digit must be from 0 to 9.");
                }
                grid[row][col] = value as u8;
            }
        }
    
        Ok(grid)
    }

    // Initialize candidates for each cell, given the current board
    fn initialize_candidates(&mut self) {
        // First, initialize candidates for each cell as if they were all empty
        for cell in &self.cells {
            self.candidates.insert(cell.clone(), (1..=9).collect());
        }

        // Then, go through the board and for each cell that has a value,
        // remove this value from the candidates of all its peers
        for row in 0..9 {
            for col in 0..9 {
                let cell = utils::coords_to_cell(row, col);
                let digit = self.board[row][col];
                if digit != 0 {
                    self.assign(&cell, digit as usize);
                }
            }
        }
    }

            // // If the digit is not a candidate, we do nothing and return true
        // if !self.candidates[cell].contains(&digit) {
        //     return true;
        // }
        // // If the digit has more than one candidate, we remove it from the candidates of the cell
        // if self.candidates[cell].len() > 1 {
        //     self.candidates.get_mut(cell).unwrap().remove(&digit);
        //     println!("Eliminated {} from {}", digit, cell);
        //     println!("Candidates for {} are now {:?}", cell, self.candidates[cell]);
        // }
        
        // // If the cell has no remaining candidates, we return false to signal a contradiction
        // if self.candidates[cell].is_empty() {
        //     println!("Contradiction: {} has no candidates left", cell);
        //     return false;
        // } 
        // // If the cell has one remaining candidate, we need to eliminate this digit from all peers
        // else if self.candidates[cell].len() == 1 {
        //     let d2 = *self.candidates[cell].iter().next().unwrap();
        
        //     // Get a copy of peers before we start mutating `self`
        //     let peers = self.peers[cell].clone();
        
        //     // If elimination from any peer results in a contradiction, we return false
        //     for s2 in peers.iter() {
        //         if !self.eliminate(s2, d2) {
        //             println!("Elimination of {} from {} resulted in a contradiction", d2, s2);
        //             return false;
        //         }
        //     }
        // }

    fn assign(&mut self, cell: &str, digit: usize) -> bool {
        println!("Assigning {} to {}", digit, cell);
        // other_values is a set of digits that are not equal to the assigned digit
        let mut other_values: HashSet<usize> = self.candidates[cell].clone();
        other_values.remove(&digit);
        // We try to eliminate all other values from the cell
        for d2 in other_values {
            if !self.eliminate(cell, d2) {
                // If elimination of any value results in a contradiction, we return false
                return false;
            }
        }
        true
    }
    



    fn eliminate(&mut self, cell: &str, digit: usize) -> bool {
        println!("Eliminating {} from {}", digit, cell);
        let mut tasks = vec![(cell.to_string(), digit)];
    
        while let Some((cell, digit)) = tasks.pop() {
            // If the digit is not a candidate, we do nothing and continue with the next task
            if !self.candidates[&cell].contains(&digit) {
                continue;
            }
            // If the digit has more than one candidate, we remove it from the candidates of the cell
            if self.candidates[&cell].len() > 1 {
                self.candidates.get_mut(&cell).unwrap().remove(&digit);
                // println!("Eliminated {} from {}", digit, cell);
                // println!("Candidates for {} are now {:?}", cell, self.candidates[&cell]);
                // println!("Candidates of board are now {:?}", self.candidates);
                // println!("Peers of cell are {:?}", self.peers[&cell]);
            }
            // If the cell has no remaining candidates, we return false to signal a contradiction
            if self.candidates[&cell].is_empty() {
                println!("Contradiction: {} has no candidates left", cell);
                return false;
            }
            // If the cell has one remaining candidate, we need to eliminate this digit from all peers
            else if self.candidates[&cell].len() == 1 {
                let d2 = *self.candidates[&cell].iter().next().unwrap();
    
                // Get a copy of peers before we start mutating `self`
                let peers = self.peers[&cell].clone();
    
                // Instead of recursively calling eliminate, add the peers to the task list
                for s2 in peers.iter() {
                    tasks.push((s2.clone(), d2));
                }
            }
    
            // Ensure that for every unit of the cell, the digit has at least one place it can be
            let units = vec![self.row_peers[&cell].clone(), self.col_peers[&cell].clone(), self.box_peers[&cell].clone()];
            for unit in units.iter() {
                let d_places: Vec<_> = unit.iter().filter(|&s| self.candidates[s].contains(&digit)).cloned().collect();
                // If not, we return false to signal a contradiction
                if d_places.is_empty() {
                    println!("Contradiction: {:?} has no place for {}", unit, digit);
                    return false;
                } 
                // If there is only one such place, we assign the digit there
                else if d_places.len() == 1 {
                    // println!("Only one place for {} in {:?}: {}", digit, unit, d_places[0]);
                    if !self.assign(&d_places[0], digit) {
                        println!("Assigning {} to {} resulted in a contradiction", digit, d_places[0]);
                        return false;
                    }
                }
            }
        }
        true
    }
        

    // Check if a given number is valid in a given cell
    // Check directly on the board. If the cell is 0, check if the number is valid
    // Used in naive backtracking
    fn is_valid(&self, row: usize, col: usize, num: usize) -> bool {
        if self.board[row][col] != 0 {
            return false;
        }
        // Check if num is in the same row
        for i in 0..9 {
            if self.board[row][i] == num as u8 {
                return false;
            }
        }
        // Check if num is in the same column
        for i in 0..9 {
            if self.board[i][col] == num as u8 {
                return false;
            }
        }
        // Check if num is in the same box
        let box_row = row - row % 3;
        let box_col = col - col % 3;
        for i in box_row..box_row + 3 {
            for j in box_col..box_col + 3 {
                if self.board[i][j] == num as u8 {
                    return false;
                }
            }
        }
        true
    }

    // Check the unique elements in a given array
    fn unique_elements(arr: [u8; 9]) -> i32 {
        let unique_set: std::collections::HashSet<_> = arr.iter().filter(|&&x| x != 0).collect();
        unique_set.len() as i32
    }

    // Check if the board is solved
    pub fn is_solved(&self) -> bool {
        for i in 0..9 {
            let mut row = [false; 9];
            let mut col = [false; 9];
            let mut box_ = [false; 9];

            for j in 0..9 {
                // check row
                if self.board[i][j] != 0 {
                    if row[(self.board[i][j] - 1) as usize] {
                        return false;
                    }
                    row[(self.board[i][j] - 1) as usize] = true;
                }

                // check column
                if self.board[j][i] != 0 {
                    if col[(self.board[j][i] - 1) as usize] {
                        return false;
                    }
                    col[(self.board[j][i] - 1) as usize] = true;
                }

                // check box
                let box_row = 3*(i/3) + j/3;
                let box_col = 3*(i%3) + j%3;
                if self.board[box_row][box_col] != 0 {
                    if box_[(self.board[box_row][box_col] - 1) as usize] {
                        return false;
                    }
                    box_[(self.board[box_row][box_col] - 1) as usize] = true;
                }
            }
        }
        true
    }

}

pub trait Solver {
    fn solve(&mut self, board: &mut Sudoku) -> bool;
    fn name(&self) -> String;
    fn initialize_candidates(&mut self, sudoku: &mut Sudoku);
}

pub struct BruteForceSolver;
// Brute force solver.
// This solver will try every possible candidate in every empty cell.
// If it hits a dead end, it will backtrack and try a different candidate.

impl Solver for BruteForceSolver {
    fn solve(&mut self, board: &mut Sudoku) -> bool {
        let mut empty_cells = vec![];
        for row in 0..9 {
            for col in 0..9 {
                if board.board[row][col] == 0 {
                    empty_cells.push((row, col));
                }
            }
        }

        println!("Empty cells: {:?}", empty_cells);
        println!("Candidates: {:?}", board.candidates);

        // let cell_id = utils::coords_to_cell(row, col);
        // if !board.candidates.contains_key(&cell_id) {
        //     println!("No candidates for cell with ID: {}", cell_id);
        // }
        empty_cells.sort_by_key(|&(row, col)| board.candidates[&utils::coords_to_cell(row, col)].len());
    
        if empty_cells.is_empty() {
            println!("Brute force solved!");
            return true; // All cells filled, solution found
        }
    
        let (row, col) = empty_cells[0];
        let cell = utils::coords_to_cell(row, col);
        let candidates = board.candidates[&cell].clone(); // Clone the candidates for the first empty cell
    
        for &num in candidates.iter() {
            if board.is_valid(row, col, num) {
                board.board[row][col] = num as u8;
                if self.solve(board) {
                    return true;
                }
                board.board[row][col] = 0; // Undo the assignment
            }
        }
    
        false // No solution found
    }
    


    fn name(&self) -> String {
        "Brute Force Solver".to_string()
    }

    fn initialize_candidates(&mut self, board: &mut Sudoku) {
        board.initialize_candidates();
        println!("Candidates: {:?}", board.candidates);
    }
}

impl BruteForceSolver {
    pub fn new() -> BruteForceSolver {
        BruteForceSolver
    }
}

// Constraint programming with forward propagation and backtracking.

pub struct CSPSolver {
    queue: Vec<String>
}

impl CSPSolver {
    // Constructor for CSPSolver
    pub fn new() -> Self {
        CSPSolver {
            queue: Vec::new()
        }
    }

    fn solved(&self, board: &Sudoku) -> bool {
        // Check if the board is solved by verifying that every cell has exactly one candidate
        for cell in board.candidates.keys() {
            if board.candidates.get(cell).unwrap().len() != 1 {
                return false;
            }
        }
        true
    }
}


impl Solver for CSPSolver {
    fn solve(&mut self, board: &mut Sudoku) -> bool {
            
        let mut depth = 1;
        println!("Depth: {}", depth);
        println!("Queue: {:?}", self.queue);
        while !self.solved(board) {
            let mut counter = 1;
            let mut index = 0;
            for cell in self.queue.clone().iter() {
                println!("Cell: {}", cell);
                println!("Cell Candidates: {:?}", board.candidates[cell]);
                let mut digitstack = Vec::new();
                for digit in board.candidates[cell].clone().iter() {
                    println!("Digit: {}", digit);
                    while counter < depth {
                        let candidates_copy: HashMap<String, HashSet<usize>> = board
                            .candidates
                            .iter()
                            .map(|(key, value)| (key.clone(), value.clone()))
                            .collect();  // Make a copy of the board
                        // println!("board.candidates at start of loop: {:?}", board.candidates);
                        // println!("candidates_copy at start of loop: {:?}", candidates_copy);
                        if !board.assign(&self.queue[0], *digit){
                            println!("CSPSOLVER: Assigning {} to {} failed", digit, cell);
                            digitstack.push(*digit);
                            // println!("board.candidates before backtracking: {:?}", board.candidates);
                            // println!("candidates_copy before backtracking: {:?}", candidates_copy);
                            board.candidates = candidates_copy.clone();  // Revert the board
                            // println!("board.candidates after backtracking: {:?}", board.candidates);
                            if !board.eliminate(&self.queue[0], *digit) {
                                // big problem...
                                println!("CSPSOLVER: Eliminating {} from {} failed", digit, cell);
                                board.candidates = candidates_copy;  // Revert the board
                                return false;
                            }
                            break;
                        }
                        println!("CSPSOLVER: Assigning {} to {} succeeded", digit, cell);
                        // digit is good, go one layer deeper.
                        counter += 1;
                        // println!("Counter: {}", counter);
                    }
                }
                // println!("Digitstack: {:?}", digitstack);
                // println!("Cell: {}", cell);
                // if digitstack.len() != 0 {
                //     println!("Popping!");
                //     self.queue.pop();
                // }
                if board.candidates[cell].len() == 1 {
                    self.queue.remove(index);
                    if index != 0 {
                        index -= 1;
                    }
                    break;
                }
                else{
                index += 1;
                }
            }
            depth += 1;
            println!("Depth: {}", depth);
            println!("Queue: {:?}", self.queue);
            println!("Candidates: {:?}", board.candidates);
        }
        // solved

        // Update self.board to be equivalent to the candidate board
        for row in 0..9 {
            for col in 0..9 {
                let cell = utils::coords_to_cell(row, col);
                let candidates = board.candidates.get(&cell).unwrap().clone();
                if candidates.len() == 1 {
                    board.board[row][col] = candidates.iter().next().unwrap().clone() as u8;
                }
            }
        }
        println!("CSPsolver finished.");
        return true;
    }

    fn name(&self) -> String {
        "Constraint Programming Solver".to_string()
    }

    fn initialize_candidates(&mut self, board: &mut Sudoku) {
        board.initialize_candidates();

        // Priority queue for candidates
        // cells must have more than 1 candidate
        // and be sorted by the number of candidates
        self.queue = board.cells.iter()
            .filter(|cell| board.candidates[*cell].len() > 1)
            .cloned()
            .collect();

        // sort by number of candidates (value, ascending)
        self.queue.sort_by_key(|cell| board.candidates[cell].len());

        println!("Candidates: {:?}", board.candidates);
    }
}


pub struct RuleBasedSolver{
    cells_with_candidates: Vec<String>
}
// Rule-based solver.
// Note that a naked tuple is accompanied by a hidden pair. So this will implement up to naked/hidden tuples. But not quads.

impl Solver for RuleBasedSolver {
    fn solve(&mut self, board: &mut Sudoku) -> bool {
        self.cells_with_candidates = board.cells.iter()
            .filter(|cell| board.candidates[*cell].len() > 1)
            .cloned()
            .collect();

        // Loop through rules
        loop {
            let boardcopy: HashMap<String, HashSet<usize>> = board
                .candidates
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect();  // Make a copy of the board
            println!("Candidates: {:?}", board.candidates);
            let mut changes_made = false;

            // Try to apply each rule in turn.
            if self.apply_basic_rules(board) {
                changes_made = true; 
            }
            if self.apply_intermediate_rules(board) {
                changes_made = true;  
            }
            // if self.apply_complex_rules(board) {
            //     changes_made = true;  
            // }
            if !changes_made {
                break;
            }
            // if self.solved(board) {
            //     break;
            // }

            // if boardcopy is the same as the board then no changes were made and we can break
            if boardcopy == board.candidates {
                break;
            }
        }
    
        // If board is solved, update it
        if self.solved(board) {           
            // Update self.board to be equivalent to the candidate board
            for row in 0..9 {
                for col in 0..9 {
                    let cell = utils::coords_to_cell(row, col);
                    let candidates = board.candidates.get(&cell).unwrap().clone();
                    if candidates.len() == 1 {
                        board.board[row][col] = candidates.iter().next().unwrap().clone() as u8;
                    }
                }
            }
            println!("Rule-based solver finished.");
            return true;
        }
    
        // If board is not solved, apply brute force solver
        else {
            let mut brute_force_solver = BruteForceSolver;
            return brute_force_solver.solve(board);
        }
    }

    fn name(&self) -> String {
        "Rule Based Solver".to_string()
    }

    fn initialize_candidates(&mut self, board: &mut Sudoku) {
        board.initialize_candidates();
        println!("Candidates: {:?}", board.candidates);
    }
}

impl RuleBasedSolver {
    pub fn new() -> RuleBasedSolver {
        RuleBasedSolver{
            cells_with_candidates: Vec::new()
        }
    }
    
    fn apply_basic_rules(&self, board: &mut Sudoku) -> bool {
        // Apply basic rules here: Naked Single, Hidden Single, Naked Pair, Hidden Pair
        // Returns true if a rule could be applied, false otherwise
        // When any rule succeeds, call the solver again

        let mut applied = false;

        if self.naked_single(board) {
            println!("Naked single applied");
            applied = true;
        }
        if self.hidden_single(board) {
            println!("Hidden single applied");
            applied = true;
        }
        if self.naked_pair(board) {
            println!("Naked pair applied");
            applied = true;
        }
        if self.hidden_pair(board) {
            println!("Hidden pair applied");
            applied = true;
        }
        applied
    }

    fn apply_intermediate_rules(&self, board: &mut Sudoku) -> bool {
        // Apply intermediate rules here: Locked Candidates Type 1 and Type 2
        // Returns true if a rule could be applied, false otherwise

        let mut applied = false;

        if self.locked_candidates_type_1(board) {
            println!("Locked candidates type 1 applied");
            applied = true;
        }
        if self.locked_candidates_type_2(board) {
            println!("Locked candidates type 2 applied");
            applied = true;
        }
        applied
    }

    // fn apply_complex_rules(&self, board: &mut Sudoku) -> bool {
    //     // Apply complex rules here: X-Wing, Swordfish
    //     // Returns true if a rule could be applied, false otherwise

    //     let mut applied = false;

    //     if self.x_wing(board) {
    //         applied = true;
    //     }
    //     if self.swordfish(board) {
    //         applied = true;
    //     }
    //     applied
    // }

    fn solved(&self, board: &Sudoku) -> bool {
        // Check if the board is solved by verifying that every cell has exactly one candidate
        for cell in board.candidates.keys() {
            if board.candidates.get(cell).unwrap().len() != 1 {
                return false;
            }
        }
        true
    }

    // Basic rules: Naked Single, Hidden Single, Naked Pair, Hidden Pair

    fn naked_single(&self, board: &mut Sudoku) -> bool {
        let mut found = false; // flag for finding a naked single
        for cell in board.cells.clone().iter() {
            if board.candidates[cell].len() > 1 {
                continue;
            }
            for digit in board.candidates[cell].clone() {
                if !board.assign(cell, digit) {
                    panic!("Contradiction encountered during naked single");
                }
            }
            found = true; // mark that a naked single has been found
        }
        found
    }

    fn hidden_single(&self, board: &mut Sudoku) -> bool {
        let mut found = false;
        // For each cell on the board that has more than one candidate
        for cell in &self.cells_with_candidates {
            // For each digit in candidates
            for digit in board.candidates[cell].clone() {

                // Check the digit's occurrence in row peers
                if self.not_in_peers(board, &board.row_peers[cell], digit) {
                    if !board.assign(&cell, digit) {
                        panic!("Contradiction encountered during hidden single");
                    }
                    found = true;
                }
                // Check the digit's occurrence in column peers
                else if self.not_in_peers(board, &board.col_peers[cell], digit) {
                    if !board.assign(&cell, digit) {
                        panic!("Contradiction encountered during hidden single");
                    }
                    found = true;
                }
                // Check the digit's occurrence in box peers
                else if self.not_in_peers(board, &board.box_peers[cell], digit) {
                    if !board.assign(&cell, digit) {
                        panic!("Contradiction encountered during hidden single");
                    }
                    found = true;
                }
            }
        }
        found
    }
    
    // Helper function to check if a digit isn't in peers
    fn not_in_peers(&self, board:&Sudoku, peers: &HashSet<String>, digit: usize) -> bool {
        let peers_with_digit: Vec<&String> = peers.iter()
            .filter(|&peer| board.candidates[peer].contains(&digit))
            .collect();
        peers_with_digit.len() == 0
    }

    fn naked_pair(&self, board: &mut Sudoku) -> bool {
        let mut found = false;
        for cell in &self.cells_with_candidates {
            let candidates = board.candidates[cell].clone();
            // If there are more than two candidates, we can't have a naked pair
            if candidates.len() != 2 {
                continue;
            }
            for unit in vec![&board.row_peers[cell].clone(), &board.col_peers[cell].clone(), &board.box_peers[cell].clone()] {
                // Find other cells in the unit that have the same two candidates
                let other_cells: Vec<_> = unit.iter()
                    .filter(|&cell2| *cell2 != *cell && board.candidates[cell2] == candidates)
                    .cloned()
                    .collect();
                // If there's exactly one cell with the same two candidates, we have a naked pair
                if other_cells.len() != 1 {
                    continue;
                }
                // Eliminate the two digits from all other cells in the unit
                for cell_to_update in unit {
                    if cell_to_update != cell && cell_to_update != &other_cells[0] {
                        for digit in &candidates {
                            if !board.eliminate(cell_to_update, *digit) {
                                panic!("Contradiction encountered during naked pair");
                            }
                            else{
                                found = true;
                            }
                        }
                    }
                }
            }
        }
        found
    }
    
    fn hidden_pair(&self, board: &mut Sudoku) -> bool {
        let mut found = false;
        for cell in &self.cells_with_candidates {
            for unit in vec![board.row_peers[cell].clone(), board.col_peers[cell].clone(), board.box_peers[cell].clone()] {
                for digit1 in 1..9 {
                    for digit2 in (digit1 + 1)..10 {
                        let cells_with_digits: Vec<_> = unit.iter()
                            .filter(|&cell2| board.candidates[cell2].contains(&digit1) || board.candidates[cell2].contains(&digit2))
                            .cloned() // Clone the cells to avoid borrowing `board`
                            .collect();
                        if cells_with_digits.len() == 2 && cells_with_digits.contains(&cell) {
                            for digit in board.candidates[cell].clone() {
                                if digit != digit1 && digit != digit2 {
                                    if !board.eliminate(&cell, digit) {
                                        panic!("Contradiction encountered during hidden pair");
                                    }
                                }
                            }
                            found = true;
                        }
                    }
                }
            }
        }
        found
    }
    
// Locked Candidates Type 1:
fn locked_candidates_type_1(&self, board: &mut Sudoku) -> bool {
    let mut found = false;
    // For each cell on the board that has more than one candidate
    for cell in &self.cells_with_candidates {
        for unit in vec![&board.row_peers[cell].clone(), &board.col_peers[cell].clone(), &board.box_peers[cell].clone()] {
            for digit in 1..=9 {
                let candidate_cells: Vec<_> = unit.iter()
                    .filter(|&cell| board.candidates[cell].contains(&digit))
                    .collect();

                if candidate_cells.is_empty() {
                    continue;
                }

                let rows: HashSet<_> = candidate_cells.iter().map(|cell| cell.chars().next().unwrap()).collect();
                let cols: HashSet<_> = candidate_cells.iter().map(|cell| cell.chars().nth(1).unwrap()).collect();

                if rows.len() == 1 {
                    let row = rows.into_iter().next().unwrap();
                    for cell in unit {
                        if board.candidates[cell].len() == 1 {
                            continue;
                        }
                        if cell.starts_with(row) && !candidate_cells.contains(&cell) && board.candidates[cell].contains(&digit) {
                            board.eliminate(cell, digit);
                            found = true;
                        }
                    }
                } else if cols.len() == 1 {
                    let col = cols.into_iter().next().unwrap();
                    for cell in unit {
                        if board.candidates[cell].len() == 1 {
                            continue;
                        }
                        if cell.ends_with(col) && !candidate_cells.contains(&cell) && board.candidates[cell].contains(&digit) {
                            if !board.eliminate(cell, digit) {
                                panic!("Contradiction encountered during locked candidates type 1");
                            }
                            else {found = true;}
                        }
                    }
                }
            }
        }
    }
    found
}

// Function to implement the Locked Candidates Type 2 rule
fn locked_candidates_type_2(&self, board: &mut Sudoku) -> bool {
    let mut found = false;
    // For each cell on the board that has more than one candidate
    for cell in &self.cells_with_candidates {
        let mut row_inclusive = board.row_peers[cell].clone();
        let mut col_inclusive = board.col_peers[cell].clone();
        row_inclusive.insert(cell.to_string());
        col_inclusive.insert(cell.to_string());
        // For each cell, consider the row and column peers 
        for unit in vec![row_inclusive, col_inclusive] {
            // Check for each digit from 1 to 9
            for digit in 1..=9 {
                // Find the cells in the current unit (row or column) that contain the digit as a candidate
                let candidate_cells: Vec<_> = unit.iter()
                    .filter(|&cell| board.candidates[cell].contains(&digit))
                    .collect();

                // If there are no such cells, move on to the next digit
                if candidate_cells.is_empty() {
                    continue;
                }

                // Check if all candidate cells are in the same box
                let peers = &board.box_peers[candidate_cells[0]].clone();
                // peers.insert(candidate_cells[0].to_string());
                let mut all_in_same_box = true;
                for &cell in &candidate_cells[1..] {
                    if !peers.contains(cell) {
                        all_in_same_box = false;
                        break;
                    }
                }

                if !all_in_same_box {
                    continue;
                }

                // println!("Candidate cells length: {}", candidate_cells.len());
                // println!("Candidate cells: {:?}", candidate_cells);
                // println!("Unit: {:?}", unit);
                // println!("Peers: {:?}", peers);
                // If all candidates are in a single box, get that box
                // Then in that box, eliminate the digit from the cells that are not in the row or column
                for cell in peers {
                    if board.candidates[cell].len() == 1 {
                        continue;
                    }
                    if !candidate_cells.contains(&cell) && board.candidates[cell].contains(&digit) {
                        if !board.eliminate(cell, digit) {
                            println!("{:?}", cell);
                            println!("{:?}", digit);
                            panic!("Contradiction encountered during locked candidates type 2");
                        }
                        else {
                            println!("STOP");
                            found = true;
                        }
                    }
                }
            }
        }
    }
    // If no elimination was possible, the function returns false indicating that no progress was made.
    found
}




    

    // // Complex rules: X-Wing, Swordfish

    // // X-Wing:
    // // Look for two rows (the base sets) with two candidates of the same digit (the fish digit).
    // // If you can find two columns, such that all candidates of the specific digit in both rows
    // // are contained in the columns (the cover sets), all fish candidates in the columns that are not
    // // part of the rows can be eliminated. The result is called an X-Wing in the rows.
    // fn x_wing(&self, board: &mut Sudoku) -> bool {
    //     for digit in 1..=9 {
    //         // Check rows as base sets
    //         if self.x_wing_base_sets(board, digit, |i| (0..9).map(|j| utils::coords_to_cell(i, j)).collect()) {
    //             return true;
    //         }
    //         // Check columns as base sets
    //         if self.x_wing_base_sets(board, digit, |j| (0..9).map(|i| utils::coords_to_cell(i, j)).collect()) {
    //             return true;
    //         }
    //     }
    //     false
    // }
    
    // fn x_wing_base_sets(&self, board: &mut Sudoku, digit: usize, base_set_fn: impl Fn(usize) -> Vec<String>) -> bool {
    //     for i in 0..9 {
    //         let i_base_set = base_set_fn(i);
    //         let i_candidate_indices: Vec<_> = i_base_set.iter().enumerate().filter(|(_, cell)| board.candidates[cell].contains(&digit)).map(|(index, _)| index).collect();
    //         if i_candidate_indices.len() != 2 {
    //             continue;
    //         }
    //         for j in i + 1..9 {
    //             let j_base_set = base_set_fn(j);
    //             let j_candidate_indices: Vec<_> = j_base_set.iter().enumerate().filter(|(_, &cell)| board.candidates[&cell].contains(&digit)).map(|(index, _)| index).collect();
    //             if j_candidate_indices == i_candidate_indices {
    //                 for &index in &i_candidate_indices {
    //                     for k in 0..9 {
    //                         if k != i && k != j {
    //                             let cell = base_set_fn(k)[index].clone();
    //                             if board.candidates[&cell].contains(&digit) {
    //                                 board.eliminate(&cell, digit);
    //                                 return true;
    //                             }
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     }
    //     false
    // }
    

    

    // // Swordfish:
    // // Look for three rows (the base sets) with two or three candidates of the same digit (the fish digit).
    // // If you can find three columns, such that all candidates of the specific digit in the three rows
    // // are contained in the columns (the cover sets), all fish candidates in the columns that are not
    // // part of the rows can be eliminated. The result is called a Swordfish in the rows.
    // fn swordfish(&self, board: &mut Sudoku) -> bool {
    //     for digit in 1..=9 {
    //         // Check rows as base sets
    //         if self.swordfish_base_sets(board, digit, |i| (0..9).map(|j| utils::coords_to_cell(i, j)).collect()) {
    //             return true;
    //         }
    //         // Check columns as base sets
    //         if self.swordfish_base_sets(board, digit, |j| (0..9).map(|i| utils::coords_to_cell(i, j)).collect()) {
    //             return true;
    //         }
    //     }
    //     false
    // }
    
    // fn swordfish_base_sets(&self, board: &mut Sudoku, digit: usize, base_set_fn: impl Fn(usize) -> Vec<String>) -> bool {
    //     let base_sets_with_digit: Vec<_> = (0..9).filter(|&i| base_set_fn(i).iter().any(|cell| board.candidates[cell].contains(&digit))).collect();
    //     if base_sets_with_digit.len() < 3 {
    //         return false;
    //     }
    //     for combo in base_sets_with_digit.into_iter().combinations(3) {
    //         let candidate_indices: Vec<_> = combo.iter().flat_map(|&i| {
    //             let base_set = base_set_fn(i);
    //             base_set.iter().enumerate().filter(move |(_, &cell)| board.candidates[&cell].contains(&digit)).map(|(index, _)| index).collect::<Vec<_>>()
    //         }).collect();            
    //         let unique_candidate_indices: HashSet<_> = candidate_indices.iter().cloned().collect();
    //         if unique_candidate_indices.len() == 3 {
    //             for &index in &unique_candidate_indices {
    //                 for k in 0..9 {
    //                     if !combo.contains(&k) {
    //                         let cell = base_set_fn(k)[index].clone();
    //                         if board.candidates[&cell].contains(&digit) {
    //                             board.eliminate(&cell, digit);
    //                             return true;
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     }
    //     false
    // }
    

    
}

// Crook's algorithm.
// A little tough :( sadge

    
// Stochastic search.

pub struct StochasticSolver {
    temperature: f64,
    cooling_factor: f64,
    units: Vec<Vec<String>>,
    counter: usize,
}

// Uses stochastic search with simulated annealing.
// https://en.wikipedia.org/wiki/Simulated_annealing

impl Solver for StochasticSolver {
    fn solve(&mut self, board: &mut Sudoku) -> bool {
        let mut digit_count = [0; 9];

        // Count the occurrences of each digit in the board
        for row in 0..9 {
            for col in 0..9 {
                let digit = board.board[row][col];
                if digit != 0 {
                    digit_count[(digit - 1) as usize] += 1;
                }
            }
        }
        let mut rng = thread_rng();

        let mut missing_digits = Vec::new();
        let mut extra_digits = Vec::new();

        // Separate missing digits and extra digits
        for (digit, &count) in digit_count.iter().enumerate() {
            let digit_value = (digit + 1) as u8;
            if count < 9 {
                missing_digits.extend(std::iter::repeat(digit_value).take(9 - count));
            } else if count > 9 {
                extra_digits.extend(std::iter::repeat(digit_value).take(count - 9));
            }
        }

        // Shuffle the missing digits randomly
        missing_digits.shuffle(&mut rng);

        println!("{:?}", missing_digits);

        // Replace extra occurrences with missing digits
        let mut index = 0;
        for row in 0..9 {
            for col in 0..9 {
                let digit = board.board[row][col];
                if digit == 0 {
                    board.board[row][col] = missing_digits[index];
                    index += 1;
                }
            }
        }

        let mut score = self.score(board);
        while score > -243 && self.counter < 1000000 {
            let old_board = board.clone();
            let old_score = score;

            self.swap_random(board);
            score = self.score(board);

            if !self.accept(score - old_score) {
                println!("{}", score - old_score);
                println!("Rejecting swap");
                *board = old_board;
                score = old_score;
            }
            self.cool_down();
        }
        println!("Stochastic solver finished.");
        score == -243
    }

    fn name(&self) -> String {
        format!("Stochastic (T={}, cooling={})", self.temperature, self.cooling_factor)
    }

    fn initialize_candidates(&mut self, _board: &mut Sudoku) {
        // unneeded
    }
} 

impl StochasticSolver {
    pub fn new(temperature: f64, cooling_factor: f64, board: Sudoku) -> Self {
        let units: Vec<Vec<String>> = board.cells.iter()
            .flat_map(|cell| vec![board.row_peers[cell].clone(), board.col_peers[cell].clone(), board.box_peers[cell].clone()])
            .map(|unit| unit.iter().cloned().collect())
            .collect();

        let counter = 0;

        StochasticSolver { 
            temperature, 
            cooling_factor,
            units,
            counter
        }
    }

    // First get a random unit,
    // Then get two random cells within that unit, and swap their values.
    fn swap_random(&mut self, board: &mut Sudoku) {
        println!("Swapping random!");
        println!("Board: {:?}", board.board);
        let unit_index = rand::thread_rng().gen_range(0..self.units.len());
        let unit = &self.units[unit_index];
        let mut rng = rand::thread_rng();
        let (i, j) = (rng.gen_range(0..unit.len()), rng.gen_range(0..unit.len()));
        
        let coords_i = utils::cell_to_coords(&unit[i]);
        let coords_j = utils::cell_to_coords(&unit[j]);

        let temp = board.board[coords_i.0][coords_i.1];
        board.board[coords_i.0][coords_i.1] = board.board[coords_j.0][coords_j.1];
        board.board[coords_j.0][coords_j.1] = temp;
        self.counter += 1;
    }

    fn score(&self, board: &Sudoku) -> i32 {
        let mut score = 0;
        for i in 0..9 {
            let row = board.board[i];
            let column: [u8; 9] = (0..9).map(|j| board.board[j][i]).collect::<Vec<u8>>().try_into().unwrap();
            score -= Sudoku::unique_elements(row) + Sudoku::unique_elements(column);
        }
        // New box checking section
        for box_x in 0..3 {
            for box_y in 0..3 {
                let box_values: [u8; 9] = (0..9)
                    .map(|i| board.board[box_x*3 + i/3][box_y*3 + i%3])
                    .collect::<Vec<u8>>().try_into().unwrap();
                score -= Sudoku::unique_elements(box_values);
            }
        }
        println!("Score: {}", score);
        score
    }

    fn cool_down(&mut self) {
        self.temperature *= self.cooling_factor;
    }

    fn accept(&self, delta_s: i32) -> bool {
        if delta_s <= 0 {
            true
        } else {
            let u: f64 = rand::thread_rng().gen();
            println!("u: {}", u);
            println!("delta_s: {}", delta_s);
            println!("temperature: {}", self.temperature);
            println!("exp: {}", (delta_s as f64 / self.temperature).exp());
            (delta_s as f64 / self.temperature).exp() < u
        }
    }
}



// Knuth's Algorithm X, with dancing links.
// This definitely won't work right now, or anytime in the future.
// Due to Rust borrow rules.

// struct Node {
//     row: usize,
//     col: usize,

//     // The size of the column this node is in.
//     size: usize,

//     // Neighboring nodes.
//     up: Option<Rc<RefCell<Node>>>,
//     down: Option<Rc<RefCell<Node>>>,
//     left: Option<Rc<RefCell<Node>>>,
//     right: Option<Rc<RefCell<Node>>>,
// }

// impl Node {
//     fn new(row: usize, col: usize) -> Self {
//         Node {
//             row,
//             col,
//             up: None,
//             down: None,
//             left: None,
//             right: None,
//         }
//     }
// }

// struct Column {
//     node: Rc<RefCell<Node>>,
// }

// impl Column {
//     fn cover(&mut self) {
//         self.node.borrow_mut().right.as_ref().unwrap().borrow_mut().left = self.node.borrow().left.clone();
//         self.node.borrow_mut().left.as_ref().unwrap().borrow_mut().right = self.node.borrow().right.clone();

//         let mut i = self.node.borrow().down.clone();
//         while let Some(node) = i {
//             let mut j = node.borrow().right.clone();
//             while let Some(node_j) = j {
//                 node_j.borrow_mut().down.as_ref().unwrap().borrow_mut().up = node_j.borrow().up.clone();
//                 node_j.borrow_mut().up.as_ref().unwrap().borrow_mut().down = node_j.borrow().down.clone();
//                 node_j.borrow().column.borrow_mut().size -= 1;

//                 j = node_j.borrow().right.clone();
//             }
//             i = node.borrow().down.clone();
//         }
//     }

//     fn uncover(&mut self) {
//         let mut i = self.node.borrow().up.clone();
//         while let Some(node) = i {
//             let mut j = node.borrow().left.clone();
//             while let Some(node_j) = j {
//                 node_j.borrow().column.borrow_mut().size += 1;
//                 node_j.borrow_mut().down.as_ref().unwrap().borrow_mut().up = node_j.clone();
//                 node_j.borrow_mut().up.as_ref().unwrap().borrow_mut().down = node_j.clone();

//                 j = node_j.borrow().left.clone();
//             }
//             i = node.borrow().up.clone();
//         }

//         self.node.borrow_mut().right.as_ref().unwrap().borrow_mut().left = self.node.clone();
//         self.node.borrow_mut().left.as_ref().unwrap().borrow_mut().right = self.node.clone();
//     }
// }

// struct DancingLinks {
//     header: Rc<RefCell<Node>>,
//     columns: Vec<Column>,
// }

// impl DancingLinks {
//     fn new(matrix: &Vec<Vec<bool>>) -> Self {
//         let header = Rc::new(RefCell::new(Node::new(0, 0)));

//         let mut columns = Vec::new();
//         for i in 0..matrix[0].len() {
//             let column_node = Rc::new(RefCell::new(Node::new(0, i)));
//             column_node.borrow_mut().left = Some(if let Some(last_column) = columns.last() {
//                 last_column.node.clone()
//             } else {
//                 header.clone()
//             });

//             columns.push(Column { node: column_node.clone() });

//             if let Some(prev_column_node) = column_node.borrow().left {
//                 prev_column_node.borrow_mut().right = Some(column_node.clone());
//             }
//         }

//         // Link the last column to the header and vice versa
//         columns.last().unwrap().node.borrow_mut().right = Some(header.clone());
//         header.borrow_mut().left = Some(columns.last().unwrap().node.clone());

//         // Create all row nodes and link them to the corresponding column nodes
//         for (i, row) in matrix.iter().enumerate() {
//             let mut last_node_in_row = None;
//             for (j, &value) in row.iter().enumerate() {
//                 if value {
//                     let node = Rc::new(RefCell::new(Node::new(i, j)));
//                     node.borrow_mut().left = last_node_in_row.clone();

//                     let column = &mut columns[j];
//                     column.node.borrow_mut().size += 1;

//                     if let Some(last_node) = last_node_in_row {
//                         last_node.borrow_mut().right = Some(node.clone());
//                         node.borrow_mut().left = Some(last_node.clone());
//                     }

//                     node.borrow_mut().up = Some(column.node.clone());
//                     column.node.borrow_mut().down = Some(node.clone());

//                     last_node_in_row = Some(node);
//                 }
//             }
//         }

//         fn search(&self, k: usize, o: &mut Vec<Rc<RefCell<Node>>>) -> Option<Vec<Rc<RefCell<Node>>>> {
//             if self.header.borrow().right.as_ref().unwrap().borrow().as_ptr() == self.header.borrow().as_ptr() {
//                 return Some(o.clone());
//             } else {
//                 let mut c = self.header.borrow().right.clone();
//                 self.cover(c.borrow().column.borrow_mut());
    
//                 let mut r = c.borrow().down.clone();
//                 while let Some(node_r) = r {
//                     o.push(node_r.clone());
    
//                     let mut j = node_r.borrow().right.clone();
//                     while let Some(node_j) = j {
//                         self.cover(node_j.borrow().column.borrow_mut());
    
//                         j = node_j.borrow().right.clone();
//                     }
    
//                     let result = self.search(k + 1, o);
//                     if result.is_some() {
//                         return result;
//                     }
    
//                     r = o.pop().unwrap();
//                     c = r.borrow().column.clone();
    
//                     let mut j = r.borrow().left.clone();
//                     while let Some(node_j) = j {
//                         self.uncover(node_j.borrow().column.borrow_mut());
    
//                         j = node_j.borrow().left.clone();
//                     }
    
//                     r = r.borrow().down.clone();
//                 }
    
//                 self.uncover(c.borrow().column.borrow_mut());
//             }
    
//             None
//         }
        
//         DancingLinks { header, columns }
//     }
// }
