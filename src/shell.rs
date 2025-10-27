use anyhow::Result;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub fn run(cmdline: &str, timeout: Option<Duration>, cwd: &Path) -> Result<bool> {
    let mut cmd = if cfg!(windows) {
        let mut c = Command::new("cmd");
        c.args(["/C", cmdline]);
        c
    } else {
        let mut c = Command::new("sh");
        c.args(["-lc", cmdline]);
        c
    };
    let mut child = cmd.current_dir(cwd)
        .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
        .spawn()?;

    let ok = if let Some(t) = timeout {
        let start = Instant::now();
        loop {
            if let Some(s) = child.try_wait()? { break s.success(); }
            if start.elapsed() >= t {
                let _ = child.kill();
                break false;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    } else {
        child.wait()?.success()
    };
    Ok(ok)
}
