use std::{cell::RefCell, rc::Rc};

use cursive::{
    direction::Direction,
    event::{Event, EventResult, Key},
    theme::{BaseColor, Color, PaletteColor, Theme},
    traits::Finder,
    view::{CannotFocus, Nameable, Resizable, Selector, ViewNotFound},
    views::{Button, EditView, LinearLayout, Panel, ResizedView, ThemedView},
    Cursive, Printer, Rect, Vec2, View, With, XY,
};

pub struct NavigationBar {
    layout: Rc<RefCell<LinearLayout>>,

    input_id: &'static str,
    button_id: &'static str,
}

impl NavigationBar {
    pub fn new(default_value: String) -> NavigationBar {
        let theme = Theme::default().with(|theme| {
            theme.palette[PaletteColor::Primary] = Color::Dark(BaseColor::Black);
            theme.palette[PaletteColor::View] = Color::Dark(BaseColor::Black);
            theme.palette[PaletteColor::Secondary] = Color::Dark(BaseColor::White);
        });

        NavigationBar {
            input_id: "navbar-input",
            button_id: "navbar-button",
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
                                        .with_name("navbar-input")
                                        .full_width(),
                                ),
                            )))
                            .child(Panel::new(
                                Button::new("Go", |s: &mut Cursive| {})
                                    .with_name("navbar-button")
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
        self.layout
            .borrow_mut()
            .call_on_name(self.input_id, |view: &mut EditView| {
                view.set_on_submit(move |s, text| {
                    cb_input(s, text.to_string());
                });
            });

        let cb_button = callback.clone();
        let layout = self.layout.clone();
        let input_id = self.input_id;
        self.layout
            .borrow_mut()
            .call_on_name(self.button_id, |view: &mut Button| {
                view.set_callback(move |s| {
                    layout
                        .borrow_mut()
                        .call_on_name(input_id, |view: &mut EditView| {
                            view.on_event(Event::Key(Key::Enter));
                            cb_button(s, view.get_content().to_string());
                        });
                });
            });
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
