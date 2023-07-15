fn cross<A: Clone, B: Clone>(a: &[A], b: &[B]) -> Vec<String> {
    let mut result = Vec::new();

    for a in a {
        for b in b {
            result.push(format!("{}{}", a, b));
        }
    }

    return result;
}