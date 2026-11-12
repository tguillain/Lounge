use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PowerAction {
    Sleep,
    Restart,
    ShutDown,
}

pub fn shutdown_arguments(action: PowerAction) -> Option<[&'static str; 3]> {
    match action {
        PowerAction::Sleep => None,
        PowerAction::Restart => Some(["/r", "/t", "0"]),
        PowerAction::ShutDown => Some(["/s", "/t", "0"]),
    }
}

#[cfg(windows)]
pub fn run(action: PowerAction) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    use windows_sys::Win32::System::Power::SetSuspendState;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let Some(arguments) = shutdown_arguments(action) else {
        let suspended = unsafe { SetSuspendState(0, 0, 0) };
        return if suspended != 0 { Ok(()) } else { Err("Windows refused to go to sleep.".into()) };
    };
    let status = std::process::Command::new("shutdown")
        .args(arguments)
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map_err(|error| error.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("Windows refused to {} (code {}).", verb(action), status.code().unwrap_or(-1)))
    }
}

#[cfg(not(windows))]
pub fn run(_action: PowerAction) -> Result<(), String> {
    Err("Power actions only work on Windows.".into())
}

#[cfg(windows)]
fn verb(action: PowerAction) -> &'static str {
    match action {
        PowerAction::Sleep => "go to sleep",
        PowerAction::Restart => "restart",
        PowerAction::ShutDown => "shut down",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restarts_and_shuts_down_right_away() {
        assert_eq!(shutdown_arguments(PowerAction::Restart), Some(["/r", "/t", "0"]));
        assert_eq!(shutdown_arguments(PowerAction::ShutDown), Some(["/s", "/t", "0"]));
        assert_eq!(shutdown_arguments(PowerAction::Sleep), None);
    }

    #[test]
    fn reads_actions_sent_by_the_interface() {
        let action: PowerAction = serde_json::from_str("\"shut-down\"").unwrap();
        assert_eq!(action, PowerAction::ShutDown);
    }
}
