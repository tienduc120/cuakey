//! The app's own widgets, drawn in the colours of the current egui theme.

use egui::{
    Pos2, Rect, Response, Sense, Shape, Stroke, StrokeKind, TextStyle, TextWrapMode, Ui, Vec2, WidgetInfo, WidgetText,
    pos2, vec2,
};

use crate::labels::{EVERY_LANGUAGE, Labels};

/// The side of a checkbox's box: one step larger than egui's, so the box reads at a glance.
pub(crate) const CHECKBOX_SIZE: f32 = 15.0;

/// The space between a checkbox's box and its label.
pub(crate) const CHECKBOX_GAP: f32 = 5.0;

/// The tick's corners as fractions of the box, so the tick grows with it.
const TICK: [Vec2; 3] = [vec2(0.22, 0.52), vec2(0.42, 0.72), vec2(0.78, 0.30)];

/// The tick's stroke width as a fraction of the box's side: 2 points in a 15 point box, firm enough to read at a
/// glance without filling the box.
const TICK_WIDTH: f32 = 2.0 / 15.0;

/// The width of `text` on one line, in `style` of the ui's current font.
pub(crate) fn text_width(ui: &Ui, text: &str, style: TextStyle) -> f32 {
    WidgetText::from(text)
        .into_galley(ui, Some(TextWrapMode::Extend), f32::INFINITY, style)
        .size()
        .x
}

/// The width of the widest text `texts` picks from the labels of either interface language, on one line in `style` of
/// the ui's current font. A place this wide holds its text in both languages, so what follows it stays where it is when
/// the language changes.
pub(crate) fn widest_in_both_languages<I>(ui: &Ui, style: &TextStyle, texts: impl Fn(&'static Labels) -> I) -> f32
where
    I: IntoIterator<Item = &'static str>,
{
    EVERY_LANGUAGE
        .into_iter()
        .flat_map(texts)
        .map(|text| text_width(ui, text, style.clone()))
        .fold(0.0, f32::max)
}

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
    // The outline and size follow the widget's state, ticked or not, so hover and focus show on every box. A ticked
    // box is filled with the theme's selection colour and ticked in its selected-text colour; the label keeps the
    // plain widget colour either way.
    let state = ui.style().interact(&response);
    let selection = ui.visuals().selection;
    let painter = ui.painter();
    painter.rect(
        icon.expand(state.expansion),
        state.corner_radius,
        if on { selection.bg_fill } else { state.bg_fill },
        state.bg_stroke,
        StrokeKind::Inside,
    );
    if on {
        let stroke = Stroke::new(icon.width() * TICK_WIDTH, selection.stroke.color);
        painter.add(Shape::line(tick(icon), stroke));
    }
    let text_pos = pos2(icon.right() + CHECKBOX_GAP, rect.center().y - galley.size().y / 2.0);
    painter.galley(text_pos, galley, state.text_color());
    response
}

#[cfg(test)]
mod tests;
