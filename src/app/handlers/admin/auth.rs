use crate::app::sync::populate_ui_state;
use crate::domains::showcase::verify_password;
use crate::MainWindow;
use slint::ComponentHandle;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

pub fn setup_auth_handlers(main_window: &MainWindow) {
    // 1. 4-Tap Admin Trigger in top-right corner
    let tap_tracker = Rc::new(RefCell::new((0u32, Instant::now())));
    let tap_handle = main_window.as_weak();
    main_window.on_corner_tap_clicked(move || {
        let mut st = tap_tracker.borrow_mut();
        let now = Instant::now();
        if now.duration_since(st.1).as_millis() < 800 {
            st.0 += 1;
        } else {
            st.0 = 1;
        }
        st.1 = now;

        if st.0 >= 4 {
            st.0 = 0;
            if let Some(ui) = tap_handle.upgrade() {
                ui.set_is_admin_open(true);
                ui.set_is_admin_authenticated(false);
                ui.set_admin_error_msg("".into());
                ui.set_admin_password_input("".into());
            }
        }
    });

    // 2. Close Admin Panel
    let close_admin_handle = main_window.as_weak();
    main_window.on_close_admin_clicked(move || {
        if let Some(ui) = close_admin_handle.upgrade() {
            ui.set_is_admin_open(false);
            ui.set_is_admin_authenticated(false);
            ui.set_admin_error_msg("".into());
        }
    });

    // 3. Verify Admin Password
    let pwd_handle = main_window.as_weak();
    main_window.on_check_admin_password(move |pwd| {
        if let Some(ui) = pwd_handle.upgrade() {
            if verify_password(&pwd) {
                ui.set_is_admin_authenticated(true);
                ui.set_admin_error_msg("".into());
                populate_ui_state(&ui);
            } else {
                ui.set_admin_error_msg("Contraseña incorrecta. Intente nuevamente.".into());
            }
        }
    });
}
