use std::env;
use std::process::Command;

/// Install a Windows scheduled task that re-runs `uwd3 inject` at every
/// interactive logon (including session reconnect).
///
/// Uses PowerShell's `Register-ScheduledTask` so no COM interop is needed.
/// Requires elevation; if not elevated the PowerShell call will fail with a
/// permission error.
pub fn install_task() {
    let exe = match env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Cannot determine current exe path: {e}");
            return;
        }
    };
    let exe_str = exe.to_string_lossy();
    // Escape single quotes for PowerShell string literals
    let exe_escaped = exe_str.replace('\'', "''");

    let script = format!(
        r#"
$ErrorActionPreference = 'Stop'
$exe = '{exe_escaped}'
$action   = New-ScheduledTaskAction -Execute $exe -Argument 'inject'
$trigger  = New-ScheduledTaskTrigger -AtLogOn
$settings = New-ScheduledTaskSettingsSet `
    -AllowStartIfOnBatteries `
    -DontStopIfGoingOnBatteries `
    -ExecutionTimeLimit (New-TimeSpan -Minutes 2) `
    -StartWhenAvailable
$principal = New-ScheduledTaskPrincipal `
    -UserId ([System.Security.Principal.WindowsIdentity]::GetCurrent().Name) `
    -LogonType Interactive `
    -RunLevel Highest
Register-ScheduledTask `
    -TaskName    'UWD3_WatermarkRemover' `
    -Action      $action `
    -Trigger     $trigger `
    -Settings    $settings `
    -Principal   $principal `
    -Description 'UWD3: removes the Windows Insider evaluation watermark at logon.' `
    -Force | Out-Null
Write-Host 'Task registered.'
"#,
        exe_escaped = exe_escaped
    );

    let status = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("Scheduled task 'UWD3_WatermarkRemover' installed.");
            println!("UWD3 will now run automatically at every logon.");
        }
        Ok(s) => {
            eprintln!(
                "PowerShell exited with code {:?}. Try running uwd3 as administrator.",
                s.code()
            );
        }
        Err(e) => {
            eprintln!("Failed to launch PowerShell: {e}");
        }
    }
}

/// Remove the scheduled task installed by `install_task`.
pub fn remove_task() {
    let script = r#"
$ErrorActionPreference = 'Stop'
Unregister-ScheduledTask -TaskName 'UWD3_WatermarkRemover' -Confirm:$false
Write-Host 'Task removed.'
"#;

    let status = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .status();

    match status {
        Ok(s) if s.success() => println!("Scheduled task removed."),
        Ok(s) => eprintln!(
            "PowerShell exited with code {:?}. The task may not have been installed.",
            s.code()
        ),
        Err(e) => eprintln!("Failed to launch PowerShell: {e}"),
    }
}
