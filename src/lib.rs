#[macro_use] extern crate lazy_static;
extern crate nix;
extern crate which;

use std::sync::Mutex;
use std::{process};
#[cfg(target_os = "linux")]
use std::{fs, env};
use nix::unistd;

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
compile_error!("debug-here: this crate currently only builds on linux and macos");

/// The debug here macro. Just invoke this macro somewhere in your
/// source, and when your program reaches it a terminal running
/// `rust-gdb` will launch.
#[macro_export]
macro_rules! debug_here {
    () => {
        ::debug_here::debug_here_impl(None);
    };
    ( gdb ) => {
        ::debug_here::debug_here_impl(Some("rust-gdb"));
    };
    ( lldb ) => {
        ::debug_here::debug_here_impl(Some("rust-lldb"));
    };
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
pub fn debug_here_impl(debugger: Option<&str>) {
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

    #[cfg(target_os = "linux")]
    let sane_env = linux_check();
    #[cfg(target_os = "macos")]
    let sane_env = macos_check();

    if let Err(e) = sane_env {
        eprintln!("debug-here: {}", e);
        return
    }

    #[cfg(target_os = "linux")]
    let debugger = debugger.unwrap_or("rust-gdb");
    #[cfg(target_os = "macos")]
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
    #[cfg(target_os = "macos")]
    let launch_stat = macos_launch_term(debugger);

    if let Err(e) = launch_stat {
        eprintln!("debug-here: {}", e);
        return
    }

    // Now we enter an infinite loop and wait for rust-gdb to come to
    // our rescue
    while looping {}
}

fn debugger_args(debugger: &str) -> Vec<String> {
    if debugger == "rust-lldb" {
        vec!["-p".to_string(),
             unistd::getpid().to_string(),
             "-o".to_string(),
             "expression looping = 0".to_string(),
             "-o".to_string(),
             "finish".to_string()]
    } else if debugger == "rust-gdb" {
        vec!["-pid".to_string(),
             unistd::getpid().to_string(),
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
            format!("1,{}", unistd::getpid()));
    } else {
        env::set_var("RUST_DEBUG_HERE_LIFELINE",
            format!("2,{},{}", unistd::getpid(), debugger));
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
#[cfg(target_os = "macos")]
fn macos_check() -> Result<(), String> {
    if which::which("osascript").is_err() {
        return Err(format!("debug-here: can't find osascript. Bailing."));
    }

    Ok(())
}

/// Launch a terminal in a macos environment
#[cfg(target_os = "macos")]
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
