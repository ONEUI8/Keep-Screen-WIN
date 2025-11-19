//! Windows 暗色模式支持

use std::sync::{Arc, Mutex};
use winapi::shared::minwindef::{BOOL, DWORD, FALSE, TRUE};
use winapi::shared::windef::HWND;
use winapi::um::libloaderapi::{GetModuleHandleW, GetProcAddress, LoadLibraryW};

#[repr(i32)]
#[allow(dead_code)]
enum PreferredAppMode {
    Default = 0,
    AllowDark = 1,
    ForceDark = 2,
    ForceLight = 3,
    Max = 4,
}

type FnShouldAppsUseDarkMode = unsafe extern "system" fn() -> BOOL;
type FnAllowDarkModeForWindow = unsafe extern "system" fn(HWND, BOOL) -> BOOL;
type FnAllowDarkModeForApp = unsafe extern "system" fn(BOOL) -> BOOL;
type FnSetPreferredAppMode = unsafe extern "system" fn(i32) -> i32;
type FnRefreshImmersiveColorPolicyState = unsafe extern "system" fn();
type FnFlushMenuThemes = unsafe extern "system" fn();

/// 暗色模式管理器
pub struct DarkModeManager {
    supported: bool,
    enabled: Arc<Mutex<bool>>,
    should_apps_use_dark_mode: Option<FnShouldAppsUseDarkMode>,
    #[allow(dead_code)]
    allow_dark_mode_for_window: Option<FnAllowDarkModeForWindow>,
    allow_dark_mode_for_app: Option<FnAllowDarkModeForApp>,
    set_preferred_app_mode: Option<FnSetPreferredAppMode>,
    refresh_immersive_color_policy: Option<FnRefreshImmersiveColorPolicyState>,
    flush_menu_themes: Option<FnFlushMenuThemes>,
    build_number: DWORD,
}

impl DarkModeManager {
    /// 初始化暗色模式管理器
    pub fn new() -> Self {
        let mut manager = Self {
            supported: false,
            enabled: Arc::new(Mutex::new(false)),
            should_apps_use_dark_mode: None,
            allow_dark_mode_for_window: None,
            allow_dark_mode_for_app: None,
            set_preferred_app_mode: None,
            refresh_immersive_color_policy: None,
            flush_menu_themes: None,
            build_number: 0,
        };

        manager.init();
        manager
    }

    /// 初始化暗色模式功能
    fn init(&mut self) {
        unsafe {
            // 获取 Windows 版本号
            let ntdll = GetModuleHandleW(['n' as u16, 't' as u16, 'd' as u16, 'l' as u16, 'l' as u16, '.' as u16, 'd' as u16, 'l' as u16, 'l' as u16, 0].as_ptr());
            if ntdll.is_null() {
                return;
            }

            // 获取版本号
            let rtl_get_version = GetProcAddress(ntdll, b"RtlGetNtVersionNumbers\0".as_ptr() as *const i8);
            if rtl_get_version.is_null() {
                return;
            }

            let rtl_get_version: unsafe extern "system" fn(*mut DWORD, *mut DWORD, *mut DWORD) =
                std::mem::transmute(rtl_get_version);

            let mut major: DWORD = 0;
            let mut minor: DWORD = 0;
            let mut build: DWORD = 0;
            rtl_get_version(&mut major, &mut minor, &mut build);
            self.build_number = build & !0xF0000000;

            // 检查是否支持暗色模式（需要 Windows 10 1809+ 即 build 17763+）
            if major != 10 || minor != 0 || self.build_number < 17763 {
                return;
            }

            // 加载 uxtheme.dll
            let uxtheme_name = ['u' as u16, 'x' as u16, 't' as u16, 'h' as u16, 'e' as u16, 'm' as u16,
                                'e' as u16, '.' as u16, 'd' as u16, 'l' as u16, 'l' as u16, 0];
            let uxtheme = LoadLibraryW(uxtheme_name.as_ptr());

            if uxtheme.is_null() {
                return;
            }

            // 通过序号获取函数（未文档化的 API）
            // ordinal 104: RefreshImmersiveColorPolicyState
            if let Some(ptr) = Self::get_proc_by_ordinal(uxtheme, 104) {
                self.refresh_immersive_color_policy = Some(std::mem::transmute(ptr));
            }

            // ordinal 132: ShouldAppsUseDarkMode
            if let Some(ptr) = Self::get_proc_by_ordinal(uxtheme, 132) {
                self.should_apps_use_dark_mode = Some(std::mem::transmute(ptr));
            }

            // ordinal 133: AllowDarkModeForWindow
            if let Some(ptr) = Self::get_proc_by_ordinal(uxtheme, 133) {
                self.allow_dark_mode_for_window = Some(std::mem::transmute(ptr));
            }

            // ordinal 135: AllowDarkModeForApp (1809) 或 SetPreferredAppMode (1903+)
            if let Some(ptr) = Self::get_proc_by_ordinal(uxtheme, 135) {
                if self.build_number < 18362 {
                    self.allow_dark_mode_for_app = Some(std::mem::transmute(ptr));
                } else {
                    self.set_preferred_app_mode = Some(std::mem::transmute(ptr));
                }
            }

            // ordinal 136: FlushMenuThemes
            if let Some(ptr) = Self::get_proc_by_ordinal(uxtheme, 136) {
                self.flush_menu_themes = Some(std::mem::transmute(ptr));
            }

            // 检查是否所有必要的函数都已加载
            if self.should_apps_use_dark_mode.is_some()
                && self.refresh_immersive_color_policy.is_some()
                && (self.allow_dark_mode_for_app.is_some() || self.set_preferred_app_mode.is_some())
                && self.flush_menu_themes.is_some()
            {
                self.supported = true;

                // 启用应用级暗色模式
                self.allow_dark_mode_for_app_internal(true);

                // 刷新颜色策略状态
                if let Some(refresh) = self.refresh_immersive_color_policy {
                    refresh();
                }

                // 检测当前暗色模式状态
                self.update_dark_mode_state();
            }
        }
    }

    /// 通过序号获取函数指针
    unsafe fn get_proc_by_ordinal(module: winapi::shared::minwindef::HMODULE, ordinal: u16) -> Option<*const ()> {
        // SAFETY: 调用 Windows API 获取函数指针
        let ptr = unsafe { GetProcAddress(module, ordinal as usize as *const i8) };
        if ptr.is_null() {
            None
        } else {
            Some(ptr as *const ())
        }
    }

    /// 启用/禁用应用级暗色模式
    fn allow_dark_mode_for_app_internal(&self, allow: bool) {
        unsafe {
            if let Some(func) = self.allow_dark_mode_for_app {
                func(if allow { TRUE } else { FALSE });
            } else if let Some(func) = self.set_preferred_app_mode {
                let mode = if allow {
                    PreferredAppMode::AllowDark as i32
                } else {
                    PreferredAppMode::Default as i32
                };
                func(mode);
            }
        }
    }

    /// 更新暗色模式状态
    fn update_dark_mode_state(&self) {
        if !self.supported {
            return;
        }

        unsafe {
            if let Some(should_use_dark) = self.should_apps_use_dark_mode {
                let is_dark = should_use_dark() != FALSE && !self.is_high_contrast();
                *self.enabled.lock().unwrap() = is_dark;
            }
        }
    }

    /// 检查是否启用了高对比度模式
    fn is_high_contrast(&self) -> bool {
        use winapi::um::winuser::{SystemParametersInfoW, SPI_GETHIGHCONTRAST, HIGHCONTRASTW, HCF_HIGHCONTRASTON};

        unsafe {
            let mut hc: HIGHCONTRASTW = std::mem::zeroed();
            hc.cbSize = std::mem::size_of::<HIGHCONTRASTW>() as u32;

            if SystemParametersInfoW(
                SPI_GETHIGHCONTRAST,
                hc.cbSize,
                &mut hc as *mut _ as *mut _,
                0,
            ) != 0
            {
                (hc.dwFlags & HCF_HIGHCONTRASTON) != 0
            } else {
                false
            }
        }
    }

    /// 刷新菜单主题
    pub fn flush_menu_themes(&self) {
        if !self.supported {
            return;
        }

        unsafe {
            if let Some(flush) = self.flush_menu_themes {
                flush();
            }
        }
    }

    /// 处理系统设置变化消息
    /// 返回 true 表示需要更新 UI
    pub fn handle_setting_change(&self, lparam: isize) -> bool {
        if !self.supported {
            return false;
        }

        // 检查是否是颜色主题变化消息
        if lparam != 0 {
            unsafe {
                let lp = lparam as *const u16;
                let immersive_color_set = ['I' as u16, 'm' as u16, 'm' as u16, 'e' as u16, 'r' as u16,
                                           's' as u16, 'i' as u16, 'v' as u16, 'e' as u16,
                                           'C' as u16, 'o' as u16, 'l' as u16, 'o' as u16, 'r' as u16,
                                           'S' as u16, 'e' as u16, 't' as u16, 0];

                // 简单比较（在实际应用中应该使用更健壮的字符串比较）
                let mut matches = true;
                for (i, &ch) in immersive_color_set.iter().enumerate() {
                    if *lp.add(i) != ch {
                        matches = false;
                        break;
                    }
                }

                if matches {
                    // 刷新颜色策略状态
                    if let Some(refresh) = self.refresh_immersive_color_policy {
                        refresh();
                    }

                    // 更新暗色模式状态
                    self.update_dark_mode_state();

                    // 刷新菜单主题
                    self.flush_menu_themes();

                    return true;
                }
            }
        }

        false
    }
}

/// 全局暗色模式管理器实例
static mut DARK_MODE_MANAGER: Option<DarkModeManager> = None;

/// 初始化全局暗色模式管理器
pub fn init_dark_mode() {
    unsafe {
        let ptr = std::ptr::addr_of_mut!(DARK_MODE_MANAGER);
        if (*ptr).is_none() {
            *ptr = Some(DarkModeManager::new());
        }
    }
}

/// 获取全局暗色模式管理器
pub fn get_dark_mode_manager() -> Option<&'static DarkModeManager> {
    unsafe {
        let ptr = std::ptr::addr_of!(DARK_MODE_MANAGER);
        (*ptr).as_ref()
    }
}

/// 处理 WM_SETTINGCHANGE 消息
/// 返回 true 表示需要更新 UI
pub fn handle_setting_change(lparam: isize) -> bool {
    get_dark_mode_manager()
        .map(|mgr| mgr.handle_setting_change(lparam))
        .unwrap_or(false)
}
