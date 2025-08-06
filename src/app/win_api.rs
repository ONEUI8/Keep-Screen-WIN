//! Encapsulates Windows API calls.

use std::ffi::CString;
use winapi::shared::winerror;
use winapi::um::{errhandlingapi, handleapi, synchapi, winbase, winnt, winuser};

/// 创建一个命名互斥锁以确保只有一个实例在运行。
pub fn create_single_instance_mutex() -> bool {
    let mutex_name = CString::new("KeepScreenAppMutex").unwrap();
    let mutex_handle = unsafe {
        let handle = synchapi::CreateMutexA(std::ptr::null_mut(), 0, mutex_name.as_ptr());
        if handle.is_null() {
            return false;
        }
        if errhandlingapi::GetLastError() == winerror::ERROR_ALREADY_EXISTS {
            handleapi::CloseHandle(handle);
            return false;
        }
        handle
    };
    let _ = Box::new(mutex_handle);
    true
}

/// 设置系统的执行状态以保持亮屏
pub fn set_keep_awake(awake: bool) {
    unsafe {
        if awake {
            println!("[DEBUG] 调用 SetThreadExecutionState: ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED");
            winbase::SetThreadExecutionState(winnt::ES_SYSTEM_REQUIRED | winnt::ES_DISPLAY_REQUIRED | winnt::ES_CONTINUOUS);
        } else {
            println!("[DEBUG] 调用 SetThreadExecutionState: ES_CONTINUOUS (恢复)");
            winbase::SetThreadExecutionState(winnt::ES_CONTINUOUS);
        }
    }
}

/// Runs the main Windows message loop.
/// This is required for the tray icon to receive events.
pub fn message_loop() {
    unsafe {
        let mut msg = std::mem::MaybeUninit::uninit();
        loop {
            let bret = winuser::GetMessageA(msg.as_mut_ptr(), 0 as _, 0, 0);
            if bret > 0 {
                winuser::TranslateMessage(msg.as_ptr());
                winuser::DispatchMessageA(msg.as_ptr());
            } else {
                break;
            }
        }
    }
}