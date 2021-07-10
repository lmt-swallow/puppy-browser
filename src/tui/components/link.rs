//! This module provides an implementation of page links.

use cursive::{
    align::HAlign,
    direction::Direction,
    event::*,
    theme::{ColorStyle, Effect, Style},
    view::{CannotFocus, View},
    Cursive, Printer, Vec2,
};
use cursive::{event::Callback, impl_enabled, Rect};
use unicode_width::UnicodeWidthStr;

/// Simple text label with a callback when <Enter> is pressed.
/// A link shows its content in a single line and has a fixed size.
pub struct Link {
    label: String,
    callback: Callback,
    enabled: bool,
    last_size: Vec2,

    invalidated: bool,
}

impl Link {
    impl_enabled!(self.enabled);

    /// Creates a new link with the given content and callback.
    pub fn new<F, S>(label: S, cb: F) -> Self
    where
        F: 'static + Fn(&mut Cursive),
        S: Into<String>,
    {
        let label = label.into();
        Link {
            label: label.into(),
            callback: Callback::from_fn(cb),
            enabled: true,
            last_size: Vec2::zero(),
            invalidated: true,
        }
    }

    pub fn set_callback<F>(&mut self, cb: F)
    where
        F: Fn(&mut Cursive) + 'static,
    {
        self.callback = Callback::from_fn(cb);
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn set_label<S>(&mut self, label: S)
    where
        S: Into<String>,
    {
        self.label = label.into();
        self.invalidate();
    }

    fn req_size(&self) -> Vec2 {
        Vec2::new(self.label.width(), 1)
    }

    fn invalidate(&mut self) {
        self.invalidated = true;
    }
}

impl View for Link {
    fn draw(&self, printer: &Printer) {
        if printer.size.x == 0 {
            return;
        }

        let offset = HAlign::Center.get_offset(self.label.width(), printer.size.x);

        let mut style = Style::default();
        style.color = if !(self.enabled && printer.enabled) {
            ColorStyle::secondary()
        } else if printer.focused {
            ColorStyle::highlight()
        } else {
            ColorStyle::primary()
        };
        style.effects.insert(Effect::Underline);

        printer.with_style(style, |printer| {
            printer.print((offset, 0), &self.label);
        });
    }

    fn layout(&mut self, size: Vec2) {
        self.last_size = size;
        self.invalidated = false;
    }

    fn required_size(&mut self, _: Vec2) -> Vec2 {
        self.req_size()
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        if !self.enabled {
            return EventResult::Ignored;
        }

        let width = self.label.width();
        let self_offset = HAlign::Center.get_offset(width, self.last_size.x);
        match event {
            Event::Key(Key::Enter) => EventResult::Consumed(Some(self.callback.clone())),
            Event::Mouse {
                event: MouseEvent::Release(MouseButton::Left),
                position,
                offset,
            } if position.fits_in_rect(offset + (self_offset, 0), self.req_size()) => {
                EventResult::Consumed(Some(self.callback.clone()))
            }
            _ => EventResult::Ignored,
        }
    }

    fn take_focus(&mut self, _: Direction) -> Result<EventResult, CannotFocus> {
        self.enabled.then(EventResult::consumed).ok_or(CannotFocus)
    }

    fn important_area(&self, view_size: Vec2) -> Rect {
        let width = self.label.width();
        let offset = HAlign::Center.get_offset(width, view_size.x);

        Rect::from_size((offset, 0), (width, 1))
    }

    fn needs_relayout(&self) -> bool {
        self.invalidated
    }
}
