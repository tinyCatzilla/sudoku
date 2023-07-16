pub fn cross<A: Clone  + std::fmt::Display, B: Clone + std::fmt::Display>(a: &[A], b: &[B]) -> Vec<String> {
    let mut result = Vec::new();

    for a in a {
        for b in b {
            result.push(format!("{}{}", a, b));
        }
    }

    return result;
}

pub fn coords_to_cell(row: usize, col: usize) -> String {
    format!("{}{}", (b'A' + row as u8) as char, col + 1)
}

pub fn cell_to_coords(cell: &str) -> (usize, usize) {
    let row = cell.chars().nth(0).unwrap() as usize - 'A' as usize;
    let col = cell.chars().nth(1).unwrap().to_digit(10).unwrap() as usize - 1;
    (row, col)
}
