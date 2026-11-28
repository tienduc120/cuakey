use egui::epaint::{CircleShape, ColorMode, RectShape, TextShape};
use egui::{Context, Event, Key, Modifiers, PointerButton, Pos2, RawInput, Rect, Response, Shape, TextStyle};

use super::{CHECKBOX_GAP, CHECKBOX_SIZE, RADIO_SIZE, checkbox, radio, text_width, widest_in_both_languages};
use crate::labels::{ENGLISH, Labels, VIETNAMESE};
use crate::theme::{Accents, accents};

const LABEL: &str = "Kêu Beep";

fn flatten(shapes: Vec<Shape>) -> Vec<Shape> {
    shapes
        .into_iter()
        .flat_map(|shape| {
            if let Shape::Vec(inner) = shape {
                flatten(inner)
            } else {
                vec![shape]
            }
        })
        .collect()
}

/// Draws one checkbox in a pass fed `events`, and returns its response and what the pass painted.
fn draw(ctx: &Context, on: bool, events: Vec<Event>) -> (Response, Vec<Shape>) {
    let input = RawInput {
        events,
        ..RawInput::default()
    };
    let mut response = None;
    let mut output = ctx.run_ui(input, |ui| response = Some(checkbox(ui, LABEL, on)));
    // No renderer takes the font texture here.
    output.textures_delta.clear();
    let shapes = output.shapes.into_iter().map(|clipped| clipped.shape).collect();
    (response.unwrap(), flatten(shapes))
}

/// The smallest rectangle painted: the box, not a frame around the whole widget.
fn the_box_shape(shapes: &[Shape]) -> RectShape {
    shapes
        .iter()
        .filter_map(|shape| {
            if let Shape::Rect(rect) = shape {
                Some(rect.clone())
            } else {
                None
            }
        })
        .min_by(|a, b| a.rect.area().total_cmp(&b.rect.area()))
        .unwrap()
}

fn the_box(shapes: &[Shape]) -> Rect {
    the_box_shape(shapes).rect
}

fn the_label(shapes: &[Shape]) -> &TextShape {
    shapes
        .iter()
        .find_map(|shape| {
            if let Shape::Text(text) = shape {
                Some(text)
            } else {
                None
            }
        })
        .unwrap()
}

fn primary(pos: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}

/// Presses and releases the primary button at `pos`, and returns the checkbox's response to the release.
fn click(ctx: &Context, on: bool, pos: Pos2) -> Response {
    drop(draw(ctx, on, vec![Event::PointerMoved(pos), primary(pos, true)]));
    draw(ctx, on, vec![primary(pos, false)]).0
}

fn is_tick(shape: &Shape) -> bool {
    matches!(shape, Shape::Path(_) | Shape::LineSegment { .. })
}

#[test]
fn checkbox_geometry() {
    let ctx = Context::default();

    let (response, shapes) = draw(&ctx, false, Vec::new());
    let icon = the_box(&shapes);
    let label = the_label(&shapes);
    assert_eq!(
        icon.size(),
        egui::Vec2::splat(CHECKBOX_SIZE),
        "the box is {CHECKBOX_SIZE} px square"
    );
    assert_eq!(label.galley.text(), LABEL);
    let gap = label.pos.x - icon.right();
    assert!(
        (gap - CHECKBOX_GAP).abs() < 0.01,
        "the label starts {gap} px after the box"
    );

    let clicked = click(&ctx, false, response.rect.center());
    assert!(clicked.clicked(), "the click lands on the checkbox");
    assert!(clicked.changed(), "a click marks the checkbox changed");
}

#[test]
fn drawing_without_input_changes_nothing() {
    let ctx = Context::default();
    let (response, _) = draw(&ctx, true, Vec::new());
    assert!(!response.changed());
}

#[test]
fn the_tick_shows_only_when_on() {
    let ctx = Context::default();
    let (_, off) = draw(&ctx, false, Vec::new());
    let (_, on) = draw(&ctx, true, Vec::new());
    assert!(!off.iter().any(is_tick), "an unticked box draws no tick: {off:?}");
    assert!(on.iter().any(is_tick), "a ticked box draws its tick: {on:?}");
}

/// The tick's stroke width, from what a pass painted.
fn tick_width(shapes: &[Shape]) -> f32 {
    shapes
        .iter()
        .find_map(|shape| {
            if let Shape::Path(path) = shape {
                Some(path.stroke.width)
            } else {
                None
            }
        })
        .unwrap()
}

#[test]
fn the_tick_grows_with_the_box() {
    let ctx = Context::default();
    let (_, on) = draw(&ctx, true, Vec::new());
    let width = tick_width(&on);
    assert!(
        (width - CHECKBOX_SIZE * 2.0 / 15.0).abs() < 0.01,
        "a 2 px tick in the {CHECKBOX_SIZE} px box, not a hairline: {width}"
    );
}

/// The painted box of a ticked checkbox at rest, under the pointer, and holding the keyboard focus.
fn ticked_box_states() -> [RectShape; 3] {
    let ctx = Context::default();
    let (response, shapes) = draw(&ctx, true, Vec::new());
    let at_rest = the_box_shape(&shapes);

    let over = response.rect.center();
    drop(draw(&ctx, true, vec![Event::PointerMoved(over)]));
    let (_, shapes) = draw(&ctx, true, vec![Event::PointerMoved(over)]);
    let hovered = the_box_shape(&shapes);

    let mut output = ctx.run_ui(
        RawInput {
            events: vec![Event::PointerGone],
            ..RawInput::default()
        },
        |ui| checkbox(ui, LABEL, true).request_focus(),
    );
    output.textures_delta.clear();
    let (_, shapes) = draw(&ctx, true, Vec::new());
    let focused = the_box_shape(&shapes);
    [at_rest, hovered, focused]
}

#[test]
fn a_ticked_box_shows_hover_and_focus() {
    let [at_rest, hovered, focused] = ticked_box_states();
    assert_eq!(hovered.fill, at_rest.fill, "the box stays ticked under the pointer");
    assert_ne!(hovered.stroke, at_rest.stroke, "the pointer outlines the box");
    assert_ne!(focused.stroke, at_rest.stroke, "the focus outlines the box");
    assert_ne!(focused.stroke, hovered.stroke, "focus reads apart from hover");
}

#[test]
fn the_widest_text_of_either_language_sets_the_width() {
    let mut output = Context::default().run_ui(RawInput::default(), |ui| {
        let width = |text| text_width(ui, text, TextStyle::Button);
        let longer_in_vietnamese: fn(&Labels) -> &'static str = |text| text.button_reset;
        let longer_in_english: fn(&Labels) -> &'static str = |text| text.about_name;
        assert!(width(longer_in_vietnamese(&VIETNAMESE)) > width(longer_in_vietnamese(&ENGLISH)));
        assert!(width(longer_in_english(&ENGLISH)) > width(longer_in_english(&VIETNAMESE)));

        for (pick, widest) in [
            (longer_in_vietnamese, longer_in_vietnamese(&VIETNAMESE)),
            (longer_in_english, longer_in_english(&ENGLISH)),
        ] {
            let measured = widest_in_both_languages(ui, &TextStyle::Button, |text| [pick(text), text.button_close]);
            assert!(
                (measured - width(widest)).abs() < 0.01,
                "{measured} is the width of {widest}, {}",
                width(widest)
            );
        }
    });
    output.textures_delta.clear();
}

/// The accents of the theme a default context draws in.
fn default_accents() -> &'static Accents {
    accents(&Context::default().global_style().visuals)
}

#[test]
fn an_empty_box_has_a_strong_outline_at_rest() {
    let (_, shapes) = draw(&Context::default(), false, Vec::new());

    let outline = the_box_shape(&shapes).stroke;

    assert_eq!(
        outline.color,
        default_accents().strong_stroke,
        "seen at a glance on the window"
    );
    assert!(outline.width >= 1.0, "{outline:?}");
}

#[test]
fn a_ticked_box_is_filled_with_the_accent_and_ticked_on_it() {
    let (_, shapes) = draw(&Context::default(), true, Vec::new());

    let fill = the_box_shape(&shapes).fill;
    let tick = shapes.iter().find_map(|shape| {
        if let Shape::Path(path) = shape {
            Some(path.stroke.color.clone())
        } else {
            None
        }
    });

    assert_eq!(fill, default_accents().accent);
    assert_eq!(tick, Some(ColorMode::Solid(default_accents().on_accent)));
}

const CHOICE: &str = "Tiếng Việt";

/// Draws one radio button in a pass fed `events`, and returns its response and what the pass painted.
fn draw_radio(ctx: &Context, chosen: bool, events: Vec<Event>) -> (Response, Vec<Shape>) {
    let input = RawInput {
        events,
        ..RawInput::default()
    };
    let mut response = None;
    let mut output = ctx.run_ui(input, |ui| response = Some(radio(ui, CHOICE, chosen)));
    output.textures_delta.clear();
    let shapes = output.shapes.into_iter().map(|clipped| clipped.shape).collect();
    (response.unwrap(), flatten(shapes))
}

/// The circles a pass painted, largest first: the radio's ring, then its dot when it has one.
fn circles(shapes: &[Shape]) -> Vec<CircleShape> {
    let mut circles: Vec<CircleShape> = shapes
        .iter()
        .filter_map(|shape| {
            if let Shape::Circle(circle) = shape {
                Some(*circle)
            } else {
                None
            }
        })
        .collect();
    circles.sort_by(|a, b| b.radius.total_cmp(&a.radius));
    circles
}

#[test]
fn a_radio_is_a_circle_as_large_as_a_checkbox_beside_its_label() {
    let ctx = Context::default();

    let (response, shapes) = draw_radio(&ctx, false, Vec::new());
    let ring = *circles(&shapes).first().expect("the radio draws its circle");
    let label = the_label(&shapes);

    assert!((ring.radius - RADIO_SIZE / 2.0).abs() < 0.01, "{ring:?}");
    assert_eq!(label.galley.text(), CHOICE);
    let gap = label.pos.x - (ring.center.x + ring.radius);
    assert!(
        (gap - CHECKBOX_GAP).abs() < 0.01,
        "the label starts {gap} px after the circle"
    );
    let at = response.rect.center();
    drop(draw_radio(
        &ctx,
        false,
        vec![Event::PointerMoved(at), primary(at, true)],
    ));
    let (released, _) = draw_radio(&ctx, false, vec![primary(at, false)]);
    assert!(released.clicked(), "the click lands on the radio");
}

#[test]
fn an_unchosen_radio_has_a_strong_outline_and_no_dot() {
    let (_, shapes) = draw_radio(&Context::default(), false, Vec::new());

    let rings = circles(&shapes);

    assert_eq!(rings.len(), 1, "{rings:?}");
    assert_eq!(rings[0].stroke.color, default_accents().strong_stroke);
}

#[test]
fn a_chosen_radio_is_filled_with_the_accent_and_shows_its_dot() {
    let (_, shapes) = draw_radio(&Context::default(), true, Vec::new());

    let rings = circles(&shapes);

    assert_eq!(rings.len(), 2, "{rings:?}");
    assert_eq!(rings[0].fill, default_accents().accent);
    assert_eq!(rings[1].fill, default_accents().on_accent);
    assert!(rings[1].radius < rings[0].radius);
}

#[test]
fn space_chooses_a_focused_radio() {
    let ctx = Context::default();
    let mut output = ctx.run_ui(RawInput::default(), |ui| radio(ui, CHOICE, false).request_focus());
    output.textures_delta.clear();
    let space = Event::Key {
        key: Key::Space,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    };

    let (response, _) = draw_radio(&ctx, false, vec![space]);

    assert!(response.clicked(), "Space picks the radio that has focus");
}

#[test]
fn space_changes_a_focused_checkbox() {
    let ctx = Context::default();
    let mut output = ctx.run_ui(RawInput::default(), |ui| checkbox(ui, LABEL, false).request_focus());
    output.textures_delta.clear();

    let space = Event::Key {
        key: Key::Space,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    };
    let (response, _) = draw(&ctx, false, vec![space]);
    assert!(response.changed(), "Space toggles the checkbox that has focus");
}
