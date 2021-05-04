use cursive::logger;
use log::{set_max_level, Level};

pub fn setup_logger(level: Level) {
    logger::init();
    set_max_level(level.to_level_filter());
}
