use egui::epaint::TextShape;
use egui::{Context, Event, Key, Modifiers, PointerButton, Pos2, RawInput, Rect, Response, Shape};

use super::{CHECKBOX_GAP, CHECKBOX_SIZE, checkbox};

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
fn the_box(shapes: &[Shape]) -> Rect {
    shapes
        .iter()
        .filter_map(|shape| {
            if let Shape::Rect(rect) = shape {
                Some(rect.rect)
            } else {
                None
            }
        })
        .min_by(|a, b| a.area().total_cmp(&b.area()))
        .unwrap()
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
