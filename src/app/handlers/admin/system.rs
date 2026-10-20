use crate::app::state::{get_last_dismiss_tracker, IS_SCREENSAVER_ACTIVE};
use crate::domains::media::SlintMpvPlayer;
use crate::MainWindow;
use slint::ComponentHandle;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub fn setup_system_handlers(
    main_window: &MainWindow,
    player_shared: Arc<Mutex<Option<SlintMpvPlayer>>>,
) {
    // 1. Screensaver Dismiss Callback
    let dismiss_handle = main_window.as_weak();
    let player_dismiss = player_shared.clone();
    let last_dismiss_tracker = get_last_dismiss_tracker();
    main_window.on_dismiss_screensaver(move || {
        if let Ok(mut t) = last_dismiss_tracker.lock() {
            *t = Instant::now();
        }
        if let Some(ui) = dismiss_handle.upgrade() {
            ui.set_is_screensaver_active(false);
            IS_SCREENSAVER_ACTIVE.store(false, Ordering::SeqCst);
        }
        let landing_video = crate::domains::media::resolve_showroom_video_path();
        if let Ok(mut guard) = player_dismiss.lock() {
            if let Some(ref mut p) = *guard {
                p.change_video_looping(&landing_video.to_string_lossy(), true);
            }
        }
    });

    // 2. Emergency Exit & Restart
    main_window.on_emergency_exit_clicked(|| {
        std::process::exit(0);
    });

    main_window.on_emergency_restart_clicked(|| {
        if let Ok(current_exe) = std::env::current_exe() {
            let _ = std::process::Command::new(current_exe).spawn();
        }
        std::process::exit(0);
    });
}
