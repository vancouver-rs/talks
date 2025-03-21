
/// An iterative implementation of fib()
pub fn fib_iterative(n: u64) -> u64 {
    if n < 2 {
        return 1;
    }

    let mut current = 1;
    let mut prev = 1;

    for _ in 2..=n {
        let next = current + prev;
        prev = current;
        current = next;
    }

    current
}
