use anyhow::Result;
use std::{
    path::Path,
    process::{Child, Command, ExitStatus, Stdio},
    time::{Duration, Instant},
};

pub fn run(cmdline: &str, timeout: Option<Duration>, cwd: &Path) -> Result<bool> {
    let mut command = build_command(cmdline);
    command.stdout(Stdio::null()).stderr(Stdio::null());
    command.current_dir(cwd);

    match timeout {
        Some(to) => run_with_timeout(&mut command, to),
        None => Ok(command.status()?.success()),
    }
}

fn build_command(cmdline: &str) -> Command {
    if cfg!(windows) {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", cmdline]);
        cmd
    } else {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", cmdline]);
        cmd
    }
}

fn run_with_timeout(command: &mut Command, timeout: Duration) -> Result<bool> {
    let mut child = command.spawn()?;

    if let Some(status) = child.try_wait()? {
        return Ok(status.success());
    }

    match wait_or_timeout(&mut child, timeout)? {
        Some(status) => Ok(status.success()),
        None => {
            child.kill()?;
            Ok(false)
        }
    }
}

fn wait_or_timeout(child: &mut Child, timeout: Duration) -> Result<Option<ExitStatus>> {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if let Some(status) = child.try_wait()? {
            return Ok(Some(status));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Ok(None)
}