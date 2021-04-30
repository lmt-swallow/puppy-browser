use std::{cell::RefCell, rc::Rc};

use cursive::{
    direction::Direction,
    event::{Event, EventResult},
    theme::{BaseColor, Color, PaletteColor, Theme},
    traits::Finder,
    view::{CannotFocus, Nameable, Resizable, Selector, ViewNotFound},
    views::{Button, EditView, LinearLayout, Panel, ResizedView, ThemedView},
    Cursive, Printer, Rect, Vec2, View, With, XY,
};
use log::{error, info};

use super::traits::Clearable;
use crate::{
    fetch::{self, Request},
    html,
    ui::ElementContainer,
};

pub static NAVIGATION_INPUT_NAME: &str = "navbar-input";
pub static NAVIGATION_BUTTON_NAME: &str = "navbar-button";

pub struct NavigationBar {
    layout: Rc<RefCell<LinearLayout>>,
}

impl NavigationBar {
    pub fn new(default_value: String) -> NavigationBar {
        let theme = Theme::default().with(|theme| {
            theme.palette[PaletteColor::Primary] = Color::Dark(BaseColor::Black);
            theme.palette[PaletteColor::View] = Color::Dark(BaseColor::Black);
            theme.palette[PaletteColor::Secondary] = Color::Dark(BaseColor::White);
        });

        NavigationBar {
            layout: Rc::new(RefCell::new(
                LinearLayout::vertical().child(
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
            )),
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
            .layout
            .borrow_mut()
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
        let layout = self.layout.clone();
        if self
            .layout
            .borrow_mut()
            .call_on_name(NAVIGATION_BUTTON_NAME, |view: &mut Button| {
                view.set_callback(move |s| {
                    if layout
                        .borrow_mut()
                        .call_on_name(NAVIGATION_INPUT_NAME, |view: &mut EditView| {
                            cb_button(s, view.get_content().to_string());
                        })
                        .is_none()
                    {
                        error!("failed to find navigation input");
                    };
                });
            })
            .is_none()
        {
            error!("failed to find navigation button");
        };
    }
}

impl View for NavigationBar {
    fn draw(&self, printer: &Printer<'_, '_>) {
        self.layout.borrow_mut().draw(printer)
    }

    fn layout(&mut self, size: Vec2) {
        self.layout.borrow_mut().layout(size)
    }

    fn needs_relayout(&self) -> bool {
        self.layout.borrow_mut().needs_relayout()
    }

    fn required_size(&mut self, constraint: XY<usize>) -> XY<usize> {
        self.layout.borrow_mut().required_size(constraint)
    }

    fn focus_view(&mut self, selector: &Selector<'_>) -> Result<EventResult, ViewNotFound> {
        self.layout.borrow_mut().focus_view(selector)
    }

    fn take_focus(&mut self, source: Direction) -> Result<EventResult, CannotFocus> {
        self.layout.borrow_mut().take_focus(source)
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        self.layout.borrow_mut().on_event(event)
    }

    fn important_area(&self, view_size: Vec2) -> Rect {
        self.layout.borrow_mut().important_area(view_size)
    }
}

pub fn resolve_and_navigate(s: &mut Cursive, possibly_relative_url: String) {
    todo!();
    navigate(s, possibly_relative_url)
}

pub fn navigate(s: &mut Cursive, absolute_url: String) {
    info!("start to navigate to {}", absolute_url);

    // TODO (enhancement): error handling
    if s.screen_mut()
        .call_on_name("content", |view: &mut ElementContainer| view.clear())
        .is_none()
    {
        error!(
            "failed to clear the current view to render {}; no element container found",
            absolute_url
        );
    }

    info!("fetch a resource from {}", absolute_url);
    let req = Request::new(absolute_url.clone());
    let response = fetch::fetch(req);
    if let Err(_e) = response {
        error!("failed to fetch {}; {}", absolute_url, _e);
        return;
    }

    info!("parse the resource from {}", absolute_url);
    let document = html::parse(response.unwrap());
    if let Err(_e) = document {
        error!("failed to parse {}; {}", absolute_url, _e);
        return;
    }

    info!("render the DOM of {}", absolute_url);
    if s.screen_mut()
        .call_on_name("content", |view: &mut ElementContainer| {
            super::render_node_from_document(view, &document.unwrap());
        })
        .is_none()
    {
        error!(
            "failed to render {}; no element container found",
            absolute_url
        );
    }
}
