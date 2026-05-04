use std::ffi::c_void;

use windows::core::imp::CloseHandle;
use windows::core::s;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows::Win32::UI::Shell::{SHChangeNotify, SHCNE_ASSOCCHANGED, SHCNF_IDLIST};
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowA, GetWindow, GetWindowInfo, SendMessageA, GW_CHILD, WINDOWINFO, WM_COMMAND,
    WS_VISIBLE,
};

use crate::constants::*;
use crate::explorer_modinfo::{get_explorer_handle, get_shell32_offset};

pub unsafe fn inject(rva: u32) {
    println!("Getting shell32 offset...");
    let offset = get_shell32_offset();
    println!("Offset of shell32 inside explorer.exe is {offset:#x}");
    let explorerhandle = get_explorer_handle();
    println!("Injecting ret...");
    WriteProcessMemory(
        explorerhandle,
        (offset + rva as u64) as *const c_void,
        &RET as *const u8 as *const c_void,
        RET.len(),
        None,
    )
    .unwrap();
    println!("Injected!");
    CloseHandle(explorerhandle.0);
}

pub unsafe fn refresh() {
    println!("Refreshing desktop...");
    let hWnd = GetWindow(FindWindowA(s!("Progman"), s!("Program Manager")), GW_CHILD);

    let hWnd2 = GetWindow(hWnd, GW_CHILD);
    let mut wi = WINDOWINFO::default();
    wi.cbSize = std::mem::size_of::<WINDOWINFO>() as u32;
    GetWindowInfo(hWnd2, &mut wi as *mut _).unwrap();
    let visible = wi.dwStyle & WS_VISIBLE == WS_VISIBLE;

    if visible {
        SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None);
    } else {
        SendMessageA(hWnd, WM_COMMAND, WPARAM(0x7402), LPARAM::default());
        SendMessageA(hWnd, WM_COMMAND, WPARAM(0x7402), LPARAM::default());
    }
    println!("Refreshed!")
}
