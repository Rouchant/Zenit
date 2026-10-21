use super::state::IS_IN_TRY_PC_MODE;
use crate::domains::media::SlintMpvPlayer;
use crate::MainWindow;
use slint::{ComponentHandle, Rgba8Pixel, SharedPixelBuffer};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

static LATEST_FRAME: Mutex<Option<SharedPixelBuffer<Rgba8Pixel>>> = Mutex::new(None);
static HAS_NEW_FRAME: AtomicBool = AtomicBool::new(false);

pub fn setup_video_timer(
    window: &MainWindow,
    player_shared: Arc<Mutex<Option<SlintMpvPlayer>>>,
) -> slint::Timer {
    let window_weak = window.as_weak();
    let player = player_shared.clone();

    // 1. Dedicated background worker performs 100% of MPV software decode and SIMD calculations
    std::thread::Builder::new()
        .name("zenit-video-decode".to_string())
        .spawn(move || {
            while let Some(_) = window_weak.upgrade() {
                std::thread::sleep(std::time::Duration::from_millis(16));

                if IS_IN_TRY_PC_MODE.load(Ordering::Relaxed) {
                    continue;
                }

                // If the UI hasn't consumed the last frame yet, wait to avoid CPU waste
                if HAS_NEW_FRAME.load(Ordering::Relaxed) {
                    continue;
                }

                if let Ok(mut guard) = player.try_lock() {
                    if let Some(ref mut p) = *guard {
                        if let Ok(Some(buffer)) = p.render_frame_buffer() {
                            if let Ok(mut frame_slot) = LATEST_FRAME.lock() {
                                *frame_slot = Some(buffer);
                                HAS_NEW_FRAME.store(true, Ordering::Release);
                            }
                        }
                    }
                }
            }
        })
        .expect("Failed to spawn video render thread");

    // 2. UI Timer: takes less than 0.01ms on the UI thread, simply checks the flag and sets the image!
    let video_timer = slint::Timer::default();
    let vh = window.as_weak();

    video_timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(16),
        move || {
            if HAS_NEW_FRAME.swap(false, Ordering::Acquire) {
                if let Some(ui) = vh.upgrade() {
                    if !ui.get_is_admin_open() {
                        if let Ok(mut frame_slot) = LATEST_FRAME.lock() {
                            if let Some(buffer) = frame_slot.take() {
                                ui.set_video_frame(slint::Image::from_rgba8(buffer));
                            }
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
