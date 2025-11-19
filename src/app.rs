//! 应用主模块，负责初始化和运行

use std::sync::{Arc, Mutex};

// 声明子模块
mod darkmode;
mod i18n;
mod state;
mod timer;
mod tray;
mod win_api;

use state::AppState;

/// 运行应用程序
pub fn run() {
    // 1. 确保只有一个实例在运行
    if !win_api::create_single_instance_mutex() {
        return;
    }

    // 2. 初始化暗色模式支持
    darkmode::init_dark_mode();

    // 3. 初始化应用状态 (这会加载语言文件)
    let app_state = Arc::new(Mutex::new(AppState::new()));

    // 4. 创建托盘图标并启动事件循环
    tray::run_tray_event_loop(app_state);

    // 5. 运行 Windows 消息循环
    win_api::message_loop();
}
