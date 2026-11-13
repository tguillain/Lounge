use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct Process {
    pub id: u32,
    pub exe: PathBuf,
}

/// Lists running processes with their executable, keeping the paths it has
/// already read so a poll only opens the processes that started since.
#[derive(Default)]
pub struct Scanner {
    known: HashMap<u32, (String, Option<PathBuf>)>,
}

impl Scanner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn running(&mut self) -> Vec<Process> {
        let entries = platform::entries();
        let mut known = HashMap::with_capacity(entries.len());
        for (id, name) in entries {
            let exe = match self.known.remove(&id) {
                Some((previous, exe)) if previous == name => exe,
                _ => platform::exe_path(id),
            };
            known.insert(id, (name, exe));
        }
        self.known = known;
        self.known
            .iter()
            .filter_map(|(id, (_, exe))| exe.clone().map(|exe| Process { id: *id, exe }))
            .collect()
    }
}

pub fn running() -> Vec<Process> {
    Scanner::new().running()
}

pub fn terminate(id: u32) -> io::Result<()> {
    platform::terminate(id)
}

#[cfg(windows)]
mod platform {
    use std::ffi::OsString;
    use std::io;
    use std::mem::size_of;
    use std::os::windows::ffi::OsStringExt;
    use std::path::PathBuf;

    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
    };
    use windows_sys::Win32::System::Threading::{
        OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_TERMINATE, QueryFullProcessImageNameW,
        TerminateProcess,
    };

    struct Handle(HANDLE);

    impl Drop for Handle {
        fn drop(&mut self) {
            unsafe { CloseHandle(self.0) };
        }
    }

    fn open(id: u32, access: u32) -> io::Result<Handle> {
        let handle = unsafe { OpenProcess(access, 0, id) };
        if handle.is_null() { Err(io::Error::last_os_error()) } else { Ok(Handle(handle)) }
    }

    pub fn entries() -> Vec<(u32, String)> {
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
        if snapshot == INVALID_HANDLE_VALUE {
            return Vec::new();
        }
        let snapshot = Handle(snapshot);
        let mut entry: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
        entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;
        let mut entries = Vec::new();
        let mut more = unsafe { Process32FirstW(snapshot.0, &mut entry) } != 0;
        while more {
            if entry.th32ProcessID != 0 {
                let length = entry.szExeFile.iter().position(|&unit| unit == 0).unwrap_or(entry.szExeFile.len());
                entries.push((entry.th32ProcessID, String::from_utf16_lossy(&entry.szExeFile[..length])));
            }
            more = unsafe { Process32NextW(snapshot.0, &mut entry) } != 0;
        }
        entries
    }

    /// Limited access is enough to read the path of a process running as
    /// administrator, which full access (and so sysinfo) is refused.
    pub fn exe_path(id: u32) -> Option<PathBuf> {
        let process = open(id, PROCESS_QUERY_LIMITED_INFORMATION).ok()?;
        let mut buffer = vec![0u16; 1024];
        let mut length = buffer.len() as u32;
        let read = unsafe { QueryFullProcessImageNameW(process.0, PROCESS_NAME_WIN32, buffer.as_mut_ptr(), &mut length) };
        (read != 0 && length > 0).then(|| PathBuf::from(OsString::from_wide(&buffer[..length as usize])))
    }

    pub fn terminate(id: u32) -> io::Result<()> {
        let process = open(id, PROCESS_TERMINATE)?;
        if unsafe { TerminateProcess(process.0, 1) } == 0 { Err(io::Error::last_os_error()) } else { Ok(()) }
    }
}

#[cfg(not(windows))]
mod platform {
    use std::io;
    use std::path::PathBuf;

    pub fn entries() -> Vec<(u32, String)> {
        Vec::new()
    }

    pub fn exe_path(_id: u32) -> Option<PathBuf> {
        None
    }

    pub fn terminate(_id: u32) -> io::Result<()> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "Not supported on this system"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_its_own_executable() {
        let me = std::env::current_exe().unwrap();
        let mut scanner = Scanner::new();
        let mine = |processes: Vec<Process>| processes.into_iter().find(|process| process.id == std::process::id());
        assert_eq!(mine(scanner.running()).map(|process| process.exe), Some(me.clone()));
        assert_eq!(mine(scanner.running()).map(|process| process.exe), Some(me));
    }

    #[test]
    #[ignore = "reads this PC's processes"]
    fn reads_this_pc() {
        for process in Scanner::new().running() {
            println!("{} {}", process.id, process.exe.display());
        }
    }
}
