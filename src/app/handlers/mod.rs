pub mod admin;
pub mod media;
pub mod try_pc;

pub use admin::setup_admin_handlers;
pub use media::setup_media_handlers;
pub use try_pc::setup_try_pc_handlers;
