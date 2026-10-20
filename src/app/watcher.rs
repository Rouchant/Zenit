use super::state::{get_last_dismiss_tracker, IS_IN_TRY_PC_MODE, IS_SCREENSAVER_ACTIVE};
use super::window::restore_main_window;
use crate::domains::kiosk::get_system_idle_seconds;
use crate::domains::media::SlintMpvPlayer;
use crate::domains::showcase::load_config;
use crate::{FloatingDetailsButton, MainWindow};
use slint::ComponentHandle;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub fn spawn_inactivity_watcher(
    main_window: &MainWindow,
    floating_btn: &FloatingDetailsButton,
    player_shared: Arc<Mutex<Option<SlintMpvPlayer>>>,
) {
    let watcher_handle = main_window.as_weak();
    let watcher_fb_handle = floating_btn.as_weak();
    let player_watcher = player_shared.clone();
    let last_dismiss_watcher = get_last_dismiss_tracker();

    std::thread::spawn(move || {
        let mut inactivity_slot = 0usize;
        let mut screensaver_start_time = Instant::now();

        loop {
            std::thread::sleep(std::time::Duration::from_millis(500));
            let idle_seconds = get_system_idle_seconds();
            let cfg = load_config();
            let timeout = cfg.inactivity_timeout_sec;

            let time_since_dismiss = last_dismiss_watcher
                .lock()
                .map(|t| t.elapsed().as_secs())
                .unwrap_or(u64::MAX);

            let was_screensaver = IS_SCREENSAVER_ACTIVE.load(Ordering::SeqCst);
            let in_try_pc = IS_IN_TRY_PC_MODE.load(Ordering::SeqCst);

            // 1. In Try-PC Mode and Idle >= Timeout: Auto-restore Main Window AND trigger Inactivity Screensaver
            if in_try_pc && idle_seconds >= timeout {
                IS_IN_TRY_PC_MODE.store(false, Ordering::SeqCst);
                IS_SCREENSAVER_ACTIVE.store(true, Ordering::SeqCst);
                screensaver_start_time = Instant::now();

                let video_list = cfg.get_inactivity_video_list();
                if !video_list.is_empty() {
                    let selected_rel_path = &video_list[inactivity_slot % video_list.len()];
                    inactivity_slot = inactivity_slot.wrapping_add(1);
                    let promo_video = crate::domains::media::resolve_video_path(selected_rel_path);
                    if promo_video.exists() {
                        if let Ok(mut guard) = player_watcher.lock() {
                            if let Some(ref mut p) = *guard {
                                p.change_video_looping(&promo_video.to_string_lossy(), false);
                            }
                        }
                    }
                }

                let wh = watcher_handle.clone();
                let fbh = watcher_fb_handle.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(fb) = fbh.upgrade() {
                        let _ = fb.hide();
                    }
                    if let Some(ui) = wh.upgrade() {
                        let _ = ui.show();
                        ui.window().set_fullscreen(true);
                        ui.set_is_screensaver_active(true);
                        restore_main_window();
                    }
                });
                continue;
            }

            // 2. On Main Window (Not in Try-PC, Not in Screensaver): Trigger Inactivity Screensaver
            if !in_try_pc && !was_screensaver {
                if idle_seconds >= timeout && time_since_dismiss >= timeout {
                    let video_list = cfg.get_inactivity_video_list();
                    if !video_list.is_empty() {
                        let selected_rel_path = &video_list[inactivity_slot % video_list.len()];
                        inactivity_slot = inactivity_slot.wrapping_add(1);
                        let promo_video = crate::domains::media::resolve_video_path(selected_rel_path);
                        if promo_video.exists() {
                            if let Ok(mut guard) = player_watcher.lock() {
                                if let Some(ref mut p) = *guard {
                                    p.change_video_looping(&promo_video.to_string_lossy(), false);
                                }
                            }
                        }
                    }

                    IS_SCREENSAVER_ACTIVE.store(true, Ordering::SeqCst);
                    screensaver_start_time = Instant::now();

                    let wh = watcher_handle.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = wh.upgrade() {
                            ui.set_is_screensaver_active(true);
                        }
                    });
                }
            } else if was_screensaver {
                // 3. Currently in Screensaver: Dismiss if user interacts OR if inactivity video finishes
                let mut video_finished = false;
                if screensaver_start_time.elapsed().as_secs() >= 2 {
                    if let Ok(guard) = player_watcher.lock() {
                        if let Some(ref p) = *guard {
                            video_finished = p.is_eof();
                            if !video_finished {
                                if let (Some(pos), Some(dur)) = (p.get_time_pos(), p.get_duration()) {
                                    if dur > 1.0 && pos >= dur - 0.5 {
                                        video_finished = true;
                                    }
                                }
                            }
                        }
                    }
                }

                if idle_seconds < 2 || video_finished {
                    IS_SCREENSAVER_ACTIVE.store(false, Ordering::SeqCst);
                    if let Ok(mut t) = last_dismiss_watcher.lock() {
                        *t = Instant::now();
                    }

                    let landing_video = crate::domains::media::resolve_showroom_video_path();
                    if let Ok(mut guard) = player_watcher.lock() {
                        if let Some(ref mut p) = *guard {
                            p.change_video_looping(&landing_video.to_string_lossy(), true);
                        }
                    }

                    let wh = watcher_handle.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = wh.upgrade() {
                            ui.set_is_screensaver_active(false);
                        }
                    });
                }
            }
        }
    });
}
