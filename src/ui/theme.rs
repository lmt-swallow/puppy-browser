use cursive::{
    theme::{BaseColor, Color, PaletteColor},
    CursiveRunnable,
};

pub fn set_default_theme(siv: &mut CursiveRunnable) {
    let mut theme = siv.current_theme().clone();
    theme.palette[PaletteColor::Background] = Color::Dark(BaseColor::White);
    theme.palette[PaletteColor::View] = Color::Dark(BaseColor::White);
    siv.set_theme(theme);
}
