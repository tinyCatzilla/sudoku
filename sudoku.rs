// Basic structure of a sudoku board
use std::collections::{HashMap, HashSet};

pub struct Sudoku {
    board: [[u8; 9]; 9],
    units: HashMap<String, Vec<Vec<String>>>,
    peers: HashMap<String, HashSet<String>>,
    candidates: HashMap<String, HashSet<usize>>,
    //  priority queue of cells ordered by the number of candidates.
}

impl Sudoku {
    // Instantiate
    // squares: A list of strings representing the 81 cells of the Sudoku puzzle.
    // unitlist: A list of lists, where each sub-list is a "unit" that consists of the indices of 9 cells.
    // units: A dictionary that maps each cell to the list of 3 units that the cell belongs to.
    // peers: A dictionary that maps each cell to the set of its 20 peers.
    // candidates: A dictionary that maps each cell to the set of its possible values.
    fn new() -> Self {
        let rows = "ABCDEFGHI".chars().collect::<Vec<_>>();
        let cols = (1..=9).map(|i| i.to_string()).collect::<Vec<_>>();
        let squares = cross(&rows, &cols);

        let unitlist = {
            let mut unitlist = Vec::new();
            // Rows
            for c in &cols {
                unitlist.push(cross(&rows, &[*c]));
            }
            // Columns
            for r in &rows {
                unitlist.push(cross(&[*r], &cols));
            }
            // Boxes
            for rs in vec![&rows[0..3], &rows[3..6], &rows[6..9]] {
                for cs in vec![&cols[0..3], &cols[3..6], &cols[6..9]] {
                    unitlist.push(cross(rs, cs));
                }
            }
            return unitlist;
        };

        let units = squares.iter().map(|s| {
            (s.to_string(), unitlist.iter().filter(|u| u.contains(s)).cloned().collect())
        }).collect();

        let peers = squares.iter().map(|s| {
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

        Sudoku {
            board: [[0; 9]; 9],
            units,
            peers,
            candidates: HashMap::new(),
        }
    }

    // Import sudoku board from a 2D array
    fn import_sudoku(&mut self, board: [[u8; 9]; 9]) {
        self.board = board;
        self.initialize_candidates();
    }

    // Initialize candidates for each cell, given the current board
    fn initialize_candidates(&mut self) {
        for row in 0..9 {
            for col in 0..9 {
                let cell = format!("{}{}", (b'A' + row as u8) as char, col + 1);
                if self.board[row][col] == 0 {
                    // If cell is empty, all numbers are possible candidates
                    self.candidates.insert(cell, (1..=9).collect());
                } else {
                    // If cell has a value, assign it
                    if !self.assign(&cell, self.board[row][col]) {
                        panic!("Contradiction found during initialization");
                    }
                }
            }
        }
    }

    fn assign(&mut self, cell: &str, digit: usize) -> bool {
        // other_values is a set of digits that are not equal to the assigned digit
        let other_values = self.candidates[cell].clone();
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
            if !self.peers[cell].iter().all(|&s2| self.eliminate(s2, d2)) {
                return false;
            }
        }
        
        // Finally, we ensure that for every unit of the cell, the digit has at least one place it can be
        for unit in self.units[cell].iter() {
            let d_places: Vec<_> = unit.iter().filter(|&&s| self.candidates[s].contains(&digit)).cloned().collect();
            // If not, we return false to signal a contradiction
            if d_places.is_empty() {
                return false;
            } 
            // If there is only one such place, we assign the digit there
            else if d_places.len() == 1 {
                if !self.assign(d_places[0], digit) {
                    return false;
                }
            }
        }
        return true;
    }

    // Check if a given number is valid in a given cell
    fn is_valid(&self, row: usize, col: usize, num: usize) -> bool {
        // Convert row and col indices to corresponding cell string
        let cell = format!("{}{}", (b'A' + row as u8) as char, col + 1);

        // Check if num is in the same row, column, or box as the cell
        for &peer in self.peers[&cell].iter() {
            if let Some(val) = self.cells[peer] {
                if val == num {
                    return false;
                }
            }
        }
        return true;
    }
}

fn cross<A: Clone, B: Clone>(a: &[A], b: &[B]) -> Vec<String> {
    let mut result = Vec::new();

    for a in a {
        for b in b {
            result.push(format!("{}{}", a, b));
        }
    }

    return result;
}

pub trait Solver {
    fn solve(&mut self, board: &mut Sudoku) -> bool;
}

pub struct BruteForceSolver;
// Brute force solver.
// This solver will try every possible number in every empty cell.
// If it hits a dead end, it will backtrack and try a different number.

// example usage:
// let mut solver = BruteForceSolver;
// if solver.solve(&mut board) {
//     println!("Solution found:");
//     board.print();
// } else {
//     println!("No solution found.");
// }

impl Solver for BruteForceSolver {
    fn solve(&mut self, board: &mut Sudoku) -> bool {
        for row in 0..9 {
            for col in 0..9 {
                if board.board[row][col] == 0 {
                    for num in 1..=9 {
                        if board.is_valid(row, col, num) {
                            board.board[row][col] = num;
                            if self.solve(board) {
                                return true;
                            } else {
                                board.board[row][col] = 0;
                            }
                        }
                    }
                    return false;
                }
            }
        }
        true
    }
}

pub struct RuleBasedSolver;
// Rule-based solver.
// Note that a naked tuple is accompanied by a hidden pair. So this will implement up to naked/hidden tuples. But not quads.

impl Solver for RuleBasedSolver {
    fn solve(&mut self, board: &mut Sudoku) -> bool {
        // Check each rule in order of complexity and apply if possible.
        // If successful, call the solver again.
        // If no rule can be applied AND the board is not solved, call the brute force solver.
        // If the brute force solver fails, return false.
        // If the brute force solver succeeds, return true.

        if self.apply_basic_rules(board) {
            return self.solve(board);
        } else if self.apply_intermediate_rules(board) {
            return self.solve(board);
        } else if self.apply_complex_rules(board) {
            return self.solve(board);
        } else {
            if board.solved() {
                return true;
            } else {
                let mut brute_force_solver = BruteForceSolver;
                return brute_force_solver.solve(board);
            }
        }
    }

    fn apply_basic_rules(&self, board: &mut Sudoku) -> bool {
        // Apply basic rules here: Naked Single, Hidden Single, Naked Pair, Hidden Pair
        // Returns true if a rule could be applied, false otherwise
        // When any rule succeeds, call the solver again

        if self.naked_single(board) {
            return true;
        }
        if self.hidden_single(board) {
            return true;
        }
        unimplemented!()
        return false;
    }

    fn apply_intermediate_rules(&self, board: &mut Sudoku) -> bool {
        // Apply intermediate rules here: Locked Candidates Type 1 and Type 2
        // Returns true if a rule could be applied, false otherwise
        unimplemented!()
        return false;
    }

    fn apply_complex_rules(&self, board: &mut Sudoku) -> bool {
        // Apply complex rules here: X-Wing, Swordfish
        // Returns true if a rule could be applied, false otherwise
        unimplemented!()
        return false;
    }

    fn solved(&self, board: &Sudoku) -> bool {
        // Check if the board is solved
        for row in 0..9 {
            for col in 0..9 {
                if board.board[row][col] == 0 {
                    return false;
                }
            }
        }
        true
}

impl RuleBasedSolver {
    
    fn naked_single(&self, board: &mut Sudoku) -> bool {
        for (cell, candidates) in board.candidates.iter() {
            if candidates.len() == 1 {
                let val = *candidates.iter().next().unwrap();
                if !board.assign(cell, val) {
                    panic!("Contradiction encountered during naked single");
                }
                return true;
            }
        }
        false
    }

    fn hidden_single(&self, board: &mut Sudoku) -> bool {
        for unit in board.units.values() {
            for digit in 1..=9 {
                let digit_occurrences: Vec<_> = unit.iter().filter(|&&cell| board.candidates[cell].contains(&digit)).cloned().collect();
                if digit_occurrences.len() == 1 {
                    if !board.assign(&digit_occurrences[0], digit) {
                        panic!("Contradiction encountered during hidden single");
                    }
                    return true;
                }
            }
        }
        false
    }

    fn naked_pair(&self, board: &mut Sudoku) -> bool {
        for unit in board.units.values() {
            for i in 0..8 {
                for j in (i+1)..9 {
                    let cell1 = unit[i];
                    let cell2 = unit[j];
                    if board.candidates[cell1] == board.candidates[cell2] && board.candidates[cell1].len() == 2 {
                        let other_cells: Vec<_> = unit.iter().filter(|&&c| c != cell1 && c != cell2).cloned().collect();
                        for &digit in &board.candidates[cell1] {
                            for &cell in &other_cells {
                                if !board.eliminate(cell, digit) {
                                    panic!("Contradiction encountered during naked pair");
                                }
                            }
                        }
                        return true;
                    }
                }
            }
        }
        false
    }

    fn hidden_pair(&self, board: &mut Sudoku) -> bool {
        for unit in board.units.values() {
            for digit1 in 1..9 {
                for digit2 in (digit1+1)..10 {
                    let cells_with_digits: Vec<_> = unit.iter().filter(|&&cell| board.candidates[cell].contains(&digit1) || board.candidates[cell].contains(&digit2)).cloned().collect();
                    if cells_with_digits.len() == 2 {
                        for &cell in &cells_with_digits {
                            let other_digits: Vec<_> = (1..10).filter(|&d| d != digit1 && d != digit2).collect();
                            for &digit in &other_digits {
                                if !board.eliminate(cell, digit) {
                                    panic!("Contradiction encountered during hidden pair");
                                }
                            }
                        }
                        return true;
                    }
                }
            }
        }
        false
    }
}



// Crook's algorithm.

// Stochastic search.

// Constraint programming.

// Knuth's Algorithm X, with dancing links.