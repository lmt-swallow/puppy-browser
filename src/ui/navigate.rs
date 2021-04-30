use crate::{
    fetch::{self, Request},
    html,
    ui::{self, ElementContainer},
};
use cursive::Cursive;
#[allow(unused_imports)]
use cursive_aligned_view::Alignable;

use log::{error, info};

pub fn resolve_and_navigate(s: &mut Cursive, possibly_relative_url: String) {
    todo!();
    navigate(s, possibly_relative_url)
}

pub fn navigate(s: &mut Cursive, absolute_url: String) {
    info!("start to navigate to {}", absolute_url);

    // TODO (enhancement): error handling
    info!("clear the current view to render {}", absolute_url);
    if s.call_on_name("content", |view: &mut ElementContainer| {
        for _ in 0..view.len() {
            view.remove_child(0);
        }
    })
    .is_none()
    {
        error!(
            "failed to clear the current view to render {}; no element container found",
            absolute_url
        );
    }

    // TODO (enhancement): error handling
    info!("fetch a resource from {}", absolute_url);
    let req = Request::new(absolute_url.clone());
    let response = fetch::fetch(req);
    if let Err(_e) = response {
        error!("failed to fetch {}; {}", absolute_url, _e);
        return;
    }

    // TODO (enhancement): error handling
    info!("parse the resource from {}", absolute_url);
    let document = html::parse(response.unwrap());
    if let Err(_e) = document {
        error!("failed to parse {}; {}", absolute_url, _e);
        return;
    }

    // TODO (future): render with rendering tree instead of DOM itself.
    info!("render the DOM of {}", absolute_url);
    if s.call_on_name("content", |view: &mut ElementContainer| {
        ui::render_node_from_document(view, &document.unwrap());
    })
    .is_none()
    {
        error!(
            "failed to render {}; no element container found",
            absolute_url
        );
    }
}
