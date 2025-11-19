//! 系统托盘图标和菜单逻辑

use super::i18n::Translations;
use super::state::{AppState, DurationOption, Event, DURATION_OPTIONS};
use super::timer::{start_timer_thread, stop_timer_thread};
use super::win_api::{set_keep_awake, set_theme_change_callback};
use std::sync::{Arc, Mutex};
use std::thread;
use trayicon::{Icon, MenuBuilder, MenuItem, TrayIconBuilder};

/// 构建菜单UI
fn build_menu(
    is_active: bool,
    current_duration: DurationOption,
    t: &Translations,
) -> MenuBuilder<Event> {
    let mut menu = MenuBuilder::new();
    menu = menu.checkable(&t.get("keep_screen_on"), is_active, Event::ToggleActive);

    let mut duration_submenu = MenuBuilder::new();
    for &duration_opt in DURATION_OPTIONS {
        duration_submenu = duration_submenu.checkable(
            &duration_opt.display_text(t),
            duration_opt == current_duration,
            Event::SetDuration(duration_opt),
        );
    }
    
    menu = menu.with(MenuItem::Submenu {
        name: t.get("duration"),
        children: duration_submenu,
        disabled: !is_active,
        id: Some(Event::NoOp),
        icon: None,
    });

    menu = menu.separator().item(&t.get("exit_app"), Event::Exit);
    menu
}

/// 创建托盘图标并运行事件循环
pub fn run_tray_event_loop(app_state: Arc<Mutex<AppState>>) {
    let (event_tx, event_rx) = crossbeam_channel::unbounded();
    let event_tx_clone = event_tx.clone();

    // 设置主题变化回调，当系统主题变化时发送 ThemeChanged 事件
    set_theme_change_callback(event_tx.clone());

    let icon = match Icon::from_buffer(include_bytes!("../../res/tray.ico"), None, None) {
        Ok(icon) => icon,
        Err(e) => {
            eprintln!("加载托盘图标失败: {}", e);
            return;
        }
    };

    let mut tray_icon = {
        let state = match app_state.lock() {
            Ok(guard) => guard,
            Err(e) => {
                eprintln!("获取应用状态锁失败: {}", e);
                return;
            }
        };
        let menu = build_menu(state.is_active, state.duration, &state.translations);
        match TrayIconBuilder::new()
            .sender(move |e| { let _ = event_tx_clone.send(*e); })
            .icon(icon)
            .tooltip("Keep Screen")
            .on_click(Event::ShowMenu)
            .on_right_click(Event::ShowMenu)
            .menu(menu)
            .build()
        {
            Ok(icon) => icon,
            Err(e) => {
                eprintln!("构建托盘图标失败: {}", e);
                return;
            }
        }
    };

    let event_handler_state = Arc::clone(&app_state);
    thread::spawn(move || {
        // 初始调用，确保和后续调用在同一线程
        set_keep_awake(true);

        event_rx.iter().for_each(|event| {
            if event != Event::ShowMenu {
                let mut state = match event_handler_state.lock() {
                    Ok(guard) => guard,
                    Err(e) => {
                        eprintln!("获取应用状态锁失败: {}", e);
                        return;
                    }
                };

                let mut needs_menu_update = true;
                
                match event {
                    Event::ToggleActive => {
                        state.is_active = !state.is_active;
                        set_keep_awake(state.is_active);

                        if state.is_active {
                            start_timer_thread(&mut state, event_tx.clone());
                        } else {
                            stop_timer_thread(&mut state);
                        }
                    }
                    Event::SetDuration(new_duration) => {
                        state.duration = new_duration;
                        if state.is_active {
                            stop_timer_thread(&mut state);
                            start_timer_thread(&mut state, event_tx.clone());
                        }
                    }
                    Event::ThemeChanged => {
                        // ThemeChanged 已经需要更新菜单
                    }
                    Event::Exit => {
                        set_keep_awake(false);
                        std::process::exit(0);
                    }
                    Event::NoOp => {
                        needs_menu_update = false;
                    }
                    Event::ShowMenu => unreachable!(),
                }

                // 只在需要时更新菜单
                if needs_menu_update {
                    let new_menu = build_menu(state.is_active, state.duration, &state.translations);
                    if let Err(e) = tray_icon.set_menu(&new_menu) {
                        eprintln!("更新托盘菜单失败: {}", e);
                    }
                }
            } else {
                let _ = tray_icon.show_menu();
            }
        })
    });
}
