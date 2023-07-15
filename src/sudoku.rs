use std::collections::{HashMap, HashSet};
use rand::Rng;
mod utils;

// Basic structure of a sudoku board
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

    // Check the unique elements in a given array
    fn unique_elements(arr: [u8; 9]) -> i32 {
        let unique_set: std::collections::HashSet<_> = arr.iter().filter(|&&x| x != 0).collect();
        unique_set.len() as i32
    }
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
                            board.board[row][col] = num; // directly assign num to the cell
                            if self.solve(board) {
                                return true;
                            } else {
                                board.board[row][col] = 0; // directly unassign the cell
                            }
                        }
                    }
                    return false; // backtrack if no candidate number leads to a solution
                }
            }
        }
        true
    }
}

// Constraint programming with forward propagation and backtracking.

pub struct CSPSolver {
    // Store eliminated values for each cell to allow for backtracking
    assignments: HashMap<Cell, HashSet<usize>>,
}

impl CSPSolver {
    // Constructor for CSPSolver
    pub fn new() -> Self {
        Self {
            assignments: HashMap::new(),
        }
    }

    // Function to find the first unassigned variable
    fn select_unassigned_variable(&self, board: &Sudoku) -> Cell {
        for row in 0..9 {
            for col in 0..9 {
                let cell = Cell { row, col };
                if board.board[row][col] == 0 {
                    return cell;
                }
            }
        }
        panic!("No unassigned variable found");
    }

    // Function to unassign a variable
    fn unassign(&mut self, board: &mut Sudoku, cell: &Cell, digit: usize) {
        // Remove digit from the assignments of the cell
        self.assignments.get_mut(cell).unwrap().remove(&digit);

        // Add digit back to the candidates of the cell
        board.candidates.get_mut(cell).unwrap().insert(digit);
    }
}


impl Solver for CSPSolver {
    fn solve(&mut self, board: &mut Sudoku) -> bool {
        // If all variables are assigned, check if the solution is consistent
        if board.is_complete() {
            return board.is_valid();
        }

        // Get the next variable V to assign
        let cell = self.select_unassigned_variable(board);

        // Iterate over the domain of V
        for digit in board.candidates[&cell].iter() {
            if board.is_valid_assignment(&cell, *digit) {
                // Assign the value to V
                board.assign(&cell, *digit);
                self.assignments.get_mut(&cell).unwrap().insert(*digit);

                // Recursively call solve to continue to the next variable
                if self.solve(board) {
                    return true;
                }

                // Unassign the value from V (backtracking step)
                self.unassign(board, &cell, *digit);
            }
        }

        return false;
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
        // Check if the board is solved by verifying that every cell has exactly one candidate
        for row in 0..9 {
            for col in 0..9 {
                let cell = Cell { row, col };
                if let Some(candidates) = board.candidates.get(&cell) {
                    if candidates.len() != 1 {
                        return false;
                    }
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
// A little tough :( sadge

    
// Stochastic search.

pub struct Stochastic {
    temperature: f64,
    cooling_factor: f64,
}

// Uses stochastic search with simulated annealing.
// https://en.wikipedia.org/wiki/Simulated_annealing
impl Stochastic {
    pub fn new(temperature: f64, cooling_factor: f64) -> Self {
        Stochastic { 
            temperature, 
            cooling_factor,
        }
    }

    // Swap two cells randomly within the same unit
    fn swap_random(&self, board: &mut Sudoku) {
        let unit_index = rand::thread_rng().gen_range(0..board.units.len());
        let unit = &board.units[unit_index];
        let mut rng = rand::thread_rng();
        let (i, j) = (rng.gen_range(0..unit.len()), rng.gen_range(0..unit.len()));
        let temp = board.board[unit[i].0][unit[i].1];
        board.board[unit[i].0][unit[i].1] = board.board[unit[j].0][unit[j].1];
        board.board[unit[j].0][unit[j].1] = temp;
    }

    // Define the error function
    fn score(&self, board: &Sudoku) -> i32 {
        let mut score = 0;
        for i in 0..9 {
            let row = board.board[i];
            let column: Vec<u8> = (0..9).map(|j| board.board[j][i]).collect();
            score -= Sudoku::unique_elements(row) + Sudoku::unique_elements(column);
        }
        score
    }

    // Implement the annealing schedule
    fn cool_down(&mut self) {
        self.temperature *= self.cooling_factor;
    }

    // Check if we should accept the new state
    fn accept(&self, delta_s: i32) -> bool {
        if delta_s >= 0 {
            true
        } else {
            let u: f64 = rand::thread_rng().gen();
            (delta_s as f64 / self.temperature).exp() > u
        }
    }
}

impl Solver for Stochastic {
    fn solve(&mut self, board: &mut Sudoku) -> bool {
        let mut score = self.score(board);
        while score > -162 {
            // Store the current board and score
            let old_board = board.clone();
            let old_score = score;

            // Make a random move
            self.swap_random(board);
            
            // Evaluate the new score
            score = self.score(board);
            
            // Decide whether to keep the new state
            if !self.accept(score - old_score) {
                // Restore the old board and score
                *board = old_board;
                score = old_score;
            }

            // Cool down the system
            self.cool_down();
        }
        score == -162
    }
}

// Knuth's Algorithm X, with dancing links.