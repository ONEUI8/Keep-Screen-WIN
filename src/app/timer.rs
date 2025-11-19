//! 计时器管理模块

use super::state::{AppState, Event};
use std::thread;
use std::time::Duration;

/// 启动计时器线程（如果时长不是永久）
pub fn start_timer_thread(state: &mut AppState, event_tx: crossbeam_channel::Sender<Event>) {
    stop_timer_thread(state);

    if let Some(seconds) = state.duration.to_seconds() {
        let (shutdown_tx, shutdown_rx) = crossbeam_channel::unbounded();
        state.timer_shutdown_tx = Some(shutdown_tx);

        thread::spawn(move || {
            match shutdown_rx.recv_timeout(Duration::from_secs(seconds)) {
                Ok(_) => {
                    // 计时器被手动停止
                }
                Err(_) => {
                    // 计时结束，发送关闭亮屏事件
                    let _ = event_tx.send(Event::ToggleActive);
                }
            }
        });
    }
}

/// 停止当前的计时器线程
pub fn stop_timer_thread(state: &mut AppState) {
    if let Some(shutdown_tx) = state.timer_shutdown_tx.take() {
        let _ = shutdown_tx.send(());
    }
}
