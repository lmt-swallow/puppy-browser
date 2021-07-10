//! This module defines loggers for the TUI of puppy.

use cursive::logger;
use log::{set_max_level, Level};

/// `setup_logger` initializes a logger with the given log level for TUI.
pub fn setup_logger(level: Level) {
    logger::init();
    set_max_level(level.to_level_filter());
}
