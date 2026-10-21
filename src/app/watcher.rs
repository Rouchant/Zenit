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
        let mut playlist_slot = 0usize;
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
                playlist_slot = 0;

                let video_list = cfg.get_inactivity_video_list();
                if !video_list.is_empty() {
                    let promo_video = crate::domains::media::resolve_video_path(&video_list[0]);
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
                    playlist_slot = 0;
                    let video_list = cfg.get_inactivity_video_list();
                    if !video_list.is_empty() {
                        let promo_video = crate::domains::media::resolve_video_path(&video_list[0]);
                        if promo_video.exists() {
                            println!(
                                "[Inactividad] Inactividad de {}s alcanzada (idle: {}s, dismiss: {}s). Iniciando video: {}",
                                timeout, idle_seconds, time_since_dismiss, promo_video.display()
                            );
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
                            let _ = ui.show();
                            ui.window().set_fullscreen(true);
                            ui.set_is_screensaver_active(true);
                            restore_main_window();
                        }
                    });
                }
            } else if was_screensaver {
                // 3. Currently in Screensaver:
                // A) Dismiss immediately if user interacts (< 2s idle)
                if idle_seconds < 2 {
                    println!("[Inactividad] Movimiento o interacción detectada. Regresando a ventana principal. Contador reiniciado.");
                    playlist_slot = 0;
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
                            let _ = ui.show();
                            ui.window().set_fullscreen(true);
                            ui.set_is_screensaver_active(false);
                            restore_main_window();
                        }
                    });
                    continue;
                }

                // B) Check if current inactivity video finished
                let mut video_finished = false;
                if screensaver_start_time.elapsed().as_millis() >= 1200 {
                    if let Ok(guard) = player_watcher.lock() {
                        if let Some(ref p) = *guard {
                            if p.is_eof() {
                                video_finished = true;
                            } else if let (Some(pos), Some(dur)) = (p.get_time_pos(), p.get_duration()) {
                                if dur > 1.0 && pos >= dur - 0.4 {
                                    video_finished = true;
                                }
                            }
                        }
                    }
                }

                if video_finished {
                    let video_list = cfg.get_inactivity_video_list();
                    if playlist_slot + 1 < video_list.len() {
                        // Play next video in the inactivity playlist
                        playlist_slot += 1;
                        let next_rel = &video_list[playlist_slot];
                        let promo_video = crate::domains::media::resolve_video_path(next_rel);
                        if promo_video.exists() {
                            println!(
                                "[Inactividad] Reproduciendo siguiente video de lista (slot {}): {}",
                                playlist_slot + 1, promo_video.display()
                            );
                            if let Ok(mut guard) = player_watcher.lock() {
                                if let Some(ref mut p) = *guard {
                                    p.change_video_looping(&promo_video.to_string_lossy(), false);
                                }
                            }
                            screensaver_start_time = Instant::now();
                        } else {
                            // Video file does not exist, finish playlist and return
                            video_finished = false;
                            playlist_slot = video_list.len();
                        }
                    }
                    
                    if playlist_slot + 1 >= video_list.len() && video_finished {
                        // All inactivity videos completed: return to main showroom window!
                        println!(
                            "[Inactividad] Video de inactividad finalizado. Regresando a ventana principal. Contador de inactividad de {}s reiniciado.",
                            timeout
                        );
                        playlist_slot = 0;
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
                                let _ = ui.show();
                                ui.window().set_fullscreen(true);
                                ui.set_is_screensaver_active(false);
                                restore_main_window();
                            }
                        });
                        continue;
                    }
                }
            }
        }
    });
}
