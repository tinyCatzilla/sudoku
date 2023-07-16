use std::collections::{HashMap, HashSet};
use std::collections::LinkedList;
use std::cell::RefCell;
use std::rc::Rc;
use std::clone::Clone;
use rand::Rng;
use core::cell::Cell;
use crate::utils;
use itertools::Itertools;

// Basic structure of a sudoku board
#[derive(Clone)]
pub struct Sudoku {
    board: [[u8; 9]; 9],
    units: HashMap<String, Vec<HashSet<String>>>,
    peers: HashMap<String, HashSet<String>>,
    candidates: HashMap<String, HashSet<usize>>,
    // TODO? priority queue of cells ordered by the number of candidates.
}

impl Sudoku {
    // Instantiate
    // squares: A list of strings representing the 81 cells of the Sudoku puzzle.
    // unitlist: A list of lists, where each sub-list is a "unit" that consists of the indices of 9 cells.
    // units: A hashmap that maps each cell to the list of 3 units that the cell belongs to.
    // peers: A hashmap that maps each cell to the set of its 20 peers (cells sharing a unit).
    // candidates: A hashmap that maps each cell to the set of its possible values.
    pub fn new(puzzle: Option<&str>) -> Result<Self, &str> {
        let rows = "ABCDEFGHI".chars().collect::<Vec<_>>();
        let cols = (1..=9).map(|i| i.to_string()).collect::<Vec<_>>();
        let squares = utils::cross(&rows, &cols);

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

        let units: HashMap<String, Vec<HashSet<String>>> = squares.iter().map(|s| {
            (s.to_string(), unitlist.iter().filter(|u| u.contains(s)).map(|u| u.clone().into_iter().collect()).collect())
        }).collect();        

        let peers: HashMap<String, HashSet<String>> = squares.iter().map(|s| {
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
            units,
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
        for row in 0..9 {
            for col in 0..9 {
                let cell = utils::coords_to_cell(row, col);
                if self.board[row][col] == 0 {
                    // If cell is empty, all numbers are possible candidates
                    self.candidates.insert(cell, (1..=9).collect());
                } else {
                    // If cell has a value, assign it
                    if !self.assign(&cell, self.board[row][col].into()) {
                        panic!("Contradiction found during initialization");
                    }
                }
            }
        }
    }

    fn assign(&mut self, cell: &str, digit: usize) -> bool {
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
        // If the digit is not a candidate, we do nothing and return true
        if !self.candidates[cell].contains(&digit) {
            return true;
        }
        // Otherwise, we remove the digit from the candidates
        self.candidates.get_mut(cell).unwrap().remove(&digit);
        
        // If the cell has no remaining candidates, we return false to signal a contradiction
        if self.candidates[cell].is_empty() {
            return false;
        } 
        // If the cell has one remaining candidate, we need to eliminate this digit from all peers
        else if self.candidates[cell].len() == 1 {
            let d2 = *self.candidates[cell].iter().next().unwrap();
            // If elimination from any peer results in a contradiction, we return false
            if !self.peers[cell].iter().all(|s2| self.eliminate(&s2, d2)) {
                return false;
            }
        }
        
        // Finally, we ensure that for every unit of the cell, the digit has at least one place it can be
        for unit in self.units[cell].iter() {
            let d_places: Vec<_> = unit.iter().filter(|&s| self.candidates[s].contains(&digit)).cloned().collect();
            // If not, we return false to signal a contradiction
            if d_places.is_empty() {
                return false;
            } 
            // If there is only one such place, we assign the digit there
            else if d_places.len() == 1 {
                if !self.assign(&d_places[0], digit) {
                    return false;
                }
            }
        }
        return true;
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
    
        empty_cells.sort_by_key(|&(row, col)| board.candidates[&utils::coords_to_cell(row, col)].len());
    
        if empty_cells.is_empty() {
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
}

impl BruteForceSolver {
    pub fn new() -> BruteForceSolver {
        BruteForceSolver
    }
}

// Constraint programming with forward propagation and backtracking.

// pub struct CSPSolver<T> {
//     // Store eliminated values for each cell to allow for backtracking
//     assignments: HashMap<Cell<T>, HashSet<usize>>,
// }

// impl CSPSolver {
//     // Constructor for CSPSolver
//     pub fn new() -> Self {
//         Self {
//             assignments: HashMap::new(),
//         }
//     }

//     // Function to find the first unassigned variable
//     fn select_unassigned_variable(&self, board: &Sudoku) -> Cell<T> {
//         for row in 0..9 {
//             for col in 0..9 {
//                 let cell = Cell { row, col };
//                 if board.board[row][col] == 0 {
//                     return cell;
//                 }
//             }
//         }
//         panic!("No unassigned variable found");
//     }

//     // Function to unassign a variable
//     fn unassign(&mut self, board: &mut Sudoku, cell: &Cell<T>, digit: usize) {
//         // Remove digit from the assignments of the cell
//         self.assignments.get_mut(cell).unwrap().remove(&digit);

//         // Add digit back to the candidates of the cell
//         board.candidates.get_mut(cell).unwrap().insert(digit);
//     }
// }


// impl Solver for CSPSolver {
//     fn solve(&mut self, board: &mut Sudoku) -> bool {
//         // If all variables are assigned, check if the solution is consistent
//         if board.is_complete() {
//             return board.is_valid();
//         }

//         // Get the next variable V to assign
//         let cell = self.select_unassigned_variable(board);

//         // Iterate over the domain of V
//         for digit in board.candidates[&cell].iter() {
//             if board.is_valid_assignment(&cell, *digit) {
//                 // Assign the value to V
//                 board.assign(&cell, *digit);
//                 self.assignments.get_mut(&cell).unwrap().insert(*digit);

//                 // Recursively call solve to continue to the next variable
//                 if self.solve(board) {
//                     return true;
//                 }

//                 // Unassign the value from V (backtracking step)
//                 self.unassign(board, &cell, *digit);
//             }
//         }

//         return false;
//     }

//     fn name(&self) -> String {
//         "Constraint Programming Solver".to_string()
//     }
// }


pub struct RuleBasedSolver;
// Rule-based solver.
// Note that a naked tuple is accompanied by a hidden pair. So this will implement up to naked/hidden tuples. But not quads.

impl Solver for RuleBasedSolver {
    fn solve(&mut self, board: &mut Sudoku) -> bool {
        if self.apply_basic_rules(board) {
            return self.solve(board);
        } else if self.apply_intermediate_rules(board) {
            return self.solve(board);
        } else if self.apply_complex_rules(board) {
            return self.solve(board);
        } else {
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
                return true;
            } else {
                let mut brute_force_solver = BruteForceSolver;
                return brute_force_solver.solve(board);
            }
        }
    }

    fn name(&self) -> String {
        "Rule Based Solver".to_string()
    }
}

impl RuleBasedSolver {
    pub fn new() -> RuleBasedSolver {
        RuleBasedSolver
    }
    
    fn apply_basic_rules(&self, board: &mut Sudoku) -> bool {
        // Apply basic rules here: Naked Single, Hidden Single, Naked Pair, Hidden Pair
        // Returns true if a rule could be applied, false otherwise
        // When any rule succeeds, call the solver again

        // if self.naked_single(board) {
        //     return true;
        // }
        if self.hidden_single(board) {
            return true;
        }
        if self.naked_pair(board) {
            return true;
        }
        if self.hidden_pair(board) {
            return true;
        }
        false
    }

    fn apply_intermediate_rules(&self, board: &mut Sudoku) -> bool {
        // Apply intermediate rules here: Locked Candidates Type 1 and Type 2
        // Returns true if a rule could be applied, false otherwise

        if self.locked_candidates_type_1(board) {
            return true;
        }
        if self.locked_candidates_type_2(board) {
            return true;
        }
        false
    }

    fn apply_complex_rules(&self, board: &mut Sudoku) -> bool {
        // Apply complex rules here: X-Wing, Swordfish
        // Returns true if a rule could be applied, false otherwise

        if self.x_wing(board) {
            return true;
        }
        if self.swordfish(board) {
            return true;
        }
        false
    }

    fn solved(&self, board: &Sudoku) -> bool {
        // Check if the board is solved by verifying that every cell has exactly one candidate
        for row in 0..9 {
            for col in 0..9 {
                let cell = utils::coords_to_cell(row, col);
                if let Some(candidates) = board.candidates.get(&cell) {
                    if candidates.len() != 1 {
                        return false;
                    }
                }
            }
        }
        true
    }

    // Basic rules: Naked Single, Hidden Single, Naked Pair, Hidden Pair
    
    // fn naked_single(&self, board: &mut Sudoku) -> bool {
    //     for (cell, candidates) in board.candidates.iter() {
    //         if candidates.len() == 1 {
    //             let val = *candidates.iter().next().unwrap();
    //             if !board.assign(cell, val) {
    //                 panic!("Contradiction encountered during naked single");
    //             }
    //             return true;
    //         }
    //     }
    //     false
    // }

    fn hidden_single(&self, board: &mut Sudoku) -> bool {
        // Get keys first without holding reference to board
        let keys: Vec<String> = board.units.keys().cloned().collect();
    
        for cell in keys {
            for digit in 1..=9 {
                // Now we can safely borrow `board.units` again
                let digit_occurrences: Vec<_> = board.units[&cell].iter().filter(|unit| unit.contains(&cell) && board.candidates[&cell].contains(&digit)).cloned().collect();
                if digit_occurrences.len() == 1 {
                    if !board.assign(&cell, digit) {
                        panic!("Contradiction encountered during hidden single");
                    }
                    return true;
                }
            }
        }
        false
    }
    

    fn naked_pair(&self, board: &mut Sudoku) -> bool {
        for (cell, units) in &board.units {
            let candidates = &board.candidates[cell];
            if candidates.len() != 2 {
                continue;
            }
            for unit in units {
                let other_cells: Vec<_> = unit.iter()
                    .filter(|&cell2| *cell2 != *cell && board.candidates[cell2] == *candidates)
                    .cloned() // Clone the cells to avoid borrowing `board`
                    .collect();
                if other_cells.len() == 1 {
                    for &digit in candidates {
                        if !board.eliminate(&other_cells[0], digit) {
                            panic!("Contradiction encountered during naked pair");
                        }
                    }
                    return true;
                }
            }
        }
        false
    }
    
    fn hidden_pair(&self, board: &mut Sudoku) -> bool {
        for (cell, units) in &board.units {
            for unit in units {
                for digit1 in 1..9 {
                    for digit2 in (digit1 + 1)..10 {
                        let cells_with_digits: Vec<_> = unit.iter()
                            .filter(|&cell2| board.candidates[cell2].contains(&digit1) || board.candidates[cell2].contains(&digit2))
                            .cloned() // Clone the cells to avoid borrowing `board`
                            .collect();
                        if cells_with_digits.len() == 2 && cells_with_digits.contains(&cell) {
                            for &digit in &board.candidates[cell] {
                                if digit != digit1 && digit != digit2 {
                                    if !board.eliminate(&cell, digit) {
                                        panic!("Contradiction encountered during hidden pair");
                                    }
                                }
                            }
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
    
    

    // Intermediate rules: Locked Candidates Type 1 and Type 2

    // Locked Candidates Type 1:
    // If in a box all candidates of a certain digit are confined to a row or column,
    // that digit cannot appear outside of that box in that row or column.
    fn locked_candidates_type_1(&self, board: &mut Sudoku) -> bool {
        // Iterate through each unit in the Sudoku board
        for unit in board.units.values().flatten() {
            // Check for each digit from 1 to 9
            for digit in 1..=9 {
                // Find the cells in the current unit that contain the digit as a candidate
                let candidate_cells: Vec<_> = unit.iter().filter(|&cell| board.candidates[cell].contains(&digit)).collect();
                
                // If there are no such cells, move on to the next digit
                if candidate_cells.is_empty() {
                    continue;
                }

                // Determine the rows and columns in which the candidates are located
                let rows: HashSet<_> = candidate_cells.iter().map(|&cell| cell.chars().next().unwrap()).collect();
                let cols: HashSet<_> = candidate_cells.iter().map(|&cell| cell.chars().nth(1).unwrap()).collect();

                // If all candidates are in a single row
                if rows.len() == 1 {
                    let row = rows.into_iter().next().unwrap();
                    // Then in that row, eliminate the digit from the peers of the box that are not in the box
                    for cell in board.peers.keys() {
                        if cell.starts_with(row) && !unit.contains(cell) && board.candidates[cell].contains(&digit) {
                            board.eliminate(cell, digit);
                            return true;
                        }
                    }
                // If all candidates are in a single column
                } else if cols.len() == 1 {
                    let col = cols.into_iter().next().unwrap();
                    // Then in that column, eliminate the digit from the peers of the box that are not in the box
                    for cell in board.peers.keys() {
                        if cell.ends_with(col) && !unit.contains(cell) && board.candidates[cell].contains(&digit) {
                            board.eliminate(cell, digit);
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    

    // Locked Candidates Type 2:
    // If in a row (or column) all candidates of a certain digit are confined to one box,
    // that candidate that be eliminated from all other cells in that box.
    fn locked_candidates_type_2(&self, board: &mut Sudoku) -> bool {
        // Iterate through each unit in the Sudoku board
        for unit in board.units.values().flatten() {
            // Check for each digit from 1 to 9
            for digit in 1..=9 {
                // Find the cells in the current unit that contain the digit as a candidate
                let candidate_cells: Vec<_> = unit.iter().filter(|&cell| board.candidates[cell].contains(&digit)).collect();
                
                // If there are no such cells, move on to the next digit
                if candidate_cells.is_empty() {
                    continue;
                }

                // Determine the boxes in which the candidates are located
                let blocks: Vec<_> = candidate_cells.iter().map(|cell| {
                    board.units[*cell].iter().filter(|unit| unit.len() == 9 && unit.iter().any(|cell| cell.chars().nth(2).is_some())).collect::<Vec<_>>()
                }).flatten().collect();

                // If all candidates are in a single box
                if blocks.len() == 1 {
                    let block = blocks[0].clone();
                    // Then in that box, eliminate the digit from the cells that are not in the row or column
                    for cell in block {
                        if !unit.contains(&cell) && board.candidates[&cell].contains(&digit) {
                            board.eliminate(&cell, digit);
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    

    // Complex rules: X-Wing, Swordfish

    // X-Wing:
    // Look for two rows (the base sets) with two candidates of the same digit (the fish digit).
    // If you can find two columns, such that all candidates of the specific digit in both rows
    // are contained in the columns (the cover sets), all fish candidates in the columns that are not
    // part of the rows can be eliminated. The result is called an X-Wing in the rows.
    fn x_wing(&self, board: &mut Sudoku) -> bool {
        for digit in 1..=9 {
            // Check rows as base sets
            if self.x_wing_base_sets(board, digit, |i| (0..9).map(|j| utils::coords_to_cell(i, j)).collect()) {
                return true;
            }
            // Check columns as base sets
            if self.x_wing_base_sets(board, digit, |j| (0..9).map(|i| utils::coords_to_cell(i, j)).collect()) {
                return true;
            }
        }
        false
    }
    
    fn x_wing_base_sets(&self, board: &mut Sudoku, digit: usize, base_set_fn: impl Fn(usize) -> Vec<String>) -> bool {
        for i in 0..9 {
            let i_base_set = base_set_fn(i);
            let i_candidate_indices: Vec<_> = i_base_set.iter().enumerate().filter(|(_, &cell)| board.candidates[&cell].contains(&digit)).map(|(index, _)| index).collect();
            if i_candidate_indices.len() != 2 {
                continue;
            }
            for j in i + 1..9 {
                let j_base_set = base_set_fn(j);
                let j_candidate_indices: Vec<_> = j_base_set.iter().enumerate().filter(|(_, &cell)| board.candidates[&cell].contains(&digit)).map(|(index, _)| index).collect();
                if j_candidate_indices == i_candidate_indices {
                    for &index in &i_candidate_indices {
                        for k in 0..9 {
                            if k != i && k != j {
                                let cell = base_set_fn(k)[index].clone();
                                if board.candidates[&cell].contains(&digit) {
                                    board.eliminate(&cell, digit);
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }
    

    

    // Swordfish:
    // Look for three rows (the base sets) with two or three candidates of the same digit (the fish digit).
    // If you can find three columns, such that all candidates of the specific digit in the three rows
    // are contained in the columns (the cover sets), all fish candidates in the columns that are not
    // part of the rows can be eliminated. The result is called a Swordfish in the rows.
    fn swordfish(&self, board: &mut Sudoku) -> bool {
        for digit in 1..=9 {
            // Check rows as base sets
            if self.swordfish_base_sets(board, digit, |i| (0..9).map(|j| utils::coords_to_cell(i, j)).collect()) {
                return true;
            }
            // Check columns as base sets
            if self.swordfish_base_sets(board, digit, |j| (0..9).map(|i| utils::coords_to_cell(i, j)).collect()) {
                return true;
            }
        }
        false
    }
    
    fn swordfish_base_sets(&self, board: &mut Sudoku, digit: usize, base_set_fn: impl Fn(usize) -> Vec<String>) -> bool {
        let base_sets_with_digit: Vec<_> = (0..9).filter(|&i| base_set_fn(i).iter().any(|cell| board.candidates[cell].contains(&digit))).collect();
        if base_sets_with_digit.len() < 3 {
            return false;
        }
        for combo in base_sets_with_digit.into_iter().combinations(3) {
            let candidate_indices: Vec<_> = combo.iter().flat_map(|&i| base_set_fn(i).iter().enumerate().filter(move |(_, &cell)| board.candidates[&cell].contains(&digit)).map(|(index, _)| index)).collect();
            let unique_candidate_indices: HashSet<_> = candidate_indices.iter().cloned().collect();
            if unique_candidate_indices.len() == 3 {
                for &index in &unique_candidate_indices {
                    for k in 0..9 {
                        if !combo.contains(&k) {
                            let cell = base_set_fn(k)[index].clone();
                            if board.candidates[&cell].contains(&digit) {
                                board.eliminate(&cell, digit);
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }
    

    
}

// Crook's algorithm.
// A little tough :( sadge

    
// Stochastic search.

pub struct StochasticSolver {
    temperature: f64,
    cooling_factor: f64,
    units: Vec<Vec<String>>,
}

// Uses stochastic search with simulated annealing.
// https://en.wikipedia.org/wiki/Simulated_annealing

impl Solver for StochasticSolver {
    fn solve(&mut self, board: &mut Sudoku) -> bool {
        let mut score = self.score(board);
        while score > -162 {
            let old_board = board.clone();
            let old_score = score;

            self.swap_random(board);
            score = self.score(board);

            if !self.accept(score - old_score) {
                *board = old_board;
                score = old_score;
            }

            self.cool_down();
        }
        score == -162
    }

    fn name(&self) -> String {
        format!("Stochastic (T={}, cooling={})", self.temperature, self.cooling_factor)
    }
} 

impl StochasticSolver {
    pub fn new(temperature: f64, cooling_factor: f64, board: &Sudoku) -> Self {
        let units: Vec<Vec<String>> = board.units.values()
            .map(|unit| unit.iter().flat_map(HashSet::iter).cloned().collect())
            .collect();

        StochasticSolver { 
            temperature, 
            cooling_factor,
            units,
        }
    }

    // First get a random unit,
    // Then get two random cells within that unit, and swap their values.
    fn swap_random(&self, board: &mut Sudoku) {
        let unit_index = rand::thread_rng().gen_range(0..self.units.len());
        let unit = &self.units[unit_index];
        let mut rng = rand::thread_rng();
        let (i, j) = (rng.gen_range(0..unit.len()), rng.gen_range(0..unit.len()));
        
        let coords_i = utils::cell_to_coords(&unit[i]);
        let coords_j = utils::cell_to_coords(&unit[j]);

        let temp = board.board[coords_i.0][coords_i.1];
        board.board[coords_i.0][coords_i.1] = board.board[coords_j.0][coords_j.1];
        board.board[coords_j.0][coords_j.1] = temp;
    }

    fn score(&self, board: &Sudoku) -> i32 {
        let mut score = 0;
        for i in 0..9 {
            let row = board.board[i];
            let column: [u8; 9] = (0..9).map(|j| board.board[j][i]).collect::<Vec<u8>>().try_into().unwrap();
            score -= Sudoku::unique_elements(row) + Sudoku::unique_elements(column);
        }
        score
    }

    fn cool_down(&mut self) {
        self.temperature *= self.cooling_factor;
    }

    fn accept(&self, delta_s: i32) -> bool {
        if delta_s >= 0 {
            true
        } else {
            let u: f64 = rand::thread_rng().gen();
            (delta_s as f64 / self.temperature).exp() > u
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
