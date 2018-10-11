
// Notice that we don't have to re-import `debug-here`. As long
// as we've #[macro_used] it in main.rs (or lib.rs for a library),
// it will work anywhere in the project.
pub fn factorial(n: usize) -> usize {
    let mut res = 1;
    debug_here!();
    for i in 0..n {
        res *= i;
    }
    res
}
