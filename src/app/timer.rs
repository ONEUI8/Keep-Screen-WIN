//! Manages the countdown timer.

use super::state::{AppState, Event};
use std::thread;
use std::time::Duration;

/// Starts a timer thread if the duration is not permanent.
/// When the timer finishes, it sends an event to deactivate the keep-awake state.
pub fn start_timer_thread(state: &mut AppState, event_tx: crossbeam_channel::Sender<Event>) {
    // Ensure any existing timer is stopped before starting a new one.
    stop_timer_thread(state);

    if let Some(seconds) = state.duration.to_seconds() {
        let (shutdown_tx, shutdown_rx) = crossbeam_channel::unbounded();
        state.timer_shutdown_tx = Some(shutdown_tx);

        println!("[DEBUG] Starting timer for {} seconds", seconds);

        thread::spawn(move || {
            // Wait for the duration to elapse or for a shutdown signal.
            match shutdown_rx.recv_timeout(Duration::from_secs(seconds)) {
                // Shutdown signal received.
                Ok(_) => {
                    println!("[DEBUG] Timer was stopped manually.");
                }
                // Timeout reached.
                Err(_) => {
                    println!("[DEBUG] Timer finished, sending ToggleActive event.");
                    let _ = event_tx.send(Event::ToggleActive);
                }
            }
        });
    }
}

/// Stops the current timer thread, if one is running.
pub fn stop_timer_thread(state: &mut AppState) {
    if let Some(shutdown_tx) = state.timer_shutdown_tx.take() {
        // Sending a message will cause the timer thread to exit.
        let _ = shutdown_tx.send(());
        println!("[DEBUG] Sent stop signal to timer thread.");
    }
}
