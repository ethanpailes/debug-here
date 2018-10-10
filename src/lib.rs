#[macro_use] extern crate lazy_static;
extern crate nix;
extern crate which;

use std::sync::Mutex;
use std::{fs, process, env};
use nix::unistd;

/// The debug here macro. Just invoke this macro somewhere in your
/// source, and when your program reaches it a terminal running
/// `rust-gdb` will launch.
#[macro_export]
macro_rules! debug_here {
    () => {
        ::debug_here::debug_here_impl();
    }
}

/// The function responsible for actually launching the debugger.
///
/// If we have never launched a debugger before, we do so. Otherwise,
/// we just don't do anything on the theory that if you are debugging
/// something in a loop, you probably don't want a new `rust-gdb`
/// every time you step through your `debug_here!()`.
///
/// Before spawning the debugger we examine the execution environment
/// a bit to try to help users through any configuration errors.
pub fn debug_here_impl() {
    lazy_static! {
        static ref GUARD: Mutex<bool> = Mutex::new(false);
    }

    // Check to see if we have already popped open a debugger.
    {
        // just propogate the thread poisoning with unwrap.
        let mut entered = GUARD.lock().unwrap();

        if *entered {
            return;
        } else {
            *entered = true;
        }
    }

    let the_kids_are_ok =
        fs::read("/proc/sys/kernel/yama/ptrace_scope")
            .map(|contents|
                 std::str::from_utf8(&contents[..1]).unwrap_or("1") == "0")
            .unwrap_or(false);
    if !the_kids_are_ok {
        eprintln!(r#"debug-here:
            ptrace_scope must be set to 0 for debug-here to work.
            This will allow any process with a given uid to rummage around
            in the memory of any other process, so there are some security
            risks. To set ptrace_scope for just this session you can do:

            echo 0 | sudo tee /proc/sys/kernel/yama/ptrace_scope

            Giving up on debugging for now.
            "#);
        return;
    }

    if which::which("rust-gdb").is_err() {
        eprintln!("debug-here: can't find rust-gdb on your path. Bailing.");
        return;
    }

    if which::which("debug-here-gdb-wrapper").is_err() {
        eprintln!(r#"debug-here:
            can't find debug-here-gdb-wrapper on your path. To get it
            you can run `cargo install debug-here-gdb-wrapper`
            "#);
        return;
    }

    // Set up a magic environment variable telling debug-here-gdb-wrapper
    // where to enter the program to be debugged.
    //
    // It is nicer to use an environment variable instead of a magic file
    // or named pipe or something because that way only our kids will get
    // to see it.
    //
    // The format here is `<format version no>,<pid>`.
    env::set_var("RUST_DEBUG_HERE_LIFELINE",
        format!("1,{}", unistd::getpid()));

    // TODO(ethan): add support for alacritty and possibly other
    //              terminal emulators.
    let term = "xterm";
    if process::Command::new(term)
            .arg("debug-here-gdb-wrapper")
            .stdin(process::Stdio::null())
            .stdout(process::Stdio::null())
            .stderr(process::Stdio::null())
            .spawn()
            .is_err() {
        eprintln!("debug-here: Failed to launch rust-gdb in {}.", term);
        return;
    }

    // Now we enter an infinite loop and wait for rust-gdb to come to
    // our rescue
    let looping = true;
    while looping {}
}
