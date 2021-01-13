// Copyright 2018-2019 Ethan Pailes. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/*!
This module contains the internal implementation of debug-here. Nothing
in this module is part of the public api of debug-here, even if it is
marked `pub`.

Certain functions must be marked `pub` in order for the `debug_here!()`
macro to call them, but user code should never call them directly.
*/

use std::sync::Mutex;
use std::process;

#[cfg(target_os = "linux")]
use std::{fs, env};

#[cfg(target_os = "windows")]
use winapi::um::debugapi;
#[cfg(target_os = "windows")]
use std::{path, thread};
#[cfg(target_os = "windows")]
use std::time::Duration;

#[cfg(not(any(target_os = "linux", target_os = "macos", mac_catalyst, target_os = "windows")))]
compile_error!("debug-here: this crate currently only builds on linux, macos, and windows");

fn already_entered() -> bool {
    lazy_static! {
        static ref GUARD: Mutex<bool> = Mutex::new(false);
    }

    // just propogate the thread poisoning with unwrap.
    let mut entered = GUARD.lock().unwrap();

    let ret = *entered;
    *entered = true;
    return ret;
}

/// The function responsible for actually launching the debugger.
///
/// If we have never launched a debugger before, we do so. Otherwise,
/// we just don't do anything on the theory that if you are debugging
/// something in a loop, you probably don't want a new debugger
/// every time you step through your `debug_here!()`.
///
/// Before spawning the debugger we examine the execution environment
/// a bit to try to help users through any configuration errors.
///
/// Don't use this directly.
#[cfg(not(target_os = "windows"))]
pub fn debug_here_unixy_impl(debugger: Option<&str>) {
    if already_entered() {
        return;
    }


    #[cfg(target_os = "linux")]
    let sane_env = linux_check();
    #[cfg(any(target_os = "macos", mac_catalyst))]
    let sane_env = macos_check();

    if let Err(e) = sane_env {
        eprintln!("debug-here: {}", e);
        return
    }

    #[cfg(target_os = "linux")]
    let debugger = debugger.unwrap_or("rust-gdb");
    #[cfg(any(target_os = "macos", mac_catalyst))]
    let debugger = debugger.unwrap_or("rust-lldb");

    if which::which(debugger).is_err() {
        eprintln!("debug-here: can't find {} on your path. Bailing.", debugger);
        return;
    }

    if which::which("debug-here-gdb-wrapper").is_err() {
        eprintln!(r#"debug-here:
            Can't find debug-here-gdb-wrapper on your path. To get it
            you can run `cargo install debug-here-gdb-wrapper`
            "#);
        return;
    }

    // `looping` is a magic variable name that debug-here-gdb-wrapper knows to
    // set to false in order to unstick us. We set it before launching the
    // debugger to avoid a race condition.
    let looping = true;

    #[cfg(target_os = "linux")]
    let launch_stat = linux_launch_term(debugger);
    #[cfg(any(target_os = "macos", mac_catalyst))]
    let launch_stat = macos_launch_term(debugger);

    if let Err(e) = launch_stat {
        eprintln!("debug-here: {}", e);
        return;
    }

    // Now we enter an infinite loop and wait for the debugger to come to
    // our rescue
    while looping {}
}

/// Pop open a debugger on windows.
///
/// Windows has native just-in-time debugging capabilities via the debugapi
/// winapi module, so we use that instead of manually popping open a termianl
/// and launching the debugger in that.
///
/// We perform the same re-entry check as we do for non-windows platforms.
///
/// This approach pretty directly taken from:
/// https://stackoverflow.com/questions/20337870/what-is-the-equivalent-of-system-diagnostics-debugger-launch-in-unmanaged-code
///
/// Don't use this directly.
#[cfg(target_os = "windows")]
pub fn debug_here_win_impl() {
    if already_entered() {
        return;
    }

    let jitdbg_exe = r#"c:\windows\system32\vsjitdebugger.exe"#;
    if !path::Path::new(jitdbg_exe).exists() {
        eprintln!("debug-here: could not find '{}'.", jitdbg_exe);
        return;
    }


    let pid = process::id();

    let mut cmd = process::Command::new(jitdbg_exe);
    cmd.stdin(process::Stdio::null())
       .stdout(process::Stdio::null())
       .stderr(process::Stdio::null());
    cmd.arg("-p").arg(pid.to_string());

    if let Err(e) = cmd.spawn() {
        eprintln!("debug-here: failed to launch '{}': {}", jitdbg_exe, e);
        return;
    }

    // Argument for safty: this unsafe call doesn't manipulate memory
    // in any way.
    while unsafe { debugapi::IsDebuggerPresent() } == 0 {
        thread::sleep(Duration::from_millis(100));
    }

    // Just mash F10 until you see your own code
    unsafe { debugapi::DebugBreak(); }
}


/// The args required to launch the given debugger and attach to the
/// current debugger.
#[cfg(not(target_os = "windows"))]
fn debugger_args(debugger: &str) -> Vec<String> {
    if debugger == "rust-lldb" {
        vec!["-p".to_string(),
             process::id().to_string(),
             "-o".to_string(),
             "expression looping = 0".to_string(),
             "-o".to_string(),
             "finish".to_string()]
    } else if debugger == "rust-gdb" {
        vec!["-pid".to_string(),
             process::id().to_string(),
             "-ex".to_string(),
             "set variable looping = 0".to_string(),
             "-ex".to_string(),
             "finish".to_string()]
    } else {
        panic!("unknown debugger: {}", debugger);
    }
}

/// Perform sanity checks specific to a linux environment
///
/// Returns true on success, false if we should terminate early
#[cfg(target_os = "linux")]
fn linux_check() -> Result<(), String> {
    let the_kids_are_ok =
        fs::read("/proc/sys/kernel/yama/ptrace_scope")
            .map(|contents|
                 std::str::from_utf8(&contents[..1]).unwrap_or("1") == "0")
            .unwrap_or(false);
    if !the_kids_are_ok {
        return Err(format!(r#"
            ptrace_scope must be set to 0 for debug-here to work.
            This will allow any process with a given uid to rummage around
            in the memory of any other process with the same uid, so there
            are some security risks. To set ptrace_scope for just this
            session you can do:

            echo 0 | sudo tee /proc/sys/kernel/yama/ptrace_scope

            Giving up on debugging for now.
            "#));
    }

    Ok(())
}

/// Launch a terminal in a linux environment
#[cfg(target_os = "linux")]
fn linux_launch_term(debugger: &str) -> Result<(), String> {
    // Set up a magic environment variable telling debug-here-gdb-wrapper
    // where to enter the program to be debugged.
    //
    // It is nicer to use an environment variable instead of a magic file
    // or named pipe or something because that way only our kids will get
    // to see it.
    //
    // The format here is `<format version no>,<pid>`.
    //
    // We have to do this in the linux-specific launch routine becuase
    // macos is weird about how you can lauch a new terminal window,
    // and doesn't just put new windows in a subprocess.
    if debugger == "rust-gdb" {
        // If we are being asked to launch rust-gdb, that can be handled with
        // protocol version 1, so there is no need to pester users to upgrade.
        env::set_var("RUST_DEBUG_HERE_LIFELINE",
            format!("1,{}", process::id()));
    } else {
        env::set_var("RUST_DEBUG_HERE_LIFELINE",
            format!("2,{},{}", process::id(), debugger));
    }

    let term = match which::which("alacritty").or(which::which("xterm")) {
        Ok(t) => t,
        Err(_) => {
            return Err(format!(r#"
                can't find alacritty or xterm on your path. Those are the
                only terminal emulators currently supported on linux.
                "#));
        }
    };
    let term_cmd = term.clone();

    let mut cmd = process::Command::new(term_cmd);
    cmd.stdin(process::Stdio::null())
       .stdout(process::Stdio::null())
       .stderr(process::Stdio::null());

    // Alacritty doesn't need the shim
    if term.ends_with("alacritty") {
        cmd.arg("-e");
        cmd.arg(debugger);
        cmd.args(debugger_args(debugger));
    } else {
        cmd.arg("debug-here-gdb-wrapper");
    }

    match cmd.spawn() {
        Ok(_) => Ok(()),
        Err(e) => Err(
            format!("failed to launch rust-gdb in {:?}: {}", term, e))
    }
}

/// sanity check the environment in a macos environment
#[cfg(any(target_os = "macos", mac_catalyst))]
fn macos_check() -> Result<(), String> {
    if which::which("osascript").is_err() {
        return Err(format!("debug-here: can't find osascript. Bailing."));
    }

    Ok(())
}

/// Launch a terminal in a macos environment
#[cfg(any(target_os = "macos", mac_catalyst))]
fn macos_launch_term(debugger: &str) -> Result<(), String> {
    let launch_script =
        format!(r#"tell app "Terminal"
               do script "exec {} {}"
           end tell"#, debugger,
           debugger_args(debugger).into_iter()
               .map(|a| if a.contains(" ") { format!("'{}'", a) } else { a } )
               .collect::<Vec<_>>().join(" "));

    let mut cmd = process::Command::new("osascript");
    cmd.arg("-e")
       .arg(launch_script)
       .stdin(process::Stdio::null())
       .stdout(process::Stdio::null())
       .stderr(process::Stdio::null());

    match cmd.spawn() {
        Ok(_) => Ok(()),
        Err(e) => Err(
            format!("failed to launch {} in Terminal.app: {}", debugger, e))
    }
}
