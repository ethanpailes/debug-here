# debug-here-gdb-wrapper

This is program is what debug-here tells new terminals to use as a shell.
It is very simple. All it does is examine the `RUST_DEBUG_HERE_LIFELINE`
environment variable, then becomes `rust-gdb` via exec and hooks into
the part of the source code that called the `debug_here!()` macro.
As a user, you mostly shouldn't have to worry about what this program
does. You just need to have it somewhere on your path in order to use
`debug_here!()`. The easiest way to do that is with
`cargo install debug-here-gdb-wrapper`.
