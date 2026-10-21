use super::state::IS_IN_TRY_PC_MODE;
use crate::domains::media::SlintMpvPlayer;
use crate::MainWindow;
use slint::ComponentHandle;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

pub fn setup_video_timer(
    window: &MainWindow,
    player_shared: Arc<Mutex<Option<SlintMpvPlayer>>>,
) -> slint::Timer {
    let video_timer = slint::Timer::default();
    let vh = window.as_weak();
    let player_for_timer = player_shared.clone();

    video_timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(16),
        move || {
            if IS_IN_TRY_PC_MODE.load(Ordering::SeqCst) {
                return;
            }

            if let Some(ui) = vh.upgrade() {
                // Skip video rendering when Admin Panel covers the screen to preserve 100% responsiveness
                if ui.get_is_admin_open() {
                    return;
                }

                if let Ok(mut guard) = player_for_timer.try_lock() {
                    if let Some(ref mut p) = *guard {
                        if let Ok(Some(buffer)) = p.render_frame_buffer() {
                            ui.set_video_frame(slint::Image::from_rgba8(buffer));
                        }
                    }
                }
            }
        },
    );

    video_timer
}

pub fn setup_clock_timer(window: &MainWindow) -> slint::Timer {
    let clock_timer = slint::Timer::default();
    let clock_handle = window.as_weak();

    clock_timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_secs(1),
        move || {
            if let Some(ui) = clock_handle.upgrade() {
                let current = chrono::Local::now();
                ui.set_live_clock(current.format("%H:%M:%S").to_string().into());
            }
        },
    );

    clock_timer
}
