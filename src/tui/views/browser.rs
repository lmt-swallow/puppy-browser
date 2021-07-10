//! This module includes some implementations on top-level views of puppy.

use cursive::{
    traits::Finder,
    view::{Nameable, Resizable, ViewWrapper},
    views::{LinearLayout, NamedView, Panel, ScrollView},
    CbSink, Cursive, With,
};
use log::error;
use std::{error::Error, rc::Rc};

use crate::{
    fetch::{fetch, Request},
    html, url,
};

use super::{NavigationView, PageView};

pub static BROWSER_VIEW_NAME: &str = "browser-view";
pub static NAVBAR_VIEW_NAME: &str = "browser-view-navbar";
pub static PAGE_VIEW_NAME: &str = "browser-view-page";
pub static PAGE_VIEW_CONTAINER_NAME: &str = "browser-view-page-container";

/// `BrowserView` is a main view of puppy.
pub struct BrowserView {
    view: LinearLayout,
    ui_cb_sink: Rc<CbSink>,
}

impl BrowserView {
    pub fn named(ui_cb_sink: Rc<CbSink>) -> NamedView<Self> {
        (BrowserView {
            ui_cb_sink: ui_cb_sink.clone(),
            view: LinearLayout::vertical(),
        })
        .with(|view| {
            view.add_named_navigation_container();
            view.add_named_page_container();
        })
        .with_name(BROWSER_VIEW_NAME)
    }

    fn add_named_navigation_container(&mut self) {
        self.view.add_child(
            NavigationView::new("".to_string())
                .on_navigation(|s, to| {
                    with_current_browser_view(s, |b: &mut BrowserView| b.navigate_to(to));
                })
                .with_name(NAVBAR_VIEW_NAME),
        )
    }

    fn add_named_page_container(&mut self) {
        self.view.add_child(
            Panel::new(
                ScrollView::new(
                    PageView::new(self.ui_cb_sink.clone())
                        .with_name(PAGE_VIEW_NAME)
                        .full_screen(),
                )
                .full_screen(),
            )
            .full_screen()
            .with_name(PAGE_VIEW_CONTAINER_NAME),
        )
    }

    pub fn with_page_view_mut<F, R>(&mut self, f: F) -> ::std::option::Option<R>
    where
        F: FnOnce(&mut PageView) -> R,
    {
        self.view
            .call_on_name(PAGE_VIEW_NAME, |s: &mut PageView| f(s))
    }

    pub fn current_url(&mut self) -> Result<String, Box<dyn Error>> {
        self.view
            .call_on_name(NAVBAR_VIEW_NAME, |view: &mut NavigationView| view.get_url())
            .ok_or("failed to find navbar")?
    }

    pub fn resolve_url(&mut self, possibly_relative_url: String) -> Result<String, Box<dyn Error>> {
        if possibly_relative_url.starts_with("http://")
            || possibly_relative_url.starts_with("https://")
            || possibly_relative_url.starts_with("file://")
        {
            Ok(possibly_relative_url)
        } else {
            let current_url = self.current_url()?;
            let u = url::Url::parse(current_url.as_str())?;
            let absolute_url = u.join(possibly_relative_url.as_str())?;
            Ok(absolute_url.to_string())
        }
    }

    pub fn navigate_to(&mut self, absolute_url: String) {
        match self.navigate_to_intl(absolute_url) {
            Err(e) => {
                error!("failed to navigate; {}", e);
            }
            _ => {}
        };
    }

    fn navigate_to_intl(&mut self, absolute_url: String) -> Result<(), Box<dyn Error>> {
        // change navigation content
        self.view
            .call_on_name(NAVBAR_VIEW_NAME, |view: &mut NavigationView| {
                view.set_url(absolute_url.clone())
            })
            .ok_or(format!(
                "failed to navigate to {}; no element container found",
                absolute_url
            ))?;

        // remove PageView instance
        self.view.remove_child(1).ok_or(format!(
            "failed to navigate to {}; failed to find page container",
            absolute_url
        ))?;

        // add a new PageView instance
        self.add_named_page_container();

        // fetch & parse document
        let response = fetch(Request::new(absolute_url.clone()))?;
        let document = html::parse(response)?;

        // set the document to PageView
        self.view
            .call_on_name(PAGE_VIEW_NAME, |view: &mut PageView| {
                view.init_page(document)
            })
            .ok_or(format!(
                "failed to render {}; no element container found",
                absolute_url
            ))??;

        Ok(())
    }
}

pub fn with_current_browser_view<Output, F>(s: &mut Cursive, f: F) -> Option<Output>
where
    F: FnOnce(&mut BrowserView) -> Output,
{
    s.screen_mut().call_on_name(BROWSER_VIEW_NAME, f)
}

impl ViewWrapper for BrowserView {
    type V = LinearLayout;

    fn with_view<F, R>(&self, f: F) -> ::std::option::Option<R>
    where
        F: FnOnce(&Self::V) -> R,
    {
        Some(f(&self.view))
    }

    fn with_view_mut<F, R>(&mut self, f: F) -> ::std::option::Option<R>
    where
        F: ::std::ops::FnOnce(&mut Self::V) -> R,
    {
        Some(f(&mut self.view))
    }

    fn into_inner(self) -> ::std::result::Result<Self::V, Self>
    where
        Self::V: ::std::marker::Sized,
    {
        Ok(self.view)
    }
}
