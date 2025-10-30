use anyhow::Result;
use std::{
    path::Path,
    process::{Command, Stdio},
    time::Duration,
};

pub fn run(cmdline: &str, timeout: Option<Duration>, cwd: &Path) -> Result<bool> {
    let mut command = if cfg!(windows) {
        let mut command_builder = Command::new("cmd");
        command_builder.args(["/C", cmdline]);
        command_builder
    } else {
        let mut command_builder = Command::new("sh");
        command_builder.arg("-c");
        command_builder.arg(cmdline);
        command_builder
    };
    command.stdout(Stdio::null()).stderr(Stdio::null());
    command.current_dir(cwd);

    if let Some(to) = timeout {
        let mut child = command.spawn()?;
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status.success()),
            Ok(None) => {
                let start = std::time::Instant::now();
                while start.elapsed() < to {
                    std::thread::sleep(Duration::from_millis(100));
                    if let Some(status) = child.try_wait()? {
                        return Ok(status.success());
                    }
                }
                child.kill()?;
                return Ok(false);
            }
            Err(e) => return Err(e.into()),
        }
    }
    Ok(command.status()?.success())
}