use std::env;

use crate::{
    cli::CommonOpts,
    ui::{self, navigate, navigation::NavigationBar},
    util,
};
use cursive::logger;
use cursive::{
    traits::{Boxable, Nameable},
    views::{LinearLayout, Panel, ScrollView},
};
#[allow(unused_imports)]
use cursive_aligned_view::Alignable;

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
    ui::theme::init_theme(&mut siv);
    ui::menu::init_menu(&mut siv);

    // set up logger
    logger::init();
    if let Some(level) = common_opts.verbose.log_level() {
        set_max_level(level.to_level_filter());
    }

    // prepare a window
    let navbar = NavigationBar::new(start_url.clone()).on_navigation(|s, to| {
        navigate(s, to);
    });
    let content = Panel::new(
        ScrollView::new(
            LinearLayout::vertical()
                .with_name("content")
                .full_height()
                .full_screen(),
        )
        .full_screen(),
    )
    .full_screen();

    let window = LinearLayout::vertical().child(navbar).child(content);
    siv.add_fullscreen_layer(window);

    // navigate to the first page
    navigate(&mut siv, start_url.clone());

    // start event loop
    siv.run();

    // exit successfully after the event loop finishes
    return 0;
}
