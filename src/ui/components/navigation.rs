use std::{error::Error, rc::Rc};

use cursive::{
    theme::{BaseColor, Color, PaletteColor, Theme},
    traits::Finder,
    view::{Nameable, Resizable, ViewWrapper},
    views::{Button, EditView, LinearLayout, Panel, ResizedView, ThemedView},
    Cursive, With,
};
use log::error;

pub static NAVIGATION_INPUT_NAME: &str = "navbar-input";
pub static NAVIGATION_BUTTON_NAME: &str = "navbar-button";

pub struct NavigationBar {
    view: LinearLayout,
}

impl NavigationBar {
    pub fn new(default_value: String) -> NavigationBar {
        let theme = Theme::default().with(|theme| {
            theme.palette[PaletteColor::Primary] = Color::Dark(BaseColor::Black);
            theme.palette[PaletteColor::View] = Color::Dark(BaseColor::Black);
            theme.palette[PaletteColor::Secondary] = Color::Dark(BaseColor::White);
        });

        NavigationBar {
            view: LinearLayout::vertical().child(
                ResizedView::with_full_width(
                    LinearLayout::horizontal()
                        .child(Panel::new(ThemedView::new(
                            theme,
                            ResizedView::with_fixed_height(
                                1,
                                EditView::new()
                                    .content(default_value)
                                    .with_name(NAVIGATION_INPUT_NAME)
                                    .full_width(),
                            ),
                        )))
                        .child(Panel::new(
                            Button::new("Go", |_s: &mut Cursive| {})
                                .with_name(NAVIGATION_BUTTON_NAME)
                                .fixed_width(5)
                                .fixed_height(1),
                        )),
                )
                .fixed_height(3),
            ),
        }
    }

    pub fn on_navigation<F>(self, callback: F) -> Self
    where
        F: Fn(&mut Cursive, String) + 'static,
    {
        self.with(|v| {
            v.set_on_navigation(Rc::new(callback));
        })
    }

    pub fn set_on_navigation<F>(&mut self, callback: Rc<F>)
    where
        F: Fn(&mut Cursive, String) + 'static,
    {
        let cb_input = callback.clone();
        if self
            .view
            .call_on_name(NAVIGATION_INPUT_NAME, |view: &mut EditView| {
                view.set_on_submit(move |s, text| {
                    cb_input(s, text.to_string());
                });
            })
            .is_none()
        {
            error!("failed to find navigation input");
        };

        let cb_button = callback.clone();
        if self
            .view
            .call_on_name(NAVIGATION_BUTTON_NAME, |view: &mut Button| {
                view.set_callback(move |s| {
                    match s
                        .screen_mut()
                        .call_on_name(NAVIGATION_INPUT_NAME, |view: &mut EditView| {
                            view.get_content()
                        }) {
                        Some(url) => cb_button(s, url.to_string()),
                        _ => error!("failed to find navigation input"),
                    };
                });
            })
            .is_none()
        {
            error!("failed to find navigation button");
        };
    }

    pub fn set_url(&mut self, url: String) {
        if self
            .view
            .call_on_name(NAVIGATION_INPUT_NAME, |view: &mut EditView| {
                view.set_content(url);
            })
            .is_none()
        {
            error!("failed to set url to input bar");
        };
    }

    pub fn get_url(&mut self) -> Result<String, Box<dyn Error>> {
        let result = self
            .view
            .call_on_name(NAVIGATION_INPUT_NAME, |view: &mut EditView| {
                view.get_content()
            })
            .ok_or("failed to find input bar")?;
        Ok(result.to_string())
    }
}

impl ViewWrapper for NavigationBar {
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
