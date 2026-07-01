//! Shared construction of the `claude` CLI command. Both the agent runner and
//! the one-shot CV analysis spawn `claude`, so the Windows console-window
//! suppression lives here in one place.

use std::process::Command;

/// Build a `claude` command with the Windows console-window suppression applied.
///
/// On Windows `claude` resolves to a `.cmd`/`.ps1` shim, which spawns a console
/// host. Launched from the GUI .exe (no console of its own) that pops a blank
/// PowerShell/conhost window for each run. CREATE_NO_WINDOW suppresses it
/// without affecting the piped stdout/stderr we read.
pub fn command() -> Command {
    let mut cmd = Command::new("claude");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}
