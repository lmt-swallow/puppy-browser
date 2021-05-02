use cursive::{
    traits::Finder,
    view::{Nameable, Resizable, ViewWrapper},
    views::{LinearLayout, NamedView, Panel, ScrollView},
    Cursive,
};
use log::error;
use std::error::Error;

use crate::{
    fetch::{fetch, Request},
    html, url,
};

use crate::tui::traits::clearable::Clearable;

use super::{NavigationView, PageView};

pub static BROWSER_VIEW_NAME: &str = "browser-view";
pub static NAVBAR_VIEW_NAME: &str = "browser-view-navbar";
pub static PAGE_VIEW_NAME: &str = "browser-view-page";

pub struct BrowserView {
    view: LinearLayout,
}

impl BrowserView {
    pub fn named() -> NamedView<Self> {
        (BrowserView {
            view: LinearLayout::vertical()
                .child(
                    NavigationView::new("".to_string())
                        .on_navigation(|s, to| {
                            if with_current_browser_view(s, |b: &mut BrowserView| b.navigate_to(to))
                                .is_none()
                            {
                                error!("failed to initiate navigation");
                            };
                        })
                        .with_name(NAVBAR_VIEW_NAME),
                )
                .child(
                    Panel::new(
                        ScrollView::new(
                            PageView::new()
                                .with_name(PAGE_VIEW_NAME)
                                .full_height()
                                .full_screen(),
                        )
                        .full_screen(),
                    )
                    .full_screen(),
                ),
        })
        .with_name(BROWSER_VIEW_NAME)
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

    pub fn navigate_to(&mut self, absolute_url: String) -> Result<(), Box<dyn Error>> {
        self.view
            .call_on_name(NAVBAR_VIEW_NAME, |view: &mut NavigationView| {
                view.set_url(absolute_url.clone())
            })
            .ok_or(format!(
                "failed to clear the current view to render {}; no element container found",
                absolute_url
            ))?;

        self.view
            .call_on_name(PAGE_VIEW_NAME, |view: &mut PageView| {
                view.clear();
            })
            .ok_or(format!(
                "failed to clear the current view to render {}; no element container found",
                absolute_url
            ))?;

        let response = fetch(Request::new(absolute_url.clone()))?;
        let document = html::parse(response)?;

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
