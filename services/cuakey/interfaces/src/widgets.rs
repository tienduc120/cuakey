//! The app's own widgets, drawn in the colours of the current egui theme.

use std::sync::Arc;

use egui::style::WidgetVisuals;
use egui::{
    Color32, Galley, Pos2, Rect, Response, Sense, Shape, Stroke, StrokeKind, TextStyle, TextWrapMode, Ui, Vec2,
    WidgetInfo, WidgetText, pos2, vec2,
};

use crate::labels::{EVERY_LANGUAGE, Labels};
use crate::theme::{self, Accents};

/// The side of a checkbox's box: one step larger than egui's, so the box reads at a glance.
pub(crate) const CHECKBOX_SIZE: f32 = 15.0;

/// The space between a checkbox's box and its label.
pub(crate) const CHECKBOX_GAP: f32 = 5.0;

/// The tick's corners as fractions of the box, so the tick grows with it.
const TICK: [Vec2; 3] = [vec2(0.22, 0.52), vec2(0.42, 0.72), vec2(0.78, 0.30)];

/// The tick's stroke width as a fraction of the box's side: 2 points in a 15 point box, firm enough to read at a
/// glance without filling the box.
const TICK_WIDTH: f32 = 2.0 / 15.0;

/// The diameter of a radio button's circle: a checkbox's side, so the two line up in a column.
pub(crate) const RADIO_SIZE: f32 = CHECKBOX_SIZE;

/// The radius of a chosen radio button's dot as a fraction of its circle's.
const RADIO_DOT: f32 = 0.4;

/// A radio button that shows `chosen`. A click, or Space while it has focus, makes the response clicked; the caller
/// keeps the choice.
pub(crate) fn radio(ui: &mut Ui, label: &str, chosen: bool) -> Response {
    let row = mark_row(ui, label, RADIO_SIZE);
    let response = row.response;
    response.widget_info(|| WidgetInfo::selected(egui::Role::RadioButton, ui.is_enabled(), chosen, label));
    if !ui.is_rect_visible(response.rect) {
        return response;
    }
    let state = ui.style().interact(&response);
    let accents = theme::accents(ui.visuals());
    let (fill, outline) = mark_look(&response, state, accents, chosen);
    let (center, radius) = (row.mark.center(), RADIO_SIZE / 2.0 + state.expansion);
    let painter = ui.painter();
    painter.circle(center, radius, fill, outline);
    if chosen {
        painter.circle_filled(center, radius * RADIO_DOT, accents.on_accent);
    }
    painter.galley(row.text_pos, row.galley, state.text_color());
    response
}

/// A checkbox's box or a radio button's circle with its label after it, laid out and allocated in `ui`.
struct MarkRow {
    response: Response,
    /// The square the box or circle fills.
    mark: Rect,
    galley: Arc<Galley>,
    text_pos: Pos2,
}

/// Lays out a mark of side `side` and `label` after it, at least a control's height, and allocates the row. A click,
/// or Space while it has focus, marks the response changed.
fn mark_row(ui: &mut Ui, label: &str, side: f32) -> MarkRow {
    let text_room = ui.available_width() - side - CHECKBOX_GAP;
    let galley = WidgetText::from(label).into_galley(ui, None, text_room, TextStyle::Body);
    let height = galley.size().y.max(side).max(ui.spacing().interact_size.y);
    let row_size = vec2(side + CHECKBOX_GAP + galley.size().x, height);
    let (rect, mut response) = ui.allocate_exact_size(row_size, Sense::click());
    if response.clicked() {
        response.mark_changed();
    }
    let mark = Rect::from_min_size(pos2(rect.left(), rect.center().y - side / 2.0), Vec2::splat(side));
    let text_pos = pos2(mark.right() + CHECKBOX_GAP, rect.center().y - galley.size().y / 2.0);
    MarkRow {
        response,
        mark,
        galley,
        text_pos,
    }
}

/// The fill and outline of a box or circle that shows `on`: the accent when on, the control's own fill when off. At
/// rest an empty one carries the strong outline, as Windows 11 draws it, and a filled one none; under the pointer or
/// the keyboard focus the state's outline shows on either.
fn mark_look(response: &Response, state: &WidgetVisuals, accents: &Accents, on: bool) -> (Color32, Stroke) {
    let at_rest = !(response.hovered() || response.has_focus() || response.is_pointer_button_down_on());
    let fill = if on { accents.accent } else { state.bg_fill };
    let outline = match (at_rest, on) {
        (true, true) => Stroke::NONE,
        (true, false) => Stroke::new(1.0, accents.strong_stroke),
        (false, _) => state.bg_stroke,
    };
    (fill, outline)
}

/// The picture on the theme button, drawn rather than taken from a font, since the system font has neither.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ThemeIcon {
    Sun,
    Moon,
}

/// The icon's radius as a fraction of the button's side.
const ICON_RADIUS: f32 = 0.28;

/// A square button filling `rect`, laid over what is drawn there, with `icon` in the theme's text colour and `hint` on
/// hover and for screen readers.
pub(crate) fn icon_button(ui: &Ui, rect: Rect, icon: ThemeIcon, hint: &str) -> Response {
    let response = ui.interact(rect, ui.scope_id().with("icon_button"), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(egui::Role::Button, ui.is_enabled(), hint));
    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let painter = ui.painter();
        painter.rect(
            rect,
            visuals.corner_radius,
            visuals.weak_bg_fill,
            visuals.bg_stroke,
            StrokeKind::Inside,
        );
        let (center, radius, color) = (rect.center(), rect.height() * ICON_RADIUS, visuals.fg_stroke.color);
        match icon {
            ThemeIcon::Sun => {
                painter.circle_filled(center, radius * 0.55, color);
                for step in 0_u8..8 {
                    let ray = Vec2::angled(f32::from(step) * std::f32::consts::TAU / 8.0);
                    painter.line_segment(
                        [center + ray * radius * 0.85, center + ray * radius * 1.25],
                        Stroke::new(1.5, color),
                    );
                }
            }
            ThemeIcon::Moon => {
                // A disc with a bite taken out in the button's own colour leaves a crescent.
                painter.circle_filled(center, radius, color);
                painter.circle_filled(
                    center + vec2(radius * 0.45, -radius * 0.35),
                    radius * 0.85,
                    visuals.weak_bg_fill,
                );
            }
        }
    }
    response.on_hover_text(hint)
}

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
    let row = mark_row(ui, label, CHECKBOX_SIZE);
    let response = row.response;
    response.widget_info(|| WidgetInfo::selected(egui::Role::CheckBox, ui.is_enabled(), on, label));
    if !ui.is_rect_visible(response.rect) {
        return response;
    }
    // The size follows the widget's state, ticked or not, so hover and focus show on every box. A ticked box is
    // filled with the accent and ticked in the colour drawn on it; the label keeps the plain widget colour either way.
    let state = ui.style().interact(&response);
    let accents = theme::accents(ui.visuals());
    let (fill, outline) = mark_look(&response, state, accents, on);
    let icon = row.mark;
    let painter = ui.painter();
    painter.rect(
        icon.expand(state.expansion),
        state.corner_radius,
        fill,
        outline,
        StrokeKind::Inside,
    );
    if on {
        let stroke = Stroke::new(icon.width() * TICK_WIDTH, accents.on_accent);
        painter.add(Shape::line(tick(icon), stroke));
    }
    painter.galley(row.text_pos, row.galley, state.text_color());
    response
}

#[cfg(test)]
mod tests;
