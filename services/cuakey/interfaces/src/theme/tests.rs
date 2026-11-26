use egui::{Context, CornerRadius, Margin, Theme, Visuals, vec2};

use super::apply;

/// The corner radius every widget state and every window takes.
const CORNERS: CornerRadius = CornerRadius::same(6);

/// `visuals` with every corner the theme rounds set to `corners`.
fn with_corners(mut visuals: Visuals, corners: CornerRadius) -> Visuals {
    let widgets = &mut visuals.widgets;
    for state in [
        &mut widgets.noninteractive,
        &mut widgets.inactive,
        &mut widgets.hovered,
        &mut widgets.active,
        &mut widgets.open,
    ] {
        state.corner_radius = corners;
    }
    visuals.window_corner_radius = corners;
    visuals
}

#[test]
fn theme_rows_are_tighter() {
    let ctx = Context::default();
    let before = ctx.style_of(Theme::Dark);
    apply(&ctx);
    let spacing = &ctx.style_of(Theme::Dark).spacing;

    assert_eq!(
        spacing.item_spacing,
        vec2(before.spacing.item_spacing.x, 7.0),
        "rows 7 apart, the same room between controls on a row"
    );
    assert_eq!(
        spacing.interact_size,
        vec2(before.spacing.interact_size.x, 24.0),
        "controls 24 high"
    );
    assert_eq!(
        spacing.window_margin,
        Margin {
            left: 16,
            right: 16,
            top: 12,
            bottom: 12,
        },
        "16 at the sides, 12 at the top and bottom"
    );
}

#[test]
fn theme_pads_buttons_and_rounds_corners() {
    let ctx = Context::default();
    apply(&ctx);
    let style = ctx.style_of(Theme::Dark);

    assert_eq!(style.spacing.button_padding, vec2(12.0, 6.0));
    assert_eq!(
        style.visuals,
        with_corners(Visuals::dark(), CORNERS),
        "the dark theme's colours, with rounder corners"
    );
}

#[test]
fn the_windows_stay_dark_when_the_system_is_light() {
    let ctx = Context::default();
    apply(&ctx);
    let input = egui::RawInput {
        system_theme: Some(Theme::Light),
        ..egui::RawInput::default()
    };

    let mut output = ctx.run_ui(input, |_| {});
    // No renderer takes the font texture here.
    output.textures_delta.clear();

    assert_eq!(ctx.theme(), Theme::Dark);
}

#[test]
fn the_light_theme_gets_the_same_room() {
    let ctx = Context::default();
    apply(&ctx);
    let dark = ctx.style_of(Theme::Dark);
    let light = ctx.style_of(Theme::Light);
    assert_eq!(light.spacing, dark.spacing);
    assert_eq!(light.visuals, with_corners(Visuals::light(), CORNERS));
}
