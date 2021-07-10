//! This module provides an implementation of input forms.

use std::rc::Rc;

use cursive::{event::Callback, theme::BorderStyle, Cursive, With};
use cursive::{
    theme::{BaseColor, Color, PaletteColor, Theme},
    view::ViewWrapper,
    views::{EditView, ThemedView},
};

pub struct TextInputView {
    view: ThemedView<EditView>,
}

impl TextInputView {
    pub fn new() -> Self {
        let theme = Theme::default().with(|theme| {
            theme.palette[PaletteColor::Primary] = Color::Dark(BaseColor::Black);
            theme.palette[PaletteColor::View] = Color::Dark(BaseColor::Black);
            theme.palette[PaletteColor::Secondary] = Color::Dark(BaseColor::Cyan);
            theme.borders = BorderStyle::Simple;
        });

        TextInputView {
            view: ThemedView::new(theme, EditView::new()),
        }
    }

    pub fn content<S: Into<String>>(self, content: S) -> Self {
        self.with(|x| {
            x.with_view_mut(|y| y.with_view_mut(|z| z.set_content(content)));
        })
    }

    pub fn set_content<S: Into<String>>(&mut self, content: S) -> Callback {
        // TODO (enhancement): better error handling
        self.with_view_mut(|y| y.with_view_mut(|z| z.set_content(content)))
            .unwrap()
            .unwrap()
    }

    #[allow(clippy::rc_buffer)]
    pub fn get_content(&self) -> Rc<String> {
        // TODO (enhancement): better error handling
        self.with_view(|x| x.with_view(|y| y.get_content()))
            .unwrap()
            .unwrap()
    }

    pub fn set_on_submit<F>(&mut self, callback: F)
    where
        F: Fn(&mut Cursive, &str) + 'static,
    {
        self.with_view_mut(|y| y.with_view_mut(|z| z.set_on_submit(callback)));
    }
}

impl ViewWrapper for TextInputView {
    type V = ThemedView<EditView>;

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
