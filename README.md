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
point. It's not that much work, but I find myself doing it a fair
amount, so it can get annoying.

One alternative is to arrange for your program to enter some sort
of sleep or looping state, rummage around for the PID on the command
line, then attach with `gdb -pid`. Usually this ends up being a bit
more work (again, not much work, but enough to be annoying), so people
only do it if it is not convenient to launch the program in question
from the terminal.

### Making getting to the debugger easier in rust

This crate automates the process of convincing your program to
wait around for a debugger to attach to it. Entering the debugger should
be just as easy as writing a `println!` statement. This crate makes it
so.

## Usage

### Linux Specific Setup

If you are using linux, you need to install the debugger wrapper
(which is called `debug-here-gdb-wrapper` for historical reasons).
Not all terminal emulators allow you to pass extra arguments to the
program you invoke as your shell, so `debug-here-gdb-wrapper` will
arrange for your debugger backend to execute all the right commands.

```
cargo install debug-here-gdb-wrapper
```

# General Setup

Now you can add debug-here to the dependencies of a crate you want to
work on.

```
debug-here = "0.1"
```

Drop the usual `#[macro_use] extern crate debug_here;` somewhere in your
`lib.rs` or `main.rs`, then just write `debug_here!()`
wherever you want to get dropped into the debugger. When your
program reaches that point for the first time, it will launch
a terminal window with `rust-gdb` or `rust-lldb` attached to your process
right after the `debug_here!()` macro. The exact debugger used by default
depends on your platform, but you can write `debug_here!(gdb)` or
`debug_here!(lldb)` to force a particular backend. You can poke around
and start stepping through your program. If you reach another
`debug_here!()` macro invocation, you don't have to worry about
more debugger terminals spawning left and right. `debug_here!()` only
fires once per program.

## Supported Terminal Emulators

Currently debug-here supports `alacritty` and `xterm` on linux, and
`Terminal.app` on macos. If you have alacritty on your path, it will use that,
on the theory that you would rather use a less standard terminal emulator
if you went to all the trouble of installing it. If you don't have
`alacritty` on your path, it will fall back on `xterm`.

## Platforms

Right now `debug-here` only works on linux and macos. There may be support
for Windows in the future. `debug-here` defaults to using `rust-gdb` on linux
and `rust-lldb` on macos. The primary reason for defaulting to `rust-lldb`
on macos is to avoid the pain of getting a properly code-signed gdb.
debug-here aims to make getting into the debugger as painless as possible.

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
debugger shim is installed with `cargo install debug-here-gdb-wrapper`.
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
An terminal window pops up with a gdb shell that says:

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
 > cargo install debug-here-gdb-wrapper # if you are on linux
 > cd debug-here/debug-me
 > cargo run
```

You should see a terminal pop up with a rust-gdb shell ready to go. There
is another bug in the factorial routine. Try debugging it with
rust-gdb.
