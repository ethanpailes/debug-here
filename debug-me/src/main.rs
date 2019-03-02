#[macro_use] extern crate debug_here;

mod another_module;

fn factorial(n: usize) -> usize {
    let mut res = 1;
    // Try changing the debugger backend by replacing this with
    //
    // debug_here!(lldb);
    //
    // or
    //
    // debug_here!(gdb);
    debug_here!();
    for i in 0..n {
        res *= i;
    }
    res
}

fn main() {
    // only the first of these will fire, but try commenting out the
    // call to our local version of broken `factorial`.

    println!("The factorial of 5 is {}!", factorial(5));

    println!("The factorial of 4 is {}!", another_module::factorial(4));
}
