//! This module includes some implementations related to top menu.

use cursive::{event::Key, menu, views::Dialog, CursiveRunnable};

/// `init_menu` adds a top menu to the given cursive instance.
pub fn init_menu(siv: &mut CursiveRunnable) {
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
