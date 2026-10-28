use std::sync::atomic::{AtomicBool, AtomicIsize};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

pub static IS_IN_TRY_PC_MODE: AtomicBool = AtomicBool::new(false);
pub static IS_SCREENSAVER_ACTIVE: AtomicBool = AtomicBool::new(false);
pub static MAIN_HWND: AtomicIsize = AtomicIsize::new(0);
pub static FLOATING_HWND: AtomicIsize = AtomicIsize::new(0);

static TRACKER: OnceLock<Arc<Mutex<Instant>>> = OnceLock::new();
static TRY_PC_TRACKER: OnceLock<Arc<Mutex<Instant>>> = OnceLock::new();

pub fn get_last_dismiss_tracker() -> Arc<Mutex<Instant>> {
    TRACKER.get_or_init(|| Arc::new(Mutex::new(Instant::now()))).clone()
}

pub fn get_try_pc_start_tracker() -> Arc<Mutex<Instant>> {
    TRY_PC_TRACKER.get_or_init(|| Arc::new(Mutex::new(Instant::now()))).clone()
}
