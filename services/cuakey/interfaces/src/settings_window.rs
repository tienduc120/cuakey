//! The settings window drawn with egui. It opens small, with the controls used every day, and the expand button in the
//! bottom bar shows the other sections. Every change applies at once: a frame that changed a control returns one
//! `Save` with the whole edited view. What is shown and enabled lives in [`SettingsForm`]; this file only draws it, in
//! the interface language of the view it edits.

use cuakey_application::{
    CodeTableValue, InputMethodValue, InterfaceLanguageValue, LanguageValue, PreferencesView, ThemeValue, Toggle,
};
use egui::{TextStyle, Ui};

use crate::hotkey_editor::show_hotkey;
use crate::labels::{Labels, labels_for};
use crate::screen_fit::{Placement, fit_on_screen};
use crate::settings_form::{ButtonBar, GatedControl, SettingsForm, is_on};
use crate::theme::{self, BUTTON_SIZE};
use crate::{Button, Platform, UiAction, widgets};

const fn input_methods(text: &Labels) -> [(InputMethodValue, &'static str); 3] {
    [
        (InputMethodValue::Telex, text.input_method_telex),
        (InputMethodValue::Vni, text.input_method_vni),
        (InputMethodValue::SimpleTelex, text.input_method_simple_telex),
    ]
}

pub(crate) const fn code_tables(text: &Labels) -> [(CodeTableValue, &'static str); 5] {
    [
        (CodeTableValue::Unicode, text.code_table_unicode),
        (CodeTableValue::Tcvn3, text.code_table_tcvn3),
        (CodeTableValue::VniWindows, text.code_table_vni_windows),
        (CodeTableValue::UnicodeCompound, text.code_table_unicode_compound),
        (CodeTableValue::Cp1258, text.code_table_cp1258),
    ]
}

const fn typing_languages(text: &Labels) -> [(LanguageValue, &'static str); 2] {
    [
        (LanguageValue::Vietnamese, text.language_vietnamese),
        (LanguageValue::English, text.language_english),
    ]
}

/// The interface languages, each named in itself whatever the language of `text`, so a user who cannot read the one
/// showing still finds their own.
const fn interface_languages(text: &Labels) -> [(InterfaceLanguageValue, &'static str); 2] {
    [
        (InterfaceLanguageValue::Vietnamese, text.interface_vietnamese),
        (InterfaceLanguageValue::English, text.interface_english),
    ]
}

/// The window's inner size with the controls used every day only, the size it opens at: its rows, the bottom bar and
/// the margins, with nothing to spare, as the shared style draws them in Segoe UI in either interface language. Each
/// frame measures the rows, so in Tahoma or Arial, which the window borrows when Segoe UI is missing, it is a few
/// points shorter, and a line for an error message makes it taller. The width is the expanded window's, so expanding
/// only moves the bottom edge and the bar's buttons stay where they were.
pub const COMPACT_SIZE: egui::Vec2 = egui::vec2(560.0, 267.0);

/// The window's inner size with every section, where the screen has room for it; a shorter screen gets a shorter
/// window. The options sit two to a row, and the widest needs 257 points in either interface language, so two columns
/// fit in this width. The sections are taller than a small screen, so they scroll above the bottom bar; this height
/// shows the controls used every day, the whole first section even with a line for an error message, and the start of
/// the next.
pub const EXPANDED_SIZE: egui::Vec2 = egui::vec2(COMPACT_SIZE.x, 620.0);

/// The side of the square theme button in the window's top-right corner: a little taller than the first row, which
/// leaves it clear of the second.
pub(crate) const THEME_BUTTON_SIZE: f32 = 28.0;

/// A checkbox for an on/off preference. Returns the new state when the user clicked it.
pub(crate) fn checkbox(ui: &mut Ui, label: &str, current: Toggle) -> Option<bool> {
    let on = is_on(current);
    widgets::checkbox(ui, label, on).changed().then_some(!on)
}

/// The width of a combo box offering the choices `choices` names: its padding, the widest name in either interface
/// language and the arrow after it. A combo this wide keeps its width and its right edge whatever it shows.
fn combo_width<T, const N: usize>(ui: &Ui, choices: impl Fn(&'static Labels) -> [(T, &'static str); N]) -> f32 {
    let widest =
        widgets::widest_in_both_languages(ui, &TextStyle::Button, |text| choices(text).map(|(_, label)| label));
    let spacing = ui.spacing();
    2.0 * spacing.button_padding.x + widest + spacing.icon_spacing + spacing.icon_width
}

/// A drop-down of the choices `choices` names, in the language of `text`. Returns the new choice when the user picked
/// another.
pub(crate) fn combo<T: Copy + PartialEq, const N: usize>(
    ui: &mut Ui,
    id_salt: &str,
    current: T,
    choices: impl Fn(&'static Labels) -> [(T, &'static str); N],
    text: &'static Labels,
) -> Option<T> {
    let width = combo_width(ui, &choices);
    let named = choices(text);
    let mut selected = current;
    let shown = named
        .iter()
        .find(|(value, _)| *value == current)
        .map_or("", |(_, label)| *label);
    egui::ComboBox::from_id_salt(id_salt)
        .width(width)
        .selected_text(shown)
        .show_ui(ui, |ui| {
            for (value, label) in named {
                ui.selectable_value(&mut selected, value, label);
            }
        });
    (selected != current).then_some(selected)
}

/// Radio buttons side by side. Returns the new choice when the user picked another.
fn radios<T: Copy + PartialEq>(ui: &mut Ui, current: T, choices: &[(T, &str)]) -> Option<T> {
    let mut selected = current;
    ui.horizontal(|ui| {
        for (value, label) in choices {
            if widgets::radio(ui, label, selected == *value).clicked() {
                selected = *value;
            }
        }
    });
    (selected != current).then_some(selected)
}

/// An on/off preference: where its label is among the texts, where its value lives in the view, and the form's setter
/// for it.
#[derive(Debug, Clone, Copy)]
struct OptionBox {
    label: fn(&Labels) -> &'static str,
    /// The form's rule that hides or greys the box, if any.
    gate: Option<GatedControl>,
    value: fn(&PreferencesView) -> Toggle,
    set: fn(&mut SettingsForm, bool),
}

impl OptionBox {
    const fn new(
        label: fn(&Labels) -> &'static str,
        value: fn(&PreferencesView) -> Toggle,
        set: fn(&mut SettingsForm, bool),
    ) -> Self {
        Self {
            label,
            gate: None,
            value,
            set,
        }
    }

    /// The same box, hidden or greyed as the form's rule for `gate` says.
    const fn gated(self, gate: GatedControl) -> Self {
        Self {
            gate: Some(gate),
            ..self
        }
    }
}

/// The [`OptionBox`] labelled with the text `$label` for the view's field at `$path`, which the form's `$set` changes.
macro_rules! option_box {
    ($label:ident, $($path:ident).+, $set:ident) => {
        OptionBox::new(|text| text.$label, |view| view.$($path).+, SettingsForm::$set)
    };
}

const BEEP: OptionBox = option_box!(beep, switching.beep, set_beep);

const TYPING_OPTIONS: [OptionBox; 11] = [
    option_box!(new_tone_style, typing.new_tone_style, set_new_tone_style),
    option_box!(fix_autocomplete, compatibility.fix_autocomplete, set_fix_autocomplete)
        .gated(GatedControl::FixAutocomplete),
    option_box!(spell_check, typing.spell_check, set_spell_check),
    option_box!(
        restore_invalid_word,
        typing.restore_invalid_word,
        set_restore_invalid_word
    )
    .gated(GatedControl::RestoreInvalidWord),
    option_box!(
        allow_zwjf_consonants,
        typing.allow_zwjf_consonants,
        set_allow_zwjf_consonants
    )
    .gated(GatedControl::AllowZwjfConsonants),
    option_box!(
        capitalize_sentences,
        typing.capitalize_sentences,
        set_capitalize_sentences
    ),
    option_box!(smart_switch, switching.smart_switch, set_smart_switch),
    option_box!(
        remember_code_table,
        switching.remember_code_table,
        set_remember_code_table
    ),
    option_box!(
        work_in_other_languages,
        switching.work_in_other_languages,
        set_work_in_other_languages
    ),
    option_box!(keep_w, behaviours.keep_w, set_keep_w),
    option_box!(only_text_boxes, behaviours.only_text_boxes, set_only_text_boxes),
];

const MACRO_OPTIONS: [OptionBox; 6] = [
    option_box!(use_macros, macros.use_macros, set_use_macros),
    option_box!(macros_in_english, macros.macros_in_english, set_macros_in_english),
    option_box!(macro_match_case, macros.match_case, set_macro_match_case),
    option_box!(quick_telex, typing.quick_telex, set_quick_telex),
    option_box!(
        quick_first_consonant,
        typing.quick_first_consonant,
        set_quick_first_consonant
    ),
    option_box!(
        quick_last_consonant,
        typing.quick_last_consonant,
        set_quick_last_consonant
    ),
];

const SYSTEM_OPTIONS: [OptionBox; 7] = [
    option_box!(start_with_windows, system.start_with_windows, set_start_with_windows),
    option_box!(
        show_settings_at_startup,
        system.show_settings_at_startup,
        set_show_settings_at_startup
    ),
    option_box!(desktop_shortcut, system.desktop_shortcut, set_desktop_shortcut).gated(GatedControl::DesktopShortcut),
    option_box!(
        run_as_administrator,
        system.run_as_administrator,
        set_run_as_administrator
    )
    .gated(GatedControl::RunAsAdministrator),
    option_box!(
        store_app_support,
        compatibility.store_app_support,
        set_store_app_support
    )
    .gated(GatedControl::StoreAppSupport),
    option_box!(chromium_fix, compatibility.chromium_fix, set_chromium_fix).gated(GatedControl::ChromiumFix),
    option_box!(clipboard_input, compatibility.clipboard_input, set_clipboard_input)
        .gated(GatedControl::ClipboardInput),
];

/// A section only the expanded window shows: its title, its options, and the button under them that opens another
/// window, if any.
#[derive(Debug, Clone, Copy)]
struct Section {
    /// The id of the options' grid. It is not the title, which changes with the language, so the grid keeps its layout
    /// when the language changes.
    id: &'static str,
    title: fn(&Labels) -> &'static str,
    options: &'static [OptionBox],
    button: Option<Button>,
}

/// The sections in the order they are drawn, the system options first.
const SECTIONS: [Section; 3] = [
    Section {
        id: "system_options",
        title: |text| text.section_system,
        options: &SYSTEM_OPTIONS,
        button: None,
    },
    Section {
        id: "typing_options",
        title: |text| text.section_typing,
        options: &TYPING_OPTIONS,
        button: Some(Button::OpenAlwaysOn),
    },
    Section {
        id: "macro_options",
        title: |text| text.section_macros,
        options: &MACRO_OPTIONS,
        button: Some(Button::OpenMacros),
    },
];

/// Draws `option` as a checkbox, greyed when the form's rule says so, and stores a click.
fn option_checkbox(ui: &mut Ui, form: &mut SettingsForm, option: &OptionBox) {
    let enabled = option.gate.is_none_or(|gate| form.is_enabled(gate));
    let on = is_on((option.value)(form.view()));
    let label = (option.label)(labels_for(form.view()));
    // Greyed in place rather than in a scope of its own: in a grid row taller than the box, such a scope grows the row
    // a little more on every frame.
    let box_response = ui.add_enabled(enabled, |ui: &mut Ui| widgets::checkbox(ui, label, on));
    if box_response.changed() {
        (option.set)(form, !on);
    }
}

/// The width of each options column: the widest option of any section in either interface language, box included,
/// so the second column starts at the same place in every section and in both languages.
fn option_column_width(ui: &Ui) -> f32 {
    let widest = widgets::widest_in_both_languages(ui, &TextStyle::Body, |text| {
        SECTIONS
            .iter()
            .flat_map(|section| section.options)
            .map(|option| (option.label)(text))
    });
    widgets::CHECKBOX_SIZE + widgets::CHECKBOX_GAP + widest
}

/// Lays `options` out two to a row, in their order, leaving out those the form hides.
fn options_in_two_columns(ui: &mut Ui, form: &mut SettingsForm, id_salt: &str, options: &[OptionBox]) {
    let shown: Vec<&OptionBox> = options
        .iter()
        .filter(|option| option.gate.is_none_or(|gate| form.is_visible(gate)))
        .collect();
    egui::Grid::new(id_salt)
        .num_columns(2)
        .min_col_width(option_column_width(ui))
        .show(ui, |ui| {
            for row in shown.chunks(2) {
                for option in row {
                    option_checkbox(ui, form, option);
                }
                ui.end_row();
            }
        });
}

/// A section's title: muted over a thin line, so the options stand out rather than the headings, and in the body size,
/// since Vietnamese marks are hard to read any smaller.
fn section_title(ui: &mut Ui, title: &str) -> egui::Response {
    ui.add_space(ui.spacing().item_spacing.y);
    let response = ui.label(egui::RichText::new(title).weak());
    ui.separator();
    response
}

fn options_section(ui: &mut Ui, form: &mut SettingsForm, section: &Section, actions: &mut Vec<UiAction>) {
    let text = labels_for(form.view());
    drop(section_title(ui, (section.title)(text)));
    options_in_two_columns(ui, form, section.id, section.options);
    if let Some(button) = section.button
        && ui.button(button_label(text, button, form.is_expanded())).clicked()
    {
        actions.extend(button.action());
    }
}

/// The room between the about texts' two columns, so the source reads apart from the name and the version.
const ABOUT_COLUMN_GAP: f32 = 32.0;

/// The about texts: the name and the source link in one row, the version with its licence under the name. The first
/// column and the link's label take the room of their widest text in either interface language, so the link stays
/// where it is when the language changes.
fn about_grid(ui: &mut Ui, text: &Labels) {
    let first = widgets::widest_in_both_languages(ui, &TextStyle::Body, |text| {
        [text.about_name, text.about_version_license]
    });
    let source = widgets::widest_in_both_languages(ui, &TextStyle::Body, |text| [text.about_source]);
    let spacing = egui::vec2(ABOUT_COLUMN_GAP, ui.spacing().item_spacing.y);
    egui::Grid::new("about").num_columns(2).spacing(spacing).show(ui, |ui| {
        fixed_width_cell(ui, first, |ui| ui.label(text.about_name));
        ui.horizontal(|ui| {
            fixed_width_cell(ui, source, |ui| ui.label(text.about_source));
            ui.hyperlink_to(text.about_source_link, text.about_url);
        });
        ui.end_row();
        fixed_width_cell(ui, first, |ui| ui.label(text.about_version_license));
        ui.end_row();
    });
}

/// The about text, the last section. When `about_to_reveal` asks for it, the scroll area brings it into view once.
fn about_section(ui: &mut Ui, text: &Labels, about_to_reveal: &mut bool) {
    let title = section_title(ui, text.section_about);
    about_grid(ui, text);
    // egui draws a pass it will discard again at once and drops what that pass asked of the scroll area, so the
    // request waits for the pass that is kept.
    if *about_to_reveal && !ui.ctx().will_discard() {
        title.scroll_to_me_animation(Some(egui::Align::TOP), egui::style::ScrollAnimation::none());
        *about_to_reveal = false;
    }
}

/// The sections only the expanded window shows.
fn expanded_sections(ui: &mut Ui, form: &mut SettingsForm, about_to_reveal: &mut bool, actions: &mut Vec<UiAction>) {
    for section in &SECTIONS {
        options_section(ui, form, section, actions);
    }
    about_section(ui, labels_for(form.view()), about_to_reveal);
}

/// Adds what `add_contents` draws in a cell `width` wide, whatever its own width, so the next cell on the row starts at
/// the same place in both interface languages.
fn fixed_width_cell<R>(ui: &mut Ui, width: f32, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
    let size = egui::vec2(width, ui.spacing().interact_size.y);
    ui.allocate_ui_with_layout(size, egui::Layout::left_to_right(egui::Align::Center), |ui| {
        ui.set_min_width(width);
        add_contents(ui)
    })
    .inner
}

/// The labels of the controls used every day, which share one column.
const fn control_labels(text: &Labels) -> [&'static str; 6] {
    [
        text.input_method,
        text.code_table,
        text.switch_hotkey,
        text.settings_hotkey,
        text.language,
        text.interface_language,
    ]
}

/// The label of a control row, in the label column `width` wide.
fn row_label(ui: &mut Ui, width: f32, label: &str) {
    fixed_width_cell(ui, width, |ui| ui.label(label));
}

/// The controls used every day, as grid cells: a label, then its control. The labels take the room of the widest in
/// either interface language, so the controls start at the same place in both. Each hotkey editor sits in one cell and
/// draws the same parts in the same order, so the modifier boxes and key fields of the two hotkey rows line up.
fn control_rows(ui: &mut Ui, form: &mut SettingsForm) {
    let text = labels_for(form.view());
    let interface_language = form.view().system.interface_language;
    let label_width = widgets::widest_in_both_languages(ui, &TextStyle::Body, control_labels);
    row_label(ui, label_width, text.input_method);
    if let Some(method) = combo(ui, "input_method", form.view().typing.input_method, input_methods, text) {
        form.set_input_method(method);
    }
    ui.end_row();

    row_label(ui, label_width, text.code_table);
    if let Some(table) = combo(ui, "code_table", form.view().typing.code_table, code_tables, text) {
        form.set_code_table(table);
    }
    ui.end_row();

    row_label(ui, label_width, text.switch_hotkey);
    if let Some(hotkey) = show_hotkey(ui, "switch_hotkey", form.view().switching.hotkey, interface_language) {
        form.set_switch_hotkey(hotkey);
    }
    option_checkbox(ui, form, &BEEP);
    ui.end_row();

    row_label(ui, label_width, text.settings_hotkey);
    let settings_hotkey = form.view().behaviours.settings_hotkey;
    if let Some(hotkey) = show_hotkey(ui, "settings_hotkey", settings_hotkey, interface_language) {
        form.set_settings_hotkey(hotkey);
    }
    ui.end_row();

    row_label(ui, label_width, text.language);
    if let Some(language) = radios(ui, form.view().language, &typing_languages(text)) {
        form.set_language(language);
    }
    ui.end_row();

    row_label(ui, label_width, text.interface_language);
    if let Some(language) = radios(ui, interface_language, &interface_languages(text)) {
        form.set_interface_language(language);
    }
    ui.end_row();
}

/// What `button` reads, in the window expanded or not.
const fn button_label(text: &Labels, button: Button, expanded: bool) -> &'static str {
    match button {
        Button::CloseWindow => text.button_close,
        Button::ExitApp => text.button_exit,
        Button::ResetToDefaults => text.button_reset,
        Button::OpenMacros => text.macro_table,
        Button::OpenAlwaysOn => text.always_on_apps,
        Button::ToggleExpanded if expanded => text.button_collapse,
        Button::ToggleExpanded => text.button_expand,
    }
}

/// Every label the bar shows in either mode, in the language of `text`, so the buttons keep their width when the
/// window expands or collapses.
fn bar_labels(text: &Labels) -> Vec<&'static str> {
    [false, true]
        .into_iter()
        .flat_map(|expanded| {
            let bar = ButtonBar::for_expanded(expanded);
            bar.left
                .into_iter()
                .chain(bar.right)
                .map(move |button| button_label(text, button, expanded))
        })
        .collect()
}

/// The width every bar button takes when the window has room: the theme's button width, or more when the widest label
/// of the bar in either interface language needs it, so no label is cut and no button moves when the language changes.
fn bar_button_width(ui: &Ui) -> f32 {
    let widest = widgets::widest_in_both_languages(ui, &TextStyle::Button, bar_labels);
    BUTTON_SIZE.x.max(widest + 2.0 * ui.spacing().button_padding.x)
}

/// One button of the bottom bar, `size` like the others. Returns whether it was clicked.
fn bar_button(ui: &mut Ui, form: &SettingsForm, button: Button, size: egui::Vec2) -> bool {
    let label = button_label(labels_for(form.view()), button, form.is_expanded());
    // A button would otherwise grow to its label in a window too narrow for the bar, and push the last ones out.
    ui.add_sized(size, egui::Button::new(label).truncate()).clicked()
}

/// The width each of `count` bar buttons takes in `room`: `wanted`, when they fit side by side with `spacing` between
/// them. A narrower room first takes the gap before the right-hand buttons, down to `spacing`; then the buttons shrink
/// alike, so the last one still ends inside the window.
fn fitted_button_width(wanted: f32, room: f32, count: f32, spacing: f32) -> f32 {
    wanted.min((room - spacing * (count - 1.0)) / count)
}

/// The buttons of the bottom bar, where the form puts them: the left one at the left edge, the others ending at the
/// right edge. They are added left to right, so the keyboard reaches them in reading order. Returns the button clicked.
fn bar_buttons(ui: &mut Ui, form: &SettingsForm) -> Option<Button> {
    let bar = form.bar();
    let spacing = ui.spacing().item_spacing.x;
    let count = bar.left.iter().chain(&bar.right).map(|_| 1.0).sum();
    let wanted = bar_button_width(ui);
    let size = egui::vec2(
        fitted_button_width(wanted, ui.available_width(), count, spacing),
        BUTTON_SIZE.y,
    );
    ui.horizontal(|ui| {
        let mut clicked = None;
        if let Some(button) = bar.left
            && bar_button(ui, form, button, size)
        {
            clicked = Some(button);
        }
        let right_group = bar.right.iter().map(|_| size.x + spacing).sum::<f32>() - spacing;
        ui.add_space((ui.available_width() - right_group).max(0.0));
        for &button in &bar.right {
            if bar_button(ui, form, button, size) {
                clicked = Some(button);
            }
        }
        clicked
    })
    .inner
}

/// Splits the window's inner area into the room for what it shows and, a margin below, the bar along its bottom edge.
fn split_off_bar(ui: &Ui) -> (egui::Rect, egui::Rect) {
    let area = ui.max_rect();
    let bar_top = area.bottom() - BUTTON_SIZE.y;
    let above_bottom = (bar_top - f32::from(ui.spacing().window_margin.bottom)).max(area.top());
    (
        egui::Rect::from_min_max(area.min, egui::pos2(area.right(), above_bottom)),
        egui::Rect::from_min_max(egui::pos2(area.left(), bar_top), area.max),
    )
}

/// What a frame of the window drew that the window acts on afterwards.
struct Drawn {
    /// The bar button clicked, if any.
    clicked: Option<Button>,
    /// The height of the controls used every day and the message under them: all the compact window shows above its
    /// bar.
    rows_height: f32,
}

/// What the window shows, then the bar pinned along its bottom edge, outside the scrolling sections. The bar is drawn
/// last so the keyboard reaches the controls first and the reset last.
fn window_contents(
    ui: &mut Ui,
    form: &mut SettingsForm,
    about_to_reveal: &mut bool,
    actions: &mut Vec<UiAction>,
) -> Drawn {
    let (above, bar) = split_off_bar(ui);
    let rows_height = ui
        .scope_builder(egui::UiBuilder::new().max_rect(above), |ui| {
            ui.shrink_clip_rect(above);
            contents(ui, form, about_to_reveal, actions)
        })
        .inner;
    let clicked = ui
        .scope_builder(egui::UiBuilder::new().max_rect(bar), |ui| bar_buttons(ui, form))
        .inner;
    Drawn { clicked, rows_height }
}

/// The button in the top-right corner of `ui`, laid over the first row's empty end, that switches the windows between
/// light and dark. It shows where a click leads: a moon in the light theme, a sun in the dark one. Returns whether it
/// was clicked.
fn theme_button(ui: &Ui, form: &SettingsForm) -> bool {
    let text = labels_for(form.view());
    let (icon, hint) = match form.view().system.theme {
        ThemeValue::Light => (widgets::ThemeIcon::Moon, text.theme_to_dark),
        ThemeValue::Dark => (widgets::ThemeIcon::Sun, text.theme_to_light),
    };
    let inner = ui.max_rect();
    let rect = egui::Rect::from_min_max(
        egui::pos2(inner.right() - THEME_BUTTON_SIZE, inner.top()),
        egui::pos2(inner.right(), inner.top() + THEME_BUTTON_SIZE),
    );
    widgets::icon_button(ui, rect, icon, hint).clicked()
}

/// The controls used every day with the theme button beside them and, when expanded, the other sections in a scroll
/// area below; in compact mode nothing scrolls. Returns the height of the controls used every day and the message under
/// them. The theme button comes after the controls used every day, so Tab reaches it after them, as they are read.
fn contents(ui: &mut Ui, form: &mut SettingsForm, about_to_reveal: &mut bool, actions: &mut Vec<UiAction>) -> f32 {
    let rows_height = ui.scope(|ui| daily_controls(ui, form)).response.rect.height();
    if theme_button(ui, form) {
        form.toggle_theme();
    }
    if form.is_expanded() {
        egui::ScrollArea::vertical().show(ui, |ui| expanded_sections(ui, form, about_to_reveal, actions));
    }
    rows_height
}

/// The controls used every day, and the message of the last refused change under them.
fn daily_controls(ui: &mut Ui, form: &mut SettingsForm) {
    egui::Grid::new("control_rows")
        .num_columns(3)
        .show(ui, |ui| control_rows(ui, form));
    if let Some(message) = form.error() {
        ui.colored_label(ui.visuals().error_fg_color, message);
    }
}

/// The inner height of a compact window whose controls used every day, with the message under them, are `rows_height`
/// high: the margin above them, the margin between them and the bar, the bar, and the margin below it.
fn compact_height(margin: egui::Margin, rows_height: f32) -> f32 {
    f32::from(margin.top) + rows_height + 2.0 * f32::from(margin.bottom) + BUTTON_SIZE.y
}

/// Gives the native window of `ctx`'s viewport the inner size `wanted`, moved up or made shorter as
/// [`fit_on_screen`] says, so the bottom bar stays on screen. When the window's place or its monitor is unknown, it
/// only asks for `wanted`.
fn resize_on_screen(ctx: &egui::Context, wanted: egui::Vec2) {
    let native = ctx.input(|input| {
        let info = input.viewport();
        Some((info.outer_rect?, info.inner_rect?, info.monitor_size?))
    });
    let placement = native.map_or(
        Placement {
            move_to: None,
            inner_size: wanted,
        },
        |(outer, inner, monitor)| fit_on_screen(outer, inner, monitor, wanted),
    );
    if let Some(corner) = placement.move_to {
        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(corner));
    }
    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(placement.inner_size));
}

/// The settings window: draw it every frame, act on what it returns.
#[derive(Debug, Clone)]
pub struct SettingsWindow {
    form: SettingsForm,
    /// The about text is to be scrolled into view on the next frame.
    about_to_reveal: bool,
    /// What the window shows changed size, and its native window is to take the new size on the next frame.
    resize_pending: bool,
    /// The inner size of the compact window: the size it opens at, until a frame measures what it shows.
    compact_size: egui::Vec2,
}

impl SettingsWindow {
    #[must_use]
    pub const fn new(prefs: PreferencesView, platform: Platform) -> Self {
        Self {
            form: SettingsForm::new(prefs, platform),
            about_to_reveal: false,
            resize_pending: false,
            compact_size: COMPACT_SIZE,
        }
    }

    /// Expands the window and brings the about text, the last section, into view on its next frame: the About menu
    /// item leads here. The user may scroll away afterwards.
    pub const fn reveal_about(&mut self) {
        self.expand();
        self.about_to_reveal = true;
    }

    /// The same window showing every section from the start, scrolled to the top; [`Self::reveal_about`] scrolls to
    /// the about text instead.
    #[must_use]
    pub const fn expanded(mut self) -> Self {
        self.expand();
        self
    }

    /// Shows every section, for a way in that leads to something only the expanded window shows, such as the about
    /// text. The native window grows on the next frame; an expanded window stays as it is.
    pub const fn expand(&mut self) {
        if !self.form.is_expanded() {
            self.toggle_expanded();
        }
    }

    const fn toggle_expanded(&mut self) {
        self.form.toggle_expanded();
        self.resize_pending = true;
    }

    /// Acts on a clicked button. The expand button concerns this window alone: it resizes the window and asks the
    /// process nothing.
    fn press(&mut self, button: Button, actions: &mut Vec<UiAction>) {
        if button == Button::ToggleExpanded {
            self.toggle_expanded();
        }
        actions.extend(button.action());
    }

    /// Takes `height` as the compact window's inner height when it differs from the one it has by a point or more, and
    /// resizes a compact window to it. A smaller difference is rounding, which would resize the window on every frame.
    fn fit_compact_height(&mut self, height: f32) {
        if (height - self.compact_size.y).abs() < 1.0 {
            return;
        }
        self.compact_size.y = height;
        if !self.form.is_expanded() {
            self.resize_pending = true;
        }
    }

    /// The inner size of what the window shows now: the size to open it at, or to give it in another native window.
    /// Compact, it is as high as its rows and bar, measured on the last frame.
    #[must_use]
    pub const fn size(&self) -> egui::Vec2 {
        if self.form.is_expanded() {
            EXPANDED_SIZE
        } else {
            self.compact_size
        }
    }

    /// The language the window speaks: the interface language of the preferences it edits.
    #[must_use]
    pub const fn interface_language(&self) -> InterfaceLanguageValue {
        self.form.view().system.interface_language
    }

    /// Replaces the edited preferences, after a reset or when the background process changed something.
    pub fn set_preferences(&mut self, prefs: PreferencesView) {
        self.form.set_preferences(prefs);
    }

    /// Shows a message under the controls used every day, or clears it. The user's edits stay.
    pub const fn show_error(&mut self, message: Option<&'static str>) {
        self.form.show_error(message);
    }

    /// Draws the window into `ui`, the root ui of its viewport, and returns what the user asked for this frame.
    pub fn show(&mut self, ui: &mut Ui) -> Vec<UiAction> {
        let mut actions = Vec::new();
        let form = &mut self.form;
        let about_to_reveal = &mut self.about_to_reveal;
        let margin = ui.spacing().window_margin;
        let frame = egui::Frame::central_panel(ui.style()).inner_margin(margin);
        let drawn = egui::CentralPanel::default()
            .frame(frame)
            .show(ui, |ui| window_contents(ui, form, about_to_reveal, &mut actions))
            .inner;
        // A pass egui will discard lays the grids out for the first time, before they know their sizes.
        if !ui.ctx().will_discard() {
            self.fit_compact_height(compact_height(margin, drawn.rows_height));
        }
        if let Some(button) = drawn.clicked {
            self.press(button, &mut actions);
        }
        if self.resize_pending {
            self.resize_pending = false;
            resize_on_screen(ui.ctx(), self.size());
        }
        // The windows share one context, so every open window takes the theme the user just chose.
        theme::set_theme(ui.ctx(), self.form.view().system.theme);
        if let Some(view) = self.form.take_save() {
            actions.insert(0, UiAction::Save(view));
        }
        actions
    }
}

#[cfg(test)]
mod tests;
