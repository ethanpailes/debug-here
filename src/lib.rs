// Copyright 2018-2019 Ethan Pailes. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/*!
This crate provides a macro for launching the debugger most appropriate for
your platform in a new terminal. The first time a program executes the
`debug_here!()` macro, it launches a new terminal and automatically
attaches either rust-gdb or rust-lldb to your program at the location
of the macro.

[The README](https://github.com/ethanpailes/debug-here/blob/master/README.md)
contains more details and examples.

The [debug-me](https://github.com/ethanpailes/debug-here/tree/master/debug-me)
program provides a concrete usage example.
*/

#[macro_use] extern crate lazy_static;

#[cfg(not(target_os = "windows"))]
extern crate which;
#[cfg(target_os = "windows")]
extern crate winapi;

pub mod internal;

/// The debug here macro. Just invoke this macro somewhere in your
/// source, and when your program reaches it a terminal running
/// `rust-gdb` or `rust-lldb` will launch.
///
/// If you want to force a specific debugger backend, you can write
/// `debug_here!(gdb)` or `debug_here!(lldb)`.
#[cfg(not(target_os = "windows"))]
#[macro_export]
macro_rules! debug_here {
    () => {
        ::debug_here::internal::debug_here_unixy_impl(None);
    };
    ( gdb ) => {
        ::debug_here::internal::debug_here_unixy_impl(Some("rust-gdb"));
    };
    ( lldb ) => {
        ::debug_here::internal::debug_here_unixy_impl(Some("rust-lldb"));
    };
}

#[cfg(target_os = "windows")]
#[macro_export]
macro_rules! debug_here {
    () => {
        ::debug_here::internal::debug_here_win_impl();
    };
    ( gdb ) => {
        compile_error!("debug-here: gdb is not supported on windows");
    };
    ( lldb ) => {
        compile_error!("debug-here: lldb is not supported on windows");
    };
}
