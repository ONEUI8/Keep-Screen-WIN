//! Handles the system tray icon and menu logic.

use super::i18n::Translations;
use super::state::{AppState, DurationOption, Event, DURATION_OPTIONS};
use super::timer::{start_timer_thread, stop_timer_thread};
use super::win_api::set_keep_awake;
use std::sync::{Arc, Mutex};
use std::thread;
use trayicon::{Icon, MenuBuilder, MenuItem, TrayIconBuilder};

/// Builds the tray menu UI based on the current application state.
fn build_menu(
    is_active: bool,
    current_duration: DurationOption,
    t: &Translations,
) -> MenuBuilder<Event> {
    let mut menu = MenuBuilder::new();

    // Main checkable item to toggle the keep-awake functionality.
    menu = menu.checkable(&t.get("keep_screen_on"), is_active, Event::ToggleActive);

    // Submenu for selecting the duration.
    let mut duration_submenu = MenuBuilder::new();
    for &duration_opt in DURATION_OPTIONS {
        duration_submenu = duration_submenu.checkable(
            &duration_opt.display_text(t),
            duration_opt == current_duration,
            Event::SetDuration(duration_opt),
        );
    }
    
    menu = menu.with(MenuItem::Submenu {
        name: t.get("duration").into(),
        children: duration_submenu,
        disabled: !is_active, // Disable duration selection when not active.
        id: Some(Event::NoOp),
        icon: None,
    });

    // Exit button.
    menu = menu.separator().item(&t.get("exit_app"), Event::Exit);
    menu
}

/// Creates the tray icon and runs the event loop in a separate thread.
pub fn run_tray_event_loop(app_state: Arc<Mutex<AppState>>) {
    let (event_tx, event_rx) = crossbeam_channel::unbounded();
    let event_tx_clone = event_tx.clone();

    let icon = Icon::from_buffer(include_bytes!("../../res/tray.ico"), None, None).unwrap();

    // The tray_icon object must be mutable to update its menu.
    let mut tray_icon = {
        let state = app_state.lock().unwrap();
        let menu = build_menu(state.is_active, state.duration, &state.translations);
        TrayIconBuilder::new()
            .sender(move |e| { let _ = event_tx_clone.send(*e); })
            .icon(icon)
            .tooltip("Keep Screen")
            .on_click(Event::ShowMenu)
            .on_right_click(Event::ShowMenu)
            .menu(menu)
            .build()
            .unwrap()
    };
    
    let event_handler_state = Arc::clone(&app_state);
    thread::spawn(move || {
        // Initial call to `set_keep_awake` to activate on startup.
        // This must be done in the same thread as subsequent calls.
        println!("[DEBUG] Event thread started, activating keep-awake by default.");
        set_keep_awake(true);

        event_rx.iter().for_each(|event| {
            // `ShowMenu` is handled directly by the tray icon library, so we filter it out.
            if event != Event::ShowMenu {
                let mut state = event_handler_state.lock().unwrap();
                println!("[DEBUG] Received event: {:?}", event);

                match event {
                    Event::ToggleActive => {
                        state.is_active = !state.is_active;
                        println!("[DEBUG] Toggled active state to: {}", state.is_active);
                        set_keep_awake(state.is_active);

                        if state.is_active {
                            start_timer_thread(&mut state, event_tx.clone());
                        } else {
                            stop_timer_thread(&mut state);
                        }
                    }
                    Event::SetDuration(new_duration) => {
                        state.duration = new_duration;
                        println!("[DEBUG] Set duration to: {:?}", new_duration);
                        // Restart the timer with the new duration if active.
                        if state.is_active {
                            stop_timer_thread(&mut state);
                            start_timer_thread(&mut state, event_tx.clone());
                        }
                    }
                    Event::Exit => {
                        println!("[DEBUG] Exiting application...");
                        set_keep_awake(false); // Clean up by restoring default system behavior.
                        std::process::exit(0);
                    }
                    Event::NoOp => {} // Do nothing for submenu parent.
                    Event::ShowMenu => unreachable!(), // Already filtered out.
                }
                
                // Rebuild the menu to reflect the new state and update the tray icon.
                let new_menu = build_menu(state.is_active, state.duration, &state.translations);
                tray_icon.set_menu(&new_menu).unwrap();
            } else {
                println!("[DEBUG] Received event: ShowMenu");
                // Manually show the menu on left-click (or right-click).
                let _ = tray_icon.show_menu();
            }
        })
    });
}
