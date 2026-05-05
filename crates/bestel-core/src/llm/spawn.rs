use tokio::process::Command;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[cfg(windows)]
pub fn cli_command(bin: &str) -> Command {
    let mut cmd = Command::new("cmd");
    cmd.arg("/C").arg(bin);
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

#[cfg(not(windows))]
pub fn cli_command(bin: &str) -> Command {
    Command::new(bin)
}
