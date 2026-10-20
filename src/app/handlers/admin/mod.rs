pub mod auth;
pub mod defaults;
pub mod detect;
pub mod save;
pub mod system;

use crate::domains::media::SlintMpvPlayer;
use crate::MainWindow;
use std::sync::{Arc, Mutex};

pub fn setup_admin_handlers(
    main_window: &MainWindow,
    player_shared: Arc<Mutex<Option<SlintMpvPlayer>>>,
) {
    auth::setup_auth_handlers(main_window);
    detect::setup_detect_handlers(main_window);
    save::setup_save_handlers(main_window, player_shared.clone());
    defaults::setup_defaults_handlers(main_window);
    system::setup_system_handlers(main_window, player_shared);
}
