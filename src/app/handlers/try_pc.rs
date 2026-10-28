use crate::app::state::IS_IN_TRY_PC_MODE;
use crate::app::window::{minimize_main_window, position_floating_button, restore_main_window};
use crate::domains::media::SlintMpvPlayer;
use crate::{FloatingDetailsButton, MainWindow};
use slint::ComponentHandle;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

pub fn setup_try_pc_handlers(
    main_window: &MainWindow,
    floating_btn: &FloatingDetailsButton,
    player_shared: Arc<Mutex<Option<SlintMpvPlayer>>>,
) {
    // 1. Restore Clicked on Floating Button
    let restore_mw_handle = main_window.as_weak();
    let restore_fb_handle = floating_btn.as_weak();
    let player_restore = player_shared.clone();

    floating_btn.on_restore_clicked(move || {
        IS_IN_TRY_PC_MODE.store(false, Ordering::SeqCst);
        let landing_video = crate::domains::media::resolve_showroom_video_path();
        if let Ok(mut guard) = player_restore.lock() {
            if let Some(ref mut p) = *guard {
                p.change_video_looping(&landing_video.to_string_lossy(), true);
            }
        }

        let mwh = restore_mw_handle.clone();
        let fbh = restore_fb_handle.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(mw) = mwh.upgrade() {
                let _ = mw.show();
                mw.window().set_fullscreen(true);
                mw.set_is_screensaver_active(false);
            }

            restore_main_window();

            if let Some(fb) = fbh.upgrade() {
                let _ = fb.hide();
            }
        });
    });

    // 2. Try PC Mode Clicked on Main Window
    let try_pc_mw_handle = main_window.as_weak();
    let try_pc_fb_handle = floating_btn.as_weak();
    let player_try = player_shared.clone();

    main_window.on_try_pc_clicked(move || {
        println!("[TryPC] Botón 'Probar esta PC' activado. Ocultando showroom...");
        IS_IN_TRY_PC_MODE.store(true, Ordering::SeqCst);
        if let Ok(mut t) = crate::app::state::get_try_pc_start_tracker().lock() {
            *t = std::time::Instant::now();
        }

        if let Ok(mut guard) = player_try.lock() {
            if let Some(ref mut p) = *guard {
                p.pause();
            }
        }

        // 1. Mostrar y posicionar el botón flotante PRIMERO para que el event loop de Slint
        // nunca se quede con 0 ventanas visibles (si llega a 0, Slint interpreta que la app debe cerrarse)
        if let Some(fb) = try_pc_fb_handle.upgrade() {
            let cfg = crate::domains::showcase::load_config();
            crate::app::sync::sync_floating_button_theme(&fb, &cfg);
            let _ = fb.show();
            position_floating_button();
        }

        // 2. Ocultar la ventana principal a nivel Win32 (sin animación y sin cerrar el proceso)
        minimize_main_window();
        if let Some(mw) = try_pc_mw_handle.upgrade() {
            let _ = mw.hide();
        }
    });

    // 3. Clic en el icono de Zenit en la barra de tareas => mismo efecto que "Ver detalles"
    let taskbar_fb_handle = floating_btn.as_weak();
    let taskbar_timer: &'static slint::Timer = Box::leak(Box::new(slint::Timer::default()));
    taskbar_timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(250),
        move || {
            if !IS_IN_TRY_PC_MODE.load(Ordering::SeqCst) {
                return;
            }
            // Período de gracia de 1.5s tras entrar a Probar PC para evitar falsos positivos
            // generados por el traspaso de foco entre la ventana principal y la flotante
            let elapsed_ms = crate::app::state::get_try_pc_start_tracker()
                .lock()
                .map(|t| t.elapsed().as_millis())
                .unwrap_or(0);
            if elapsed_ms < 1500 {
                return;
            }

            if crate::app::window::floating_button_taskbar_activated() {
                if let Some(fb) = taskbar_fb_handle.upgrade() {
                    fb.invoke_restore_clicked();
                }
            }
        },
    );
}
