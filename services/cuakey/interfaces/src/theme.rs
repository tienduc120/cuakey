//! What every window of the app shares: the light or dark theme the user chose, whatever theme Windows uses, drawn in
//! the colours of Windows 11 so the app sits beside the system's own windows, with its own room between the controls.
//! Controls sit a shade lighter than the window with a thin border, empty checkboxes and radio buttons carry a strong
//! outline, and the Windows accent blue marks what is ticked or chosen. Text selection keeps egui's colours, which keep
//! selected text readable. The window margin set here is the settings window's alone; the other windows keep their
//! panel's own margin.

use cuakey_application::ThemeValue;
use egui::{Color32, CornerRadius, Margin, Stroke, Style, Vec2, Visuals, vec2};

/// The size of every button in the settings window's bottom bar: one size, large enough to hit at a glance. A label
/// too wide for it widens every bar button alike.
pub const BUTTON_SIZE: Vec2 = vec2(110.0, 32.0);

/// The room between a window's edge and what it shows: less at the top and bottom, so the rows and the bar keep the
/// window short.
const WINDOW_MARGIN: Margin = Margin::symmetric(16, 12);

/// The room between one row of controls and the next.
const ROW_SPACING: f32 = 7.0;

/// The room between a button's frame and its text.
const BUTTON_PADDING: Vec2 = vec2(12.0, 6.0);

/// The least height of a control: a checkbox or radio row is this high, a button or combo box as high as its text and
/// padding need when that is more.
const CONTROL_HEIGHT: f32 = 24.0;

/// The corners of every control, whatever its state, as Windows 11 rounds them.
const CONTROL_CORNERS: CornerRadius = CornerRadius::same(4);

/// The corners of a window, as Windows 11 rounds them.
const WINDOW_CORNERS: CornerRadius = CornerRadius::same(8);

/// The colours that show a control's state rather than its surface.
#[derive(Debug)]
pub(crate) struct Accents {
    /// The outline of an empty checkbox or radio button, strong enough to see at a glance.
    pub(crate) strong_stroke: Color32,
    /// What a ticked checkbox or a chosen radio button is filled with: Windows' default accent blue.
    pub(crate) accent: Color32,
    /// The tick or dot drawn on the accent.
    pub(crate) on_accent: Color32,
}

/// The colours of one theme, as Windows 11 draws them over its window background.
struct Palette {
    /// Behind everything a window shows.
    window: Color32,
    /// Inside a text field.
    field: Color32,
    /// Every other row of a striped list.
    faint: Color32,
    /// A button, a combo box or an empty box at rest, under the pointer, and pressed.
    control: Color32,
    control_hovered: Color32,
    control_pressed: Color32,
    /// The thin border of a control at rest, and the lines that separate parts of a window.
    control_border: Color32,
    divider: Color32,
    text: Color32,
    accents: Accents,
}

/// Windows 11 dark. The window takes the colour Windows draws a card in, a step up from its darkest background, since
/// a small window like this is one card.
const DARK: Palette = Palette {
    window: Color32::from_rgb(43, 43, 43),
    field: Color32::from_rgb(31, 31, 31),
    faint: Color32::from_rgb(48, 48, 48),
    control: Color32::from_rgb(55, 55, 55),
    control_hovered: Color32::from_rgb(60, 60, 60),
    control_pressed: Color32::from_rgb(50, 50, 50),
    control_border: Color32::from_rgb(69, 69, 69),
    divider: Color32::from_rgb(60, 60, 60),
    text: Color32::WHITE,
    accents: Accents {
        strong_stroke: Color32::from_rgb(154, 154, 154),
        accent: Color32::from_rgb(96, 205, 255),
        on_accent: Color32::BLACK,
    },
};

/// Windows 11 light: a light grey window with white controls on it.
const LIGHT: Palette = Palette {
    window: Color32::from_rgb(243, 243, 243),
    field: Color32::WHITE,
    faint: Color32::from_rgb(238, 238, 238),
    control: Color32::from_rgb(251, 251, 251),
    control_hovered: Color32::from_rgb(246, 246, 246),
    control_pressed: Color32::from_rgb(245, 245, 245),
    control_border: Color32::from_rgb(229, 229, 229),
    divider: Color32::from_rgb(229, 229, 229),
    text: Color32::from_rgb(26, 26, 26),
    accents: Accents {
        strong_stroke: Color32::from_rgb(134, 134, 134),
        accent: Color32::from_rgb(0, 95, 184),
        on_accent: Color32::WHITE,
    },
};

/// Sets the shared look on `ctx`, for every window it draws: `theme`, the room and corners in the dark and the light
/// style alike, and each style's Windows 11 colours.
pub fn apply(ctx: &egui::Context, theme: ThemeValue) {
    set_theme(ctx, theme);
    ctx.all_styles_mut(set_room_and_corners);
    ctx.style_mut_of(egui::Theme::Dark, |style| paint(&mut style.visuals, &DARK));
    ctx.style_mut_of(egui::Theme::Light, |style| paint(&mut style.visuals, &LIGHT));
}

/// The accents of the theme `visuals` belongs to.
pub(crate) const fn accents(visuals: &Visuals) -> &'static Accents {
    if visuals.dark_mode {
        &DARK.accents
    } else {
        &LIGHT.accents
    }
}

/// Gives `visuals` the colours of `palette`. A control's border turns strong under the pointer and takes the text
/// colour when pressed or focused, so the keyboard focus shows as plainly as Windows' focus ring.
fn paint(visuals: &mut Visuals, palette: &Palette) {
    visuals.panel_fill = palette.window;
    visuals.window_fill = palette.window;
    visuals.extreme_bg_color = palette.field;
    visuals.faint_bg_color = palette.faint;
    visuals.hyperlink_color = palette.accents.accent;
    let widgets = &mut visuals.widgets;
    widgets.noninteractive.bg_fill = palette.window;
    widgets.noninteractive.weak_bg_fill = palette.window;
    widgets.noninteractive.bg_stroke.color = palette.divider;
    widgets.noninteractive.fg_stroke.color = palette.text;
    let strong = palette.accents.strong_stroke;
    for (state, fill, border) in [
        (&mut widgets.inactive, palette.control, palette.control_border),
        (&mut widgets.hovered, palette.control_hovered, strong),
        (&mut widgets.active, palette.control_pressed, palette.text),
        (&mut widgets.open, palette.control_hovered, strong),
    ] {
        state.bg_fill = fill;
        state.weak_bg_fill = fill;
        state.bg_stroke = Stroke::new(1.0, border);
        state.fg_stroke.color = palette.text;
    }
}

/// Draws every window of `ctx` in `theme` from the next pass on. A window may call this on every pass: it changes
/// nothing when `ctx` already draws in `theme`.
pub fn set_theme(ctx: &egui::Context, theme: ThemeValue) {
    let wanted = match theme {
        ThemeValue::Light => egui::ThemePreference::Light,
        ThemeValue::Dark => egui::ThemePreference::Dark,
    };
    if ctx.options(|options| options.theme_preference) != wanted {
        ctx.set_theme(wanted);
    }
}

fn set_room_and_corners(style: &mut Style) {
    let spacing = &mut style.spacing;
    spacing.window_margin = WINDOW_MARGIN;
    spacing.item_spacing.y = ROW_SPACING;
    spacing.button_padding = BUTTON_PADDING;
    spacing.interact_size.y = CONTROL_HEIGHT;

    let visuals = &mut style.visuals;
    let widgets = &mut visuals.widgets;
    for state in [
        &mut widgets.noninteractive,
        &mut widgets.inactive,
        &mut widgets.hovered,
        &mut widgets.active,
        &mut widgets.open,
    ] {
        state.corner_radius = CONTROL_CORNERS;
    }
    visuals.window_corner_radius = WINDOW_CORNERS;
}

#[cfg(test)]
mod tests;
