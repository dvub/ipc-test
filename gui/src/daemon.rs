use daemonize::Daemonize;
use interprocess::local_socket::Name;
use std::fs::read_to_string;
use std::fs::File;
use std::io;
use tempdir::TempDir;

use crate::run;

pub fn start_daemon(name: Name) -> io::Result<usize> {
    // will this be problematic?
    let directory = TempDir::new("IPC_TEST")?;
    let pid_path = directory.path().join("test.pid");

    let stdout = File::create("/tmp/daemon.out")?;
    let stderr = File::create("/tmp/daemon.err")?;

    let daemonize = Daemonize::new()
        .pid_file(&pid_path)
        .stdout(stdout) // Redirect stdout to `/tmp/daemon.out`.
        .stderr(stderr); // Redirect stderr to `/tmp/daemon.err`.

    match daemonize.execute() {
        daemonize::Outcome::Parent(_) => {}
        daemonize::Outcome::Child(_) => {
            // i'm pretty sure unwrapping this is going to be really bad
            let _ = run(name);
        }
    };

    // immediately after starting the daemon,
    // retrieve the PID from the file (and parse it to a usize)

    // TODO:
    // take care of this expect()
    let pid = read_to_string(pid_path)?
        .trim()
        .parse::<usize>()
        .expect("error parsing PID!");

    // todo?
    // drop(tmp_file);
    // tmp_dir.close()?;

    Ok(pid)
}
