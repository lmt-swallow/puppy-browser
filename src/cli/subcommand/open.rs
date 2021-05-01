use std::env;

use crate::{
    cli::CommonOpts,
    ui::{self, BrowserView},
    util,
};
use cursive::logger;

use log::set_max_level;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Opts {
    pub url: Option<String>,
}

pub fn run(common_opts: CommonOpts, opts: Opts) -> i32 {
    let start_url = opts
        .url
        .and_then(|u| Some(util::normalize_fileurl_with(env::current_dir().unwrap(), u)))
        .unwrap_or("http://example.com".to_string());

    // set up base
    let mut siv = cursive::default();
    ui::menu::init_menu(&mut siv);

    // set up logger
    logger::init();
    if let Some(level) = common_opts.verbose.log_level() {
        set_max_level(level.to_level_filter());
    }

    // prepare a window
    let mut b = BrowserView::named();
    if b.get_mut().navigate_to(start_url).is_err() {
        return 1;
    };
    siv.add_fullscreen_layer(b);

    // start event loop
    siv.run();

    // exit successfully after the event loop finishes
    return 0;
}
