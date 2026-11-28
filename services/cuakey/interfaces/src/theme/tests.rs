use cuakey_application::ThemeValue;
use egui::{Color32, Context, CornerRadius, Margin, SystemTheme, Theme, ViewportCommand, Visuals, vec2};

use super::{accents, apply, set_theme};

/// The corner radius of every control, as Windows 11 rounds them.
const CONTROL_CORNERS: CornerRadius = CornerRadius::same(4);

/// The corner radius of a window, as Windows 11 rounds them.
const WINDOW_CORNERS: CornerRadius = CornerRadius::same(8);

/// The least contrast between text and what it is drawn on that WCAG level AA asks of body text.
const READABLE: f32 = 4.5;

/// The least contrast WCAG asks of what shows a control and its state, such as a checkbox's outline.
const VISIBLE: f32 = 3.0;

/// Every corner `visuals` rounds: the widget states, then the window.
fn corners_of(visuals: &Visuals) -> [CornerRadius; 6] {
    let widgets = &visuals.widgets;
    [
        widgets.noninteractive.corner_radius,
        widgets.inactive.corner_radius,
        widgets.hovered.corner_radius,
        widgets.active.corner_radius,
        widgets.open.corner_radius,
        visuals.window_corner_radius,
    ]
}

/// The relative luminance of `color`, as WCAG defines it.
fn luminance(color: Color32) -> f32 {
    let channel = |value: u8| {
        let value = f32::from(value) / 255.0;
        if value <= 0.040_45 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(color.r()) + 0.7152 * channel(color.g()) + 0.0722 * channel(color.b())
}

/// The WCAG contrast ratio of `text` on `background`.
fn contrast(text: Color32, background: Color32) -> f32 {
    let (a, b) = (luminance(text), luminance(background));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

/// Whether `color` is a neutral grey, as Windows 11's are.
fn is_neutral(color: Color32) -> bool {
    color.r() == color.g() && color.g() == color.b()
}

/// The visuals of `theme` after the shared look is applied.
fn themed(theme: Theme) -> Visuals {
    let ctx = Context::default();
    apply(&ctx, ThemeValue::Dark);
    ctx.style_of(theme).visuals.clone()
}

/// One pass of `ctx`, with nothing drawn, on a system that uses `system`.
fn pass(ctx: &Context, system: Theme) -> egui::FullOutput {
    let input = egui::RawInput {
        system_theme: Some(system),
        ..egui::RawInput::default()
    };
    let mut output = ctx.run_ui(input, |_| {});
    // No renderer takes the font texture here.
    output.textures_delta.clear();
    output
}

/// The theme `ctx` draws in after one pass on a system that uses `system`.
fn drawn_theme(ctx: &Context, system: Theme) -> Theme {
    drop(pass(ctx, system));
    ctx.theme()
}

/// The title bar themes one pass asked the root window for.
fn title_bar_requests(output: &egui::FullOutput) -> Vec<SystemTheme> {
    output
        .viewport_output
        .get(&egui::ViewportId::ROOT)
        .map(|viewport| {
            viewport
                .commands
                .iter()
                .filter_map(|command| {
                    if let ViewportCommand::SetTheme(theme) = command {
                        Some(*theme)
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn theme_rows_are_tighter() {
    let ctx = Context::default();
    let before = ctx.style_of(Theme::Dark);
    apply(&ctx, ThemeValue::Dark);
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
fn controls_and_windows_are_rounded_as_windows_11_rounds_them() {
    for theme in [Theme::Dark, Theme::Light] {
        let visuals = themed(theme);

        assert_eq!(
            corners_of(&visuals),
            [
                CONTROL_CORNERS,
                CONTROL_CORNERS,
                CONTROL_CORNERS,
                CONTROL_CORNERS,
                CONTROL_CORNERS,
                WINDOW_CORNERS
            ],
            "{theme:?}"
        );
    }
}

#[test]
fn theme_pads_buttons() {
    let ctx = Context::default();
    apply(&ctx, ThemeValue::Dark);

    assert_eq!(ctx.style_of(Theme::Dark).spacing.button_padding, vec2(12.0, 6.0));
}

#[test]
fn the_dark_theme_is_windows_11_dark() {
    let visuals = themed(Theme::Dark);

    assert_eq!(
        visuals.panel_fill,
        Color32::from_rgb(43, 43, 43),
        "the colour Windows 11 draws a card in"
    );
    assert_eq!(visuals.window_fill, visuals.panel_fill, "every window alike");
    assert_eq!(accents(&visuals).accent, Color32::from_rgb(96, 205, 255));
}

#[test]
fn the_light_theme_is_windows_11_light() {
    let visuals = themed(Theme::Light);

    assert_eq!(
        visuals.panel_fill,
        Color32::from_rgb(243, 243, 243),
        "the colour Windows 11 draws a window in"
    );
    assert_eq!(visuals.window_fill, visuals.panel_fill, "every window alike");
    assert_eq!(accents(&visuals).accent, Color32::from_rgb(0, 95, 184));
}

#[test]
fn the_greys_are_neutral_and_controls_sit_lighter_than_the_window() {
    for theme in [Theme::Dark, Theme::Light] {
        let visuals = themed(theme);
        let control = visuals.widgets.inactive.weak_bg_fill;

        assert!(is_neutral(visuals.panel_fill), "{theme:?}: {:?}", visuals.panel_fill);
        assert!(is_neutral(control), "{theme:?}: {control:?}");
        assert!(
            luminance(control) > luminance(visuals.panel_fill),
            "{theme:?}: a control is lighter than the window, as in Windows 11"
        );
        assert!(
            visuals.widgets.inactive.bg_stroke.width > 0.0,
            "{theme:?}: a control has a thin border"
        );
    }
}

#[test]
fn text_stays_readable_on_every_background() {
    for theme in [Theme::Dark, Theme::Light] {
        let visuals = themed(theme);
        let (labels, buttons) = (&visuals.widgets.noninteractive, &visuals.widgets.inactive);

        let on_panel = contrast(labels.fg_stroke.color, visuals.panel_fill);
        let on_button = contrast(buttons.fg_stroke.color, buttons.weak_bg_fill);
        let in_field = contrast(buttons.fg_stroke.color, visuals.extreme_bg_color);
        let links = contrast(visuals.hyperlink_color, visuals.panel_fill);

        assert!(on_panel >= READABLE, "{theme:?}: labels at {on_panel:.2}");
        assert!(on_button >= READABLE, "{theme:?}: button text at {on_button:.2}");
        assert!(in_field >= READABLE, "{theme:?}: typed text at {in_field:.2}");
        assert!(links >= READABLE, "{theme:?}: links at {links:.2}");
    }
}

#[test]
fn boxes_radios_and_the_accent_stand_out_from_the_window() {
    for theme in [Theme::Dark, Theme::Light] {
        let visuals = themed(theme);
        let accents = accents(&visuals);

        let outline = contrast(accents.strong_stroke, visuals.panel_fill);
        let accent = contrast(accents.accent, visuals.panel_fill);
        let tick = contrast(accents.on_accent, accents.accent);

        assert!(outline >= VISIBLE, "{theme:?}: an empty box's outline at {outline:.2}");
        assert!(accent >= VISIBLE, "{theme:?}: a ticked box at {accent:.2}");
        assert!(tick >= READABLE, "{theme:?}: the tick at {tick:.2}");
    }
}

#[test]
fn the_windows_are_light_when_light_is_chosen_on_a_dark_system() {
    let ctx = Context::default();

    apply(&ctx, ThemeValue::Light);

    assert_eq!(drawn_theme(&ctx, Theme::Dark), Theme::Light);
}

#[test]
fn the_windows_are_dark_when_dark_is_chosen_on_a_light_system() {
    let ctx = Context::default();

    apply(&ctx, ThemeValue::Dark);

    assert_eq!(drawn_theme(&ctx, Theme::Light), Theme::Dark);
}

#[test]
fn a_new_choice_changes_the_theme_and_keeps_the_room() {
    let ctx = Context::default();
    apply(&ctx, ThemeValue::Light);

    set_theme(&ctx, ThemeValue::Dark);

    assert_eq!(drawn_theme(&ctx, Theme::Light), Theme::Dark);
    assert_eq!(ctx.style_of(Theme::Dark).spacing.button_padding, vec2(12.0, 6.0));
}

#[test]
fn the_light_theme_gets_the_same_room() {
    let ctx = Context::default();
    apply(&ctx, ThemeValue::Light);
    let dark = ctx.style_of(Theme::Dark);
    let light = ctx.style_of(Theme::Light);
    assert_eq!(light.spacing, dark.spacing);
}

/// egui keeps the native title bar in the chosen theme itself; this guards against losing that.
#[test]
fn the_title_bar_takes_the_windows_theme_once_each_time_it_changes() {
    let ctx = Context::default();
    apply(&ctx, ThemeValue::Dark);

    let first = pass(&ctx, Theme::Light);
    let second = pass(&ctx, Theme::Light);
    set_theme(&ctx, ThemeValue::Light);
    let after_the_change = pass(&ctx, Theme::Light);

    assert_eq!(
        title_bar_requests(&first),
        [SystemTheme::Dark],
        "dark on a light system, so Windows is told"
    );
    assert_eq!(title_bar_requests(&second), [], "told once, not on every pass");
    assert_eq!(title_bar_requests(&after_the_change), [SystemTheme::Light]);
}
