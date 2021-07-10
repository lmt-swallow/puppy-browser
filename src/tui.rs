//! This module includes some sub-modules to provide text user interface of puppy.

mod api;
mod components;
mod logger;
mod render;
mod views;

pub use self::api::PageViewAPIHandler;
pub use self::logger::setup_logger;
pub use self::views::init_menu;
pub use self::views::BrowserView;
