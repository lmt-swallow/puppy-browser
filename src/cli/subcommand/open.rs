//! This module defines `open` subcommand.

use std::{env, rc::Rc};

use crate::{
    cli::CommonOpts,
    tui::{init_menu, setup_logger, BrowserView},
    util,
};

use structopt::StructOpt;

/// `Opts` defines options for the `open` subcommand.
#[derive(StructOpt, Debug)]
pub struct Opts {
    pub url: Option<String>,
}

/// `run` launches a TUI window to show the main UI.
pub fn run(common_opts: CommonOpts, opts: Opts) -> i32 {
    let start_url = opts
        .url
        .and_then(|u| Some(util::normalize_fileurl_with(env::current_dir().unwrap(), u)))
        .unwrap_or("http://example.com".to_string());

    // set up base
    let mut siv = cursive::default();
    init_menu(&mut siv);

    // set up logger
    if let Some(level) = common_opts.verbose.log_level() {
        setup_logger(level);
    }

    // prepare a window
    let mut b = BrowserView::named(Rc::new(siv.cb_sink().clone()));
    b.get_mut().navigate_to(start_url);
    siv.add_fullscreen_layer(b);

    // start event loop
    siv.run();

    // exit successfully after the event loop finishes
    return 0;
}
