
/// A recursive implementation of fib()
pub fn fib_recursive(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fib_recursive(n-1) + fib_recursive(n-2),
    }
}
