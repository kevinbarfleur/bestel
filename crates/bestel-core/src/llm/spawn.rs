use tokio::process::Command;

#[cfg(windows)]
pub fn cli_command(bin: &str) -> Command {
    let mut cmd = Command::new("cmd");
    cmd.arg("/C").arg(bin);
    cmd
}

#[cfg(not(windows))]
pub fn cli_command(bin: &str) -> Command {
    Command::new(bin)
}
