use std::process::Command;

use baseview::WindowHandle;

unsafe impl Send for Instance {}
pub struct Instance {
    // TODO: make these accessible through new()
    pub window: WindowHandle,
    pub daemon_pid: usize,
}
impl Drop for Instance {
    fn drop(&mut self) {
        self.window.close();
        self.kill_daemon();
    }
}

impl Instance {
    fn kill_daemon(&mut self) {
        let kill_output = Command::new("kill")
            // TODO:
            // could be -15 etc
            .arg("-9")
            .arg(self.daemon_pid.to_string())
            .output()
            .unwrap();

        println!("{}", String::from_utf8(kill_output.stderr).unwrap());
    }
}
