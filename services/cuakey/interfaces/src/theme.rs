//! What every window of the app shares: egui's dark theme, whatever theme Windows uses, with more room between the
//! controls and rounder corners. Only spacing and corner radius change, so every colour stays the theme's own. The
//! wider window margin set here is the settings window's alone; the other windows keep their panel's own margin.

use egui::{CornerRadius, Margin, Style, Vec2, vec2};

/// The size of every button in the settings window's bottom bar: one size, large enough to hit at a glance. A label
/// too wide for it widens every bar button alike.
pub const BUTTON_SIZE: Vec2 = vec2(110.0, 32.0);

/// The room between a window's edge and what it shows.
const WINDOW_MARGIN: Margin = Margin::same(16);

/// The room between one row of controls and the next.
const ROW_SPACING: f32 = 10.0;

/// The room between a button's frame and its text.
const BUTTON_PADDING: Vec2 = vec2(12.0, 6.0);

/// The height of a control: a button, a combo box, a checkbox row.
const CONTROL_HEIGHT: f32 = 28.0;

/// One radius for the corners of every widget, whatever its state, and of every window.
const CORNERS: CornerRadius = CornerRadius::same(6);

/// Sets the shared look on `ctx`, for every window it draws: the dark theme, and the room and corners in the dark and
/// the light style alike.
pub fn apply(ctx: &egui::Context) {
    ctx.set_theme(egui::Theme::Dark);
    ctx.all_styles_mut(set_room_and_corners);
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
        state.corner_radius = CORNERS;
    }
    visuals.window_corner_radius = CORNERS;
}

#[cfg(test)]
mod tests;
