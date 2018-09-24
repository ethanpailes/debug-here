use std::{process, env};
use std::os::unix::process::CommandExt;

fn main() {
    match inner() {
        Ok(()) => {},
        Err(errstr) => {
            eprintln!("debug-here-gdb-wrapper: {}", errstr);
            eprintln!("\n\n\n");
            eprintln!(
            r#"This terminal is just here to display the above error
               message, and won't respond to any input. When you are
               done looking, just close the window.
            "#);


            // Do nothing, but don't exit either, so the user can see
            // our error message.
            loop {
                std::thread::sleep(std::time::Duration::new(1000, 0));
            }
        }
    }
}

fn inner() -> Result<(), String> {
    let params = env::var("RUST_DEBUG_HERE_LIFELINE")
        .map_err(|_| 
            r#"Expected RUST_DEBUG_HERE_LIFELINE to be defined.
               This is a bug with debug-here. Please report it
               at `https://github.com/ethanpailes/debug-here/issues`.
               "#.to_string())?;
    env::remove_var("RUST_DEBUG_HERE_LIFELINE");

    let mut params = params.split(",");
    let fmt_version_no = params
            .next()
            .ok_or("Couldn't split out version number.".to_string())
            .and_then(|v| v.parse::<usize>().map_err(|e| e.to_string()))
            .map_err(|e| format!("Failed to parse version number: {}", e))?;

    let pid = params
            .next()
            .ok_or("Failed to get the PID of the program to be debugged."
                   .to_string())?;

    if fmt_version_no > 1 {
        return Err(
            format!("Don't know what to do with this debug-here protocol
                     version ({}). You might want to re-install
                     debug-here-gdb-wrapper.",
                fmt_version_no));
    }

    // Hopefully, this won't return, but if it does we want to display
    // the error.
    Err(process::Command::new("rust-gdb")
            .arg("-pid").arg(&pid)
            .arg("-ex").arg("set variable looping = 0")
            .arg("-ex").arg("finish")
            .exec()
            .to_string())
}
