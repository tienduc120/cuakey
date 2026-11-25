//! The app's own widgets, drawn in the colours of the current egui theme.

use egui::{Pos2, Rect, Response, Sense, Shape, StrokeKind, TextStyle, Ui, Vec2, WidgetInfo, WidgetText, pos2, vec2};

/// The side of a checkbox's box: one step larger than egui's, so the box reads at a glance.
pub(crate) const CHECKBOX_SIZE: f32 = 15.0;

/// The space between a checkbox's box and its label.
pub(crate) const CHECKBOX_GAP: f32 = 5.0;

/// The tick's corners as fractions of the box, so the tick grows with it.
const TICK: [Vec2; 3] = [vec2(0.22, 0.52), vec2(0.42, 0.72), vec2(0.78, 0.30)];

fn tick(icon: Rect) -> Vec<Pos2> {
    TICK.iter().map(|corner| icon.min + *corner * icon.size()).collect()
}

/// A checkbox that shows `on`. A click, or Space while it has focus, marks the response changed; the caller keeps the
/// value and flips it.
pub(crate) fn checkbox(ui: &mut Ui, label: &str, on: bool) -> Response {
    let text_room = ui.available_width() - CHECKBOX_SIZE - CHECKBOX_GAP;
    let galley = WidgetText::from(label).into_galley(ui, None, text_room, TextStyle::Body);
    let height = galley.size().y.max(CHECKBOX_SIZE).max(ui.spacing().interact_size.y);
    let size = vec2(CHECKBOX_SIZE + CHECKBOX_GAP + galley.size().x, height);
    let (rect, mut response) = ui.allocate_exact_size(size, Sense::click());
    if response.clicked() {
        response.mark_changed();
    }
    response.widget_info(|| WidgetInfo::selected(egui::Role::CheckBox, ui.is_enabled(), on, label));
    if !ui.is_rect_visible(rect) {
        return response;
    }
    let icon = Rect::from_min_size(
        pos2(rect.left(), rect.center().y - CHECKBOX_SIZE / 2.0),
        Vec2::splat(CHECKBOX_SIZE),
    );
    // A ticked box takes the theme's selection colours; the label keeps the plain widget colour either way.
    let boxed = ui.style().interact_selectable(&response, on);
    let text_color = ui.style().interact(&response).text_color();
    let painter = ui.painter();
    painter.rect(
        icon,
        boxed.corner_radius,
        boxed.bg_fill,
        boxed.bg_stroke,
        StrokeKind::Inside,
    );
    if on {
        painter.add(Shape::line(tick(icon), boxed.fg_stroke));
    }
    let text_pos = pos2(icon.right() + CHECKBOX_GAP, rect.center().y - galley.size().y / 2.0);
    painter.galley(text_pos, galley, text_color);
    response
}

#[cfg(test)]
mod tests;
