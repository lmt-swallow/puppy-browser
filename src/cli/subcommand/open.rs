use crate::{
    html, source,
    ui::{alert, NavigationBar},
};
use cursive::{
    event::Key,
    menu,
    theme::{BaseColor, PaletteColor},
    traits::Boxable,
    views::{DummyView, LinearLayout, Panel, ScrollView},
    CursiveRunnable,
};
use cursive::{
    theme::Color,
    views::{Dialog, TextView},
};
#[allow(unused_imports)]
use cursive_aligned_view::Alignable;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Opts {
    pub url: Option<String>,
}

fn set_theme(siv: &mut CursiveRunnable) {
    let mut theme = siv.current_theme().clone();
    theme.palette[PaletteColor::Background] = Color::Dark(BaseColor::White);
    theme.palette[PaletteColor::View] = Color::Dark(BaseColor::White);
    siv.set_theme(theme);
}

fn set_menubar(siv: &mut CursiveRunnable) {
    siv.menubar()
        .add_subtree(
            "Operation",
            menu::Tree::new().leaf("Go", |s| {
                s.add_layer(Dialog::info("TODO (not implemented yet!)"))
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

pub fn run(opts: Opts) -> i32 {
    let start_url = opts.url.unwrap_or("http://example.com".to_string());

    // set up base
    let mut siv = cursive::default();
    set_theme(&mut siv);
    set_menubar(&mut siv);

    // build window
    let navbar = NavigationBar::new(start_url.clone()).on_navigation(|s, to| {
        alert(s, "Debug (TODO)".to_string(), to.to_string());
    });
    let content = Panel::new(ScrollView::new(DummyView).full_height()); // TODO
    let layer = LinearLayout::vertical().child(navbar).child(content);
    siv.add_fullscreen_layer(layer);

    // let source = source::fetch(start_url.clone());
    // // TODO (enhancement): error handling
    // let dom = html::parse(source).unwrap();
    // let root_node = dom.child_nodes.get(0).unwrap();
    // root_node.child_nodes

    // start event loop
    siv.run();

    // exit successfully after the event loop finishes
    return 0;
}
