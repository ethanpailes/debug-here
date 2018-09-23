
#[macro_use] extern crate debug_here;

fn main() {
    foo();
}

fn foo() {
    let mut a_var = 14;

    for _ in 0..10 {
        debug_here!();
        a_var += 1;
    }

    println!("a_var is {}", a_var);
}

