use cursive::{view::View, Printer};

pub struct HorizontalLine {}

impl HorizontalLine {
    pub fn new() -> Self {
        HorizontalLine {}
    }
}
impl View for HorizontalLine {
    fn draw(&self, printer: &Printer<'_, '_>) {
        printer.print_hdelim((0, 0), printer.size.x);
    }
}

pub struct VerticalLine {}

impl VerticalLine {
    pub fn new() -> Self {
        VerticalLine {}
    }
}

impl View for VerticalLine {
    fn draw(&self, printer: &Printer<'_, '_>) {
        printer.print_vline((0, 0), printer.size.y, "∣");
    }
}
