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
fn theme_sets_spacing_and_rounding() {
    let ctx = Context::default();
    let before = ctx.style_of(Theme::Dark);
    apply(&ctx);
    let style = ctx.style_of(Theme::Dark);

    let spacing = &style.spacing;
    assert_eq!(spacing.window_margin, Margin::same(16));
    assert_eq!(
        spacing.item_spacing,
        vec2(before.spacing.item_spacing.x, 10.0),
        "more room between rows, the same between controls on a row"
    );
    assert_eq!(spacing.button_padding, vec2(12.0, 6.0));
    assert_eq!(spacing.interact_size, vec2(before.spacing.interact_size.x, 28.0));

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
