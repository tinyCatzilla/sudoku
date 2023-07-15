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
// This solver will try every possible candidate in every empty cell.
// If it hits a dead end, it will backtrack and try a different candidate.

impl Solver for BruteForceSolver {
    fn solve(&mut self, board: &mut Sudoku) -> bool {
        for row in 0..9 {
            for col in 0..9 {
                let cell = Cell { row, col };
                if board.board[row][col] == 0 {
                    // Iterate over the candidates
                    let candidates = board.candidates.get(&cell).unwrap().clone();
                    for num in candidates {
                        if board.is_valid(row, col, num) {
                            board.assign(&cell, num);
                            if self.solve(board) {
                                return true;
                            } else {
                                board.unassign(&cell, num);
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
}

impl RuleBasedSolver {

    // Basic rules: Naked Single, Hidden Single, Naked Pair, Hidden Pair
    
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

    // Intermediate rules: Locked Candidates Type 1 and Type 2

    fn locked_candidates_type_1(&self, board: &mut Sudoku) -> bool {
        for block in board.blocks.iter() {
            for digit in 1..=9 {
                let candidate_cells: Vec<_> = block.iter().filter(|&&cell| board.candidates[cell].contains(&digit)).collect();
                if candidate_cells.is_empty() {
                    continue;
                }

                let rows: HashSet<_> = candidate_cells.iter().map(|&&cell| cell.0).collect();
                let cols: HashSet<_> = candidate_cells.iter().map(|&&cell| cell.1).collect();

                if rows.len() == 1 {
                    let row = rows.into_iter().next().unwrap();
                    for col in 0..9 {
                        let cell = (row, col);
                        if !block.contains(&cell) && board.candidates[cell].contains(&digit) {
                            board.eliminate(cell, digit);
                            return true;
                        }
                    }
                } else if cols.len() == 1 {
                    let col = cols.into_iter().next().unwrap();
                    for row in 0..9 {
                        let cell = (row, col);
                        if !block.contains(&cell) && board.candidates[cell].contains(&digit) {
                            board.eliminate(cell, digit);
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn locked_candidates_type_2(&self, board: &mut Sudoku) -> bool {
        for row in 0..9 {
            for digit in 1..=9 {
                let candidate_cells: Vec<_> = (0..9).filter(|&col| board.candidates[(row, col)].contains(&digit)).map(|col| (row, col)).collect();
                if candidate_cells.is_empty() {
                    continue;
                }

                let blocks: HashSet<_> = candidate_cells.iter().map(|&cell| board.cell_to_block(cell)).collect();

                if blocks.len() == 1 {
                    let block = blocks.into_iter().next().unwrap();
                    for &cell in block {
                        if cell.0 != row && board.candidates[cell].contains(&digit) {
                            board.eliminate(cell, digit);
                            return true;
                        }
                    }
                }
            }
        }

        for col in 0..9 {
            for digit in 1..=9 {
                let candidate_cells: Vec<_> = (0..9).filter(|&row| board.candidates[(row, col)].contains(&digit)).map(|row| (row, col)).collect();
                if candidate_cells.is_empty() {
                    continue;
                }

                let blocks: HashSet<_> = candidate_cells.iter().map(|&cell| board.cell_to_block(cell)).collect();

                if blocks.len() == 1 {
                    let block = blocks.into_iter().next().unwrap();
                    for &cell in block {
                        if cell.1 != col && board.candidates[cell].contains(&digit) {
                            board.eliminate(cell, digit);
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    // Complex rules: X-Wing, Swordfish

    fn x_wing(&self, board: &mut Sudoku) -> bool {
        for digit in 1..=9 {
            for i in 0..9 {
                let i_candidate_cols: Vec<_> = (0..9).filter(|&col| board.candidates[(i, col)].contains(&digit)).collect();
                if i_candidate_cols.len() != 2 {
                    continue;
                }

                for j in i + 1..9 {
                    let j_candidate_cols: Vec<_> = (0..9).filter(|&col| board.candidates[(j, col)].contains(&digit)).collect();

                    if j_candidate_cols == i_candidate_cols {
                        for &col in &i_candidate_cols {
                            for row in 0..9 {
                                let cell = (row, col);
                                if row != i && row != j && board.candidates[cell].contains(&digit) {
                                    board.eliminate(cell, digit);
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

    fn swordfish(&self, board: &mut Sudoku) -> bool {
        for digit in 1..=9 {
            let rows_with_digit: Vec<_> = (0..9).filter(|&row| (0..9).any(|col| board.candidates[(row, col)].contains(&digit))).collect();

            if rows_with_digit.len() < 3 {
                continue;
            }

            for combo in rows_with_digit.into_iter().combinations(3) {
                let candidate_cols: Vec<_> = combo.iter().flat_map(|&row| (0..9).filter(move |&col| board.candidates[(row, col)].contains(&digit))).collect();
                let unique_candidate_cols: HashSet<_> = candidate_cols.iter().cloned().collect();

                if unique_candidate_cols.len() == 3 {
                    for &col in &unique_candidate_cols {
                        for row in 0..9 {
                            let cell = (row, col);
                            if !combo.contains(&row) && board.candidates[cell].contains(&digit) {
                                board.eliminate(cell, digit);
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

// Stochastic search.

// Constraint programming.

// Knuth's Algorithm X, with dancing links.