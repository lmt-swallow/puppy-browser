//! This module provides a modal UI implementation.

use cursive::{
    views::{Dialog, TextView},
    Cursive,
};

pub fn alert(s: &mut Cursive, title: String, content: String) {
    s.add_layer(
        Dialog::around(TextView::new(content))
            .title(title)
            .button("Quit", |s| {
                s.pop_layer();
            }),
    );
}
