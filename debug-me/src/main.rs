#[macro_use] extern crate debug_here;

fn factorial(n: usize) -> usize {
    let mut res = 1;
    debug_here!();
    for i in 0..n {
        res *= i;
    }
    res
}

fn main() {
    println!("The factorial of 5 is {}!", factorial(5));
}
