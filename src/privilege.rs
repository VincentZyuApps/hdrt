pub fn is_elevated() -> bool {
    is_elevated_platform()
}

#[cfg(unix)]
fn is_elevated_platform() -> bool {
    unsafe { libc::geteuid() == 0 }
}

#[cfg(windows)]
fn is_elevated_platform() -> bool {
    use std::process::Command;

    let script = r#"
$current = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = [Security.Principal.WindowsPrincipal]::new($current)
$principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
"#;

    Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|stdout| stdout.trim().eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

#[cfg(not(any(unix, windows)))]
fn is_elevated_platform() -> bool {
    false
}

pub fn elevated_hint() -> &'static str {
    if cfg!(windows) {
        "Run hdrt from Administrator PowerShell for more complete hardware fields."
    } else {
        "Run sudo hdrt for more complete hardware fields."
    }
}
