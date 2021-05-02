use std::{cell::RefCell, env, rc::Rc};

use crate::{
    cli::CommonOpts,
    tui::{logger, views},
    util,
};

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
    views::init_menu(&mut siv);

    // set up logger
    if let Some(level) = common_opts.verbose.log_level() {
        logger::setup_logger(level);
    }

    // prepare a window
    let mut b = views::BrowserView::named(Rc::new(RefCell::new(siv.cb_sink().clone())));
    if b.get_mut().navigate_to(start_url).is_err() {
        return 1;
    };
    siv.add_fullscreen_layer(b);

    // start event loop
    siv.run();

    // exit successfully after the event loop finishes
    return 0;
}
