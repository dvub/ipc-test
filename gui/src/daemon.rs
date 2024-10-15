use daemonize::Daemonize;
use interprocess::local_socket::Name;
use std::fs::read_to_string;
use std::fs::File;
use tempdir::TempDir;

use crate::run;

pub fn start_daemon(name: Name) -> usize {
    // will this be problematic?
    let directory = TempDir::new("IPC_TEST").unwrap();
    let pid_path = directory.path().join("test.pid");

    let stdout = File::create("/tmp/daemon.out").unwrap();
    let stderr = File::create("/tmp/daemon.err").unwrap();

    let daemonize = Daemonize::new()
        .pid_file(&pid_path)
        .stdout(stdout) // Redirect stdout to `/tmp/daemon.out`.
        .stderr(stderr); // Redirect stderr to `/tmp/daemon.err`.

    match daemonize.execute() {
        daemonize::Outcome::Parent(_p) => {}
        daemonize::Outcome::Child(_) => {
            let _ = run(name);
        }
    };

    // immediately after starting the daemon,
    // retrieve the PID from the file (and parse it to a usize)
    let pid = read_to_string(pid_path)
        .unwrap()
        .trim()
        .parse::<usize>()
        .unwrap();

    // todo?
    // drop(tmp_file);
    // tmp_dir.close()?;

    pid
}
