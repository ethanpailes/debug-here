# debug-here: a rust-gdb hook, inspired by pdb.set_trace()

### The Problem

Debuggers are a great way to examine the state of a program
you are trying to understand, but actually beginning to use
one can have a bit of ceremony involved. You have to figure
out the name of the executable you care about. This can be
harder than it might initially seem if you are running
your program through several layers of scripts that someone else
wrote, or even that you wrote. Once you've got your executable
name you have to start it with `rust-gdb exe` (if you use a wrapper
script, you have to arrange for your script to do so). Now that
you've started your program under gdb you have to set your break
point. `b filename.rs:lineno` works if the code has been statically
linked (fortunately it usually has in rust), but you might have to
first do `b main; run; b filename.rs:lineno; c` instead. Now you
are ready to debug. It's not that much work, but I find myself
doing it a fair amount, so it can get annoying.

One alternative is to arrange for your program to enter some sort
of sleep or looping state, rummage around for the PID on the command
line, then attach with `gdb -pid`. Usually this ends up being a bit
more work (again, not much work, but enough to be annoying), so people
only do it if it is not convenient to launch the program in question
from the terminal.

### Isn't Python Slick?

Python is my goto scripting language, and by far my favorite thing
about the language is `pdb`, the debugger that lives in its standard
library. My development workflow for python involves writing the
script up to the point where I no longer know the APIs that I need
to use, then dropping a `pdb.set_trace()` call into my code and
running it. Python automatically drops me into a full REPL with all
the program state to that point just lying around. I poke around for
a bit, then write the next chunk of code. You might call this debugger
driven development. When I'm debugging python I do something similar
by inserting `pdb.set_trace()` directly into my code wherever something
seems interesting. It's the single most ergonomic language or ecosystem
feature that I've ever used.

### Making getting to the debugger easier in rust

`rust-gdb` isn't yet powerful enough to support the sort of debugger
driven development that I love to use in Python, but it's not that
hard to reduce the amount of effort required to enter a debugger.
This crate just automates the process of convincing your program to
wait around for a debugger to attach to it. Entering the debugger should
be just as easy as writing a `println!` statement. This crate makes it
so.

### What about visual debuggers?

Some people love visual debuggers, and if you are one of them thats great!
`Right click > Add breakpoint` is pretty fast. If you are a visual debugger
kind of person, this crate probably isn't for you. Personally, I've never
used a visual gdb wrapper that I was able to fully trust. I've even had
enough bad experiences with TUI mode that I now prefer just the plain old
command line interface.

## Usage

First, you need the `debug-here` gdb wrapper installed. `xterm` does not
let you pass extra arguments to the program you invoke as your shell, so
`debug-here-gdb-wrapper` will arrange for `rust-gdb` to execute all the
right gdb commands.

```
cargo install debug-here-gdb-wrapper
```

Now you can add debug-here to the dependencies of a crate you want to
work on.

```
debug-here = "0.1"
```

Drop the usual `extern crate debug_here;` somewhere in your
`lib.rs` or `main.rs`, then place `#[macro_use] debug_here;`
in the module you want to debug and just write `debug_here!()`
wherever you want to get dropped into the debugger. When your
program reaches that point for the first time, it will launch
an `xterm` window with `rust-gdb` attached to your process
right after the `debug_here!()` macro. You can poke around
and start stepping through your program. If you reach another
`debug_here!()` macro invocation, you don't have to worry about
more gdb terminals spawing left and right. `debug_here!()` only
fires once per program.

## An Example: Bad Factorials

I have a very important rust program called `debug-me` which computes
the factorial of 5. Or at least its supposed to. Right now it is
telling me that the factorial of 5 is 0, which doesn't seem right.
Here is my `main.rs`.

```rust
fn factorial(n: usize) -> usize {
    let mut res = 1;
    for i in 0..n {
        res *= i;
    }
    res
}

fn main() {
    println!("The factorial of 5 is {}!", factorial(5));
}
```

You can probably see the problem, but I can't because I'm helpless without
a debugger. In order to figure out what is going on, I'm going to pull
in debug-here to help me out. First, I'll make sure that the debug-here
gdb shim is installed with `cargo install debug-here-gdb-wrapper`.
Now, I'll add debug-here to my factorial crate's `Cargo.toml`.

```
[dependencies]
debug-here = "0.1"
```

And I'll add the line

```
#[macro_use] extern crate debug_here;
```

to my source file. Now it looks like this:

```rust
#[macro_use] extern crate debug_here;

fn factorial(n: usize) -> usize {
    let mut res = 1;
    for i in 0..n {
        res *= i;
    }
    res
}

fn main() {
    println!("The factorial of 5 is {}!", factorial(5));
}
```

My loop is definitely counting up and multiplying the result variable
by bigger and bigger numbers. I feel like it should work, but I'm
going to step through the loop a few times to see what's going on.
Time to set my breakpoint with debug-here.

```rust
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
```

As easy as `println!`! Now I run my program with `cargo run`.
An xterm window pops up with a gdb shell that says:

```
debug_me::factorial (n=5) at debug-me/src/main.rs:6
6           for i in 0..n {
(gdb)
```

Looking at my source code, it seems like `res` is an interesting
variable to keep track of, so I'll ask gdb to keep me informed.

```
(gdb) disp res
1: res = 1
```

Now, I'll step through the loop a few times.

```
(gdb) n
7               res *= i;
1: res = 1
(gdb) n
6           for i in 0..n {
1: res = 0
(gdb) n
7               res *= i;
1: res = 0
```

It looks like `res` went to 0 in the first loop iteration. Looking back
at the source, I can see that this is because the counter starts at 0. I'll
quit out of the debugger and fix it.

If you want to play with this example yourself, you can just clone the
repo at `https://github.com/ethanpailes/debug-here` and run the following
commands:

```
 > cargo install debug-here-gdb-wrapper # if you haven't already done so
 > cd debug-here/debug-me
 > cargo run
```

You should see an xterm pop up with a rust-gdb shell ready to go. There
is another bug in the factorial routine. Try debugging it with
rust-gdb.

## Platforms

Right now debug-here only works on linux with rust-gdb. There might
be support for macOS and rust-lldb in the future. Windows support
is a bit less likely because I don't understand the moving pieces
as well, but that could change.

