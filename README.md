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
