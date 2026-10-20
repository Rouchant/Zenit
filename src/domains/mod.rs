//! Zenit Screaming Architecture - Domain Layer
//!
//! Each module screams what the application does:
//! - `hardware`: Low-level Win32 hardware inspection (CPU, GPU, RAM, Storage, Battery, Panel Hz)
//! - `showcase`: Presentation, pricing, SKU, theming, and showroom configuration
//! - `kiosk`: Inactivity screensaver, trial mode tracking, and desktop recovery guards
//! - `media`: Protected Video Vault isolated from accidental user deletion

pub mod hardware;
pub mod showcase;
pub mod kiosk;
pub mod media;
