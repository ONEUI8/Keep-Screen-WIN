//! Main application module, responsible for initialization and execution.

use std::sync::{Arc, Mutex};

// Declare sub-modules.
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

    // 2. 初始化应用状态 (这会加载语言文件)
    let app_state = Arc::new(Mutex::new(AppState::new()));
    
    // 3. 创建托盘图标并启动事件循环
    tray::run_tray_event_loop(app_state);

    // 4. Run the main Windows message loop.
    win_api::message_loop();
}