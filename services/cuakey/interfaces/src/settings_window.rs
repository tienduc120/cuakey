//! The settings window drawn with egui. It opens small, with the controls used every day, and the expand button in the
//! bottom bar shows the other sections. Every change applies at once: a frame that changed a control returns one
//! `Save` with the whole edited view. What is shown and enabled lives in [`SettingsForm`]; this file only draws it.

use cuakey_application::{CodeTableValue, InputMethodValue, LanguageValue, PreferencesView, Toggle};
use egui::Ui;

use crate::hotkey_editor::show_hotkey;
use crate::settings_form::{GatedControl, SettingsForm, is_on};
use crate::theme::BUTTON_SIZE;
use crate::{Button, Platform, UiAction, labels, widgets};

const INPUT_METHODS: [(InputMethodValue, &str); 3] = [
    (InputMethodValue::Telex, labels::INPUT_METHOD_TELEX),
    (InputMethodValue::Vni, labels::INPUT_METHOD_VNI),
    (InputMethodValue::SimpleTelex, labels::INPUT_METHOD_SIMPLE_TELEX),
];

pub(crate) const CODE_TABLES: [(CodeTableValue, &str); 5] = [
    (CodeTableValue::Unicode, labels::CODE_TABLE_UNICODE),
    (CodeTableValue::Tcvn3, labels::CODE_TABLE_TCVN3),
    (CodeTableValue::VniWindows, labels::CODE_TABLE_VNI_WINDOWS),
    (CodeTableValue::UnicodeCompound, labels::CODE_TABLE_UNICODE_COMPOUND),
    (CodeTableValue::Cp1258, labels::CODE_TABLE_CP1258),
];

const LANGUAGES: [(LanguageValue, &str); 2] = [
    (LanguageValue::Vietnamese, labels::LANGUAGE_VIETNAMESE),
    (LanguageValue::English, labels::LANGUAGE_ENGLISH),
];

/// The window's inner size with the controls used every day only, the size it opens at. With the shared style, the
/// rows measure at most 474 x 185 points in Segoe UI and the other fonts the window may borrow, 212 high with a line
/// for an error message; the margins and the bottom bar come on top, so nothing scrolls or clips. The width is the
/// expanded window's, so expanding only moves the bottom edge and the bar's buttons stay where they were.
pub const COMPACT_SIZE: egui::Vec2 = egui::vec2(560.0, 300.0);

/// The window's inner size with every section. The options sit two to a row, and the widest needs 257 points, so two
/// columns fit in this width. The sections are taller than a small screen, so they scroll above the bottom bar; this
/// height shows the controls used every day and the whole first section.
pub const EXPANDED_SIZE: egui::Vec2 = egui::vec2(COMPACT_SIZE.x, 620.0);

/// A checkbox for an on/off preference. Returns the new state when the user clicked it.
pub(crate) fn checkbox(ui: &mut Ui, label: &str, current: Toggle) -> Option<bool> {
    let on = is_on(current);
    widgets::checkbox(ui, label, on).changed().then_some(!on)
}

/// A drop-down of choices. Returns the new choice when the user picked another.
pub(crate) fn combo<T: Copy + PartialEq>(ui: &mut Ui, id_salt: &str, current: T, choices: &[(T, &str)]) -> Option<T> {
    let mut selected = current;
    let shown = choices
        .iter()
        .find(|(value, _)| *value == current)
        .map_or("", |(_, label)| *label);
    egui::ComboBox::from_id_salt(id_salt)
        .selected_text(shown)
        .show_ui(ui, |ui| {
            for (value, label) in choices {
                ui.selectable_value(&mut selected, *value, *label);
            }
        });
    (selected != current).then_some(selected)
}

/// Radio buttons side by side. Returns the new choice when the user picked another.
fn radios<T: Copy + PartialEq>(ui: &mut Ui, current: T, choices: &[(T, &str)]) -> Option<T> {
    let mut selected = current;
    ui.horizontal(|ui| {
        for (value, label) in choices {
            ui.radio_value(&mut selected, *value, *label);
        }
    });
    (selected != current).then_some(selected)
}

/// An on/off preference: its label, where its value lives in the view, and the form's setter for it.
#[derive(Debug, Clone, Copy)]
struct OptionBox {
    label: &'static str,
    /// The form's rule that hides or greys the box, if any.
    gate: Option<GatedControl>,
    value: fn(&PreferencesView) -> Toggle,
    set: fn(&mut SettingsForm, bool),
}

impl OptionBox {
    const fn new(label: &'static str, value: fn(&PreferencesView) -> Toggle, set: fn(&mut SettingsForm, bool)) -> Self {
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

/// The [`OptionBox`] labelled `labels::$label` for the view's field at `$path`, which the form's `$set` changes.
macro_rules! option_box {
    ($label:ident, $($path:ident).+, $set:ident) => {
        OptionBox::new(labels::$label, |view| view.$($path).+, SettingsForm::$set)
    };
}

const BEEP: OptionBox = option_box!(BEEP, switching.beep, set_beep);

const TYPING_OPTIONS: [OptionBox; 11] = [
    option_box!(NEW_TONE_STYLE, typing.new_tone_style, set_new_tone_style),
    option_box!(FIX_AUTOCOMPLETE, compatibility.fix_autocomplete, set_fix_autocomplete)
        .gated(GatedControl::FixAutocomplete),
    option_box!(SPELL_CHECK, typing.spell_check, set_spell_check),
    option_box!(
        RESTORE_INVALID_WORD,
        typing.restore_invalid_word,
        set_restore_invalid_word
    )
    .gated(GatedControl::RestoreInvalidWord),
    option_box!(
        ALLOW_ZWJF_CONSONANTS,
        typing.allow_zwjf_consonants,
        set_allow_zwjf_consonants
    )
    .gated(GatedControl::AllowZwjfConsonants),
    option_box!(
        CAPITALIZE_SENTENCES,
        typing.capitalize_sentences,
        set_capitalize_sentences
    ),
    option_box!(SMART_SWITCH, switching.smart_switch, set_smart_switch),
    option_box!(
        REMEMBER_CODE_TABLE,
        switching.remember_code_table,
        set_remember_code_table
    ),
    option_box!(
        WORK_IN_OTHER_LANGUAGES,
        switching.work_in_other_languages,
        set_work_in_other_languages
    ),
    option_box!(KEEP_W, behaviours.keep_w, set_keep_w),
    option_box!(ONLY_TEXT_BOXES, behaviours.only_text_boxes, set_only_text_boxes),
];

const MACRO_OPTIONS: [OptionBox; 6] = [
    option_box!(USE_MACROS, macros.use_macros, set_use_macros),
    option_box!(MACROS_IN_ENGLISH, macros.macros_in_english, set_macros_in_english),
    option_box!(MACRO_MATCH_CASE, macros.match_case, set_macro_match_case),
    option_box!(QUICK_TELEX, typing.quick_telex, set_quick_telex),
    option_box!(
        QUICK_FIRST_CONSONANT,
        typing.quick_first_consonant,
        set_quick_first_consonant
    ),
    option_box!(
        QUICK_LAST_CONSONANT,
        typing.quick_last_consonant,
        set_quick_last_consonant
    ),
];

const SYSTEM_OPTIONS: [OptionBox; 7] = [
    option_box!(START_WITH_WINDOWS, system.start_with_windows, set_start_with_windows),
    option_box!(
        SHOW_SETTINGS_AT_STARTUP,
        system.show_settings_at_startup,
        set_show_settings_at_startup
    ),
    option_box!(DESKTOP_SHORTCUT, system.desktop_shortcut, set_desktop_shortcut).gated(GatedControl::DesktopShortcut),
    option_box!(
        RUN_AS_ADMINISTRATOR,
        system.run_as_administrator,
        set_run_as_administrator
    )
    .gated(GatedControl::RunAsAdministrator),
    option_box!(
        STORE_APP_SUPPORT,
        compatibility.store_app_support,
        set_store_app_support
    )
    .gated(GatedControl::StoreAppSupport),
    option_box!(CHROMIUM_FIX, compatibility.chromium_fix, set_chromium_fix).gated(GatedControl::ChromiumFix),
    option_box!(CLIPBOARD_INPUT, compatibility.clipboard_input, set_clipboard_input)
        .gated(GatedControl::ClipboardInput),
];

/// A section only the expanded window shows: its title, its options, and the button under them that opens another
/// window, if any.
#[derive(Debug, Clone, Copy)]
struct Section {
    title: &'static str,
    options: &'static [OptionBox],
    button: Option<Button>,
}

const SECTIONS: [Section; 3] = [
    Section {
        title: labels::SECTION_TYPING,
        options: &TYPING_OPTIONS,
        button: Some(Button::OpenAlwaysOn),
    },
    Section {
        title: labels::SECTION_MACROS,
        options: &MACRO_OPTIONS,
        button: Some(Button::OpenMacros),
    },
    Section {
        title: labels::SECTION_SYSTEM,
        options: &SYSTEM_OPTIONS,
        button: None,
    },
];

/// Draws `option` as a checkbox, greyed when the form's rule says so, and stores a click.
fn option_checkbox(ui: &mut Ui, form: &mut SettingsForm, option: &OptionBox) {
    let enabled = option.gate.is_none_or(|gate| form.is_enabled(gate));
    let current = (option.value)(form.view());
    if let Some(on) = ui
        .add_enabled_ui(enabled, |ui| checkbox(ui, option.label, current))
        .inner
    {
        (option.set)(form, on);
    }
}

/// Lays `options` out two to a row, in their order, leaving out those the form hides. The columns split the width
/// evenly, so the second column starts at the same place in every section.
fn options_in_two_columns(ui: &mut Ui, form: &mut SettingsForm, id_salt: &str, options: &[OptionBox]) {
    let shown: Vec<&OptionBox> = options
        .iter()
        .filter(|option| option.gate.is_none_or(|gate| form.is_visible(gate)))
        .collect();
    let column_width = (ui.available_width() - ui.spacing().item_spacing.x) / 2.0;
    egui::Grid::new(id_salt)
        .num_columns(2)
        .min_col_width(column_width)
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
fn section_title(ui: &mut Ui, title: &str) {
    ui.add_space(ui.spacing().item_spacing.y);
    ui.label(egui::RichText::new(title).weak());
    ui.separator();
}

fn options_section(ui: &mut Ui, form: &mut SettingsForm, section: &Section, actions: &mut Vec<UiAction>) {
    section_title(ui, section.title);
    options_in_two_columns(ui, form, section.title, section.options);
    if let Some(button) = section.button
        && ui.button(button_label(form, button)).clicked()
    {
        actions.extend(button.action());
    }
}

fn about_section(ui: &mut Ui) {
    section_title(ui, labels::SECTION_ABOUT);
    ui.label(labels::ABOUT_NAME);
    ui.label(labels::ABOUT_VERSION);
    ui.label(labels::ABOUT_LICENSE);
    ui.hyperlink(labels::ABOUT_URL);
    ui.label(labels::ABOUT_CREDITS);
}

/// The sections only the expanded window shows.
fn expanded_sections(ui: &mut Ui, form: &mut SettingsForm, actions: &mut Vec<UiAction>) {
    for section in &SECTIONS {
        options_section(ui, form, section, actions);
    }
    about_section(ui);
}

/// The controls used every day, as grid cells: a label, then its control. Each hotkey editor sits in one cell and
/// draws the same parts in the same order, so the modifier boxes and key fields of the two hotkey rows line up.
fn control_rows(ui: &mut Ui, form: &mut SettingsForm) {
    ui.label(labels::INPUT_METHOD);
    if let Some(method) = combo(ui, "input_method", form.view().typing.input_method, &INPUT_METHODS) {
        form.set_input_method(method);
    }
    ui.end_row();

    ui.label(labels::CODE_TABLE);
    if let Some(table) = combo(ui, "code_table", form.view().typing.code_table, &CODE_TABLES) {
        form.set_code_table(table);
    }
    ui.end_row();

    ui.label(labels::SWITCH_HOTKEY);
    if let Some(hotkey) = show_hotkey(ui, "switch_hotkey", form.view().switching.hotkey) {
        form.set_switch_hotkey(hotkey);
    }
    option_checkbox(ui, form, &BEEP);
    ui.end_row();

    ui.label(labels::SETTINGS_HOTKEY);
    if let Some(hotkey) = show_hotkey(ui, "settings_hotkey", form.view().behaviours.settings_hotkey) {
        form.set_settings_hotkey(hotkey);
    }
    ui.end_row();

    ui.label(labels::LANGUAGE);
    if let Some(language) = radios(ui, form.view().language, &LANGUAGES) {
        form.set_language(language);
    }
    ui.end_row();
}

fn button_label(form: &SettingsForm, button: Button) -> &'static str {
    match button {
        Button::CloseWindow => labels::BUTTON_CLOSE,
        Button::ExitApp => labels::BUTTON_EXIT,
        Button::ResetToDefaults => labels::BUTTON_RESET,
        Button::OpenMacros => labels::MACRO_TABLE,
        Button::OpenAlwaysOn => labels::ALWAYS_ON_APPS,
        Button::ToggleExpanded if form.is_expanded() => labels::BUTTON_COLLAPSE,
        Button::ToggleExpanded => labels::BUTTON_EXPAND,
    }
}

/// Every label a bar button can show, in either mode, so the buttons keep their width when the window expands or
/// collapses.
const BAR_LABELS: [&str; 5] = [
    labels::BUTTON_EXPAND,
    labels::BUTTON_COLLAPSE,
    labels::BUTTON_EXIT,
    labels::BUTTON_CLOSE,
    labels::BUTTON_RESET,
];

/// The width every bar button takes: the theme's button width, or more when the widest of `labels` needs it, so no
/// label is ever cut.
fn bar_button_width(ui: &Ui, labels: &[&str]) -> f32 {
    let widest = labels
        .iter()
        .map(|label| widgets::button_text_width(ui, label))
        .fold(0.0, f32::max);
    BUTTON_SIZE.x.max(widest + 2.0 * ui.spacing().button_padding.x)
}

/// One button of the bottom bar, `size` like the others. Returns whether it was clicked.
fn bar_button(ui: &mut Ui, form: &SettingsForm, button: Button, size: egui::Vec2) -> bool {
    ui.add_sized(size, egui::Button::new(button_label(form, button)))
        .clicked()
}

/// The buttons of the bottom bar, where the form puts them. Returns the button clicked.
fn bar_buttons(ui: &mut Ui, form: &SettingsForm) -> Option<Button> {
    let bar = form.bar();
    let size = egui::vec2(bar_button_width(ui, &BAR_LABELS), BUTTON_SIZE.y);
    ui.horizontal(|ui| {
        let mut clicked = None;
        if let Some(button) = bar.left
            && bar_button(ui, form, button, size)
        {
            clicked = Some(button);
        }
        right_aligned(ui, form, &bar.right, size).or(clicked)
    })
    .inner
}

/// `buttons` at the right end of the row, in their order. Returns the button clicked.
fn right_aligned(ui: &mut Ui, form: &SettingsForm, buttons: &[Button], size: egui::Vec2) -> Option<Button> {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        let mut clicked = None;
        // Laid out from the right edge, so the last button comes first.
        for &button in buttons.iter().rev() {
            if bar_button(ui, form, button, size) {
                clicked = Some(button);
            }
        }
        clicked
    })
    .inner
}

/// The bar along the window's bottom edge, outside the scrolling sections so it never scrolls away. Returns the
/// button clicked.
fn bottom_bar(ui: &mut Ui, form: &SettingsForm) -> Option<Button> {
    let margin = ui.spacing().window_margin;
    // The panel above ends in a margin of its own, so the bar needs none at its top.
    let frame = egui::Frame::side_top_panel(ui.style()).inner_margin(egui::Margin { top: 0, ..margin });
    let height = BUTTON_SIZE.y + frame.total_margin().sum().y;
    egui::Panel::bottom("button_bar")
        .frame(frame)
        .show_separator_line(false)
        .exact_size(height)
        .show(ui, |ui| bar_buttons(ui, form))
        .inner
}

/// The controls used every day, the message of the last refused change under them, and, when expanded, the other
/// sections in a scroll area below. In compact mode nothing scrolls.
fn contents(ui: &mut Ui, form: &mut SettingsForm, actions: &mut Vec<UiAction>) {
    egui::Grid::new("control_rows")
        .num_columns(3)
        .show(ui, |ui| control_rows(ui, form));
    if let Some(message) = form.error() {
        ui.colored_label(ui.visuals().error_fg_color, message);
    }
    if form.is_expanded() {
        egui::ScrollArea::vertical().show(ui, |ui| expanded_sections(ui, form, actions));
    }
}

/// The inner size of a window showing what `form` shows.
const fn window_size(form: &SettingsForm) -> egui::Vec2 {
    if form.is_expanded() {
        EXPANDED_SIZE
    } else {
        COMPACT_SIZE
    }
}

/// Acts on a clicked button. The expand button concerns this window alone: it resizes the window and asks the process
/// nothing.
fn press(ui: &Ui, form: &mut SettingsForm, button: Button, actions: &mut Vec<UiAction>) {
    if button == Button::ToggleExpanded {
        form.toggle_expanded();
        ui.ctx()
            .send_viewport_cmd(egui::ViewportCommand::InnerSize(window_size(form)));
    }
    actions.extend(button.action());
}

/// The settings window: draw it every frame, act on what it returns.
#[derive(Debug, Clone)]
pub struct SettingsWindow {
    form: SettingsForm,
}

impl SettingsWindow {
    #[must_use]
    pub const fn new(prefs: PreferencesView, platform: Platform) -> Self {
        Self {
            form: SettingsForm::new(prefs, platform),
        }
    }

    /// The same window showing every section from the start, for a way in that leads to something only the expanded
    /// window shows, such as the about text.
    #[must_use]
    pub const fn expanded(mut self) -> Self {
        self.expand();
        self
    }

    /// Shows every section, for a way in that leads to something only the expanded window shows, such as the about
    /// text. An expanded window stays as it is.
    pub const fn expand(&mut self) {
        if !self.form.is_expanded() {
            self.form.toggle_expanded();
        }
    }

    /// The inner size of what the window shows now: the size to open it at, or to give it in another native window.
    #[must_use]
    pub const fn size(&self) -> egui::Vec2 {
        window_size(&self.form)
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
        // The bar comes first: a panel along an edge takes its room before the central panel fills the rest.
        let clicked = bottom_bar(ui, form);
        let frame = egui::Frame::central_panel(ui.style()).inner_margin(ui.spacing().window_margin);
        egui::CentralPanel::default()
            .frame(frame)
            .show(ui, |ui| contents(ui, form, &mut actions));
        if let Some(button) = clicked {
            press(ui, form, button, &mut actions);
        }
        if let Some(view) = form.take_save() {
            actions.insert(0, UiAction::Save(view));
        }
        actions
    }
}

#[cfg(test)]
mod tests;
