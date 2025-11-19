//! 封装 Windows API 调用

use super::darkmode;
use super::state::Event;
use crossbeam_channel::Sender;
use std::ffi::CString;
use std::sync::Mutex;
use winapi::shared::winerror;
use winapi::shared::windef::HWND;
use winapi::um::{errhandlingapi, handleapi, synchapi, winbase, winnt, winuser};

/// 主题变化通知回调
static mut THEME_CHANGE_CALLBACK: Option<Mutex<Sender<Event>>> = None;

/// 设置主题变化回调
pub fn set_theme_change_callback(sender: Sender<Event>) {
    unsafe {
        let ptr = std::ptr::addr_of_mut!(THEME_CHANGE_CALLBACK);
        *ptr = Some(Mutex::new(sender));
    }
}

/// 创建一个命名互斥锁以确保只有一个实例在运行。
pub fn create_single_instance_mutex() -> bool {
    let mutex_name = match CString::new("KeepScreenAppMutex") {
        Ok(name) => name,
        Err(e) => {
            eprintln!("创建互斥锁名称失败: {}", e);
            return false;
        }
    };
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
    // 保持 mutex 存活，防止内存泄漏
    let _ = Box::leak(Box::new(mutex_handle));
    true
}

/// 设置系统的执行状态以保持亮屏
pub fn set_keep_awake(awake: bool) {
    unsafe {
        if awake {
            winbase::SetThreadExecutionState(winnt::ES_SYSTEM_REQUIRED | winnt::ES_DISPLAY_REQUIRED | winnt::ES_CONTINUOUS);
        } else {
            winbase::SetThreadExecutionState(winnt::ES_CONTINUOUS);
        }
    }
}

/// Windows 消息循环
pub fn message_loop() {
    unsafe {
        // 创建隐藏窗口用于监听系统消息
        create_message_window();

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

/// 创建隐藏窗口用于接收系统消息
unsafe fn create_message_window() -> HWND {
    let class_name = ['K' as u16, 'e' as u16, 'e' as u16, 'p' as u16, 'S' as u16, 'c' as u16,
                      'r' as u16, 'e' as u16, 'e' as u16, 'n' as u16, 'M' as u16, 's' as u16,
                      'g' as u16, 0];

    let wc = winuser::WNDCLASSW {
        style: 0,
        lpfnWndProc: Some(message_window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: std::ptr::null_mut(),
        hIcon: std::ptr::null_mut(),
        hCursor: std::ptr::null_mut(),
        hbrBackground: std::ptr::null_mut(),
        lpszMenuName: std::ptr::null(),
        lpszClassName: class_name.as_ptr(),
    };

    // SAFETY: 注册窗口类 - 必须在 unsafe 块中调用
    unsafe {
        // 注册窗口类
        winuser::RegisterClassW(&wc);

        // 创建消息窗口
        winuser::CreateWindowExW(
            0,
            class_name.as_ptr(),
            std::ptr::null(),
            0,
            0,
            0,
            0,
            0,
            winuser::HWND_MESSAGE, // 创建消息窗口
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    }
}

/// 消息窗口过程
unsafe extern "system" fn message_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    match msg {
        winuser::WM_SETTINGCHANGE => {
            // 处理系统设置变化消息
            if darkmode::handle_setting_change(lparam) {
                // 通知应用主题已变化
                // SAFETY: 访问静态变量
                unsafe {
                    let ptr = std::ptr::addr_of!(THEME_CHANGE_CALLBACK);
                    if let Some(ref callback) = *ptr
                        && let Ok(sender) = callback.lock() {
                            let _ = sender.send(Event::ThemeChanged);
                        }
                }
            }
            0
        }
        _ => unsafe { winuser::DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}
