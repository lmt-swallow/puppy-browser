use crate::{
    cli::CommonOpts,
    html, resource,
    ui::{self, components::NavigationBar, ElementContainer},
};
use cursive::{
    event::Key,
    menu,
    traits::{Boxable, Nameable},
    views::{LinearLayout, Panel, ScrollView},
    Cursive, CursiveRunnable,
};
use cursive::{logger, views::Dialog};
#[allow(unused_imports)]
use cursive_aligned_view::Alignable;

use log::{error, info, set_max_level};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Opts {
    pub url: Option<String>,
}

fn enable_menubar(siv: &mut CursiveRunnable) {
    siv.menubar()
        .add_subtree(
            "Operation",
            menu::Tree::new().leaf("Toggle debug console", |s| {
                s.toggle_debug_console();
            }),
        )
        .add_subtree(
            "Help",
            menu::Tree::new().leaf("About", |s| {
                s.add_layer(Dialog::info(format!("Puppy {}", env!("CARGO_PKG_VERSION"))))
            }),
        )
        .add_delimiter()
        .add_leaf("Quit", |s| s.quit());

    siv.set_autohide_menu(false);
    siv.add_global_callback(Key::Esc, |s| s.select_menubar());
}

pub fn navigate(s: &mut Cursive, url: String) {
    info!("start to navigate to {}", url);

    // TODO (enhancement): error handling
    info!("fetch a resource from {}", url);
    let source = resource::fetch(&url);
    if let Err(_e) = source {
        error!("failed to fetch {}; {}", url, _e);
        return;
    }

    // TODO (enhancement): error handling
    info!("parse the resource from {}", url);
    let document = html::parse(source.unwrap());
    if let Err(_e) = document {
        error!("failed to parse {}; {}", url, _e);
        return;
    }

    // TODO (future): render with rendering tree instead of DOM itself.
    info!("render the DOM of {}", url);
    if s.call_on_name("content", |view: &mut ElementContainer| {
        for i in 0..view.len() {
            view.remove_child(0);
        }

        ui::render_node_from_document(view, &document.unwrap());
    })
    .is_none()
    {
        error!("failed to render {}; no element container found", url);
    }
}

pub fn run(common_opts: CommonOpts, opts: Opts) -> i32 {
    let start_url = opts.url.unwrap_or("http://example.com".to_string());

    // set up base
    let mut siv = cursive::default();
    ui::theme::set_default_theme(&mut siv);
    enable_menubar(&mut siv);

    // set up logger
    logger::init();
    if let Some(level) = common_opts.verbose.log_level() {
        set_max_level(level.to_level_filter());
    }

    // build window structure
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

    let layer = LinearLayout::vertical().child(navbar).child(content);
    siv.add_fullscreen_layer(layer);

    // navigate to the first page
    navigate(&mut siv, start_url.clone());

    // start event loop
    siv.run();

    // exit successfully after the event loop finishes
    return 0;
}
