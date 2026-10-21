pub mod app;
pub mod domains;

use std::rc::Rc;
use std::sync::{Arc, Mutex};

slint::include_modules!();

pub fn run() {
    let main_window = MainWindow::new().expect("Failed to initialize Slint MainWindow");
    let floating_btn = Rc::new(FloatingDetailsButton::new().expect("Failed to initialize FloatingDetailsButton"));

    // 1. Initialize Main Window Geometry & Win32 Handles
    app::window::init_main_window(&main_window);

    // 2. Populate UI State & Vault Data
    app::sync::populate_ui_state(&main_window);

    // 3. Initialize libmpv Player
    let initial_video = domains::media::resolve_showroom_video_path();
    let video_path_str = initial_video.to_string_lossy().to_string();
    let player_shared: Arc<Mutex<Option<domains::media::SlintMpvPlayer>>> = Arc::new(Mutex::new(None));

    match domains::media::SlintMpvPlayer::new(&video_path_str, 1280, 720) {
        Ok(player) => {
            println!("[libmpv] Reproductor landing inicializado con video: {}", video_path_str);
            main_window.set_has_video_stream(true);
            *player_shared.lock().unwrap() = Some(player);
        }
        Err(err) => {
            println!("[libmpv error] {}", err);
            main_window.set_has_video_stream(false);
        }
    }

    // 4. Set up Video Render (60 FPS) and Live Clock Timers
    let _video_timer = app::timers::setup_video_timer(&main_window, player_shared.clone());
    let _clock_timer = app::timers::setup_clock_timer(&main_window);

    // 5. Register Event Handlers
    app::handlers::setup_admin_handlers(&main_window, player_shared.clone());
    app::handlers::setup_try_pc_handlers(&main_window, &floating_btn, player_shared.clone());
    app::handlers::setup_media_handlers(&main_window);

    // 6. Spawn Background Inactivity Watcher
    app::watcher::spawn_inactivity_watcher(&main_window, &floating_btn, player_shared.clone());

    // 7. Start native Slint event loop (global loop keeps running even when main_window is minimized/hidden)
    slint::run_event_loop().expect("Error running Slint application");
}
