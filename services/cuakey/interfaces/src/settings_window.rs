//! The settings window drawn with egui. It opens small, with the controls used every day, and the expand button in the
//! bottom bar shows the other sections. Every change applies at once: a frame that changed a control returns one
//! `Save` with the whole edited view. What is shown and enabled lives in [`SettingsForm`]; this file only draws it, in
//! the interface language of the view it edits.

use cuakey_application::{
    CodeTableValue, InputMethodValue, InterfaceLanguageValue, LanguageValue, PreferencesView, Toggle,
};
use egui::Ui;

use crate::hotkey_editor::show_hotkey;
use crate::labels::{Labels, labels_for};
use crate::settings_form::{ButtonBar, GatedControl, SettingsForm, is_on};
use crate::theme::BUTTON_SIZE;
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

/// The window's inner size with the controls used every day only, the size it opens at. With the shared style, the
/// rows measure at most 474 x 223 points in Segoe UI and the other fonts the window may borrow, in either interface
/// language, 250 high with a line for an error message; the margins and the bottom bar come on top, so nothing scrolls
/// or clips. The width is the expanded window's, so expanding only moves the bottom edge and the bar's buttons stay
/// where they were.
pub const COMPACT_SIZE: egui::Vec2 = egui::vec2(560.0, 340.0);

/// The window's inner size with every section. The options sit two to a row, and the widest needs 257 points in either
/// interface language, so two columns fit in this width. The sections are taller than a small screen, so they scroll
/// above the bottom bar; this height shows the controls used every day and the whole first section.
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

const SECTIONS: [Section; 3] = [
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
    Section {
        id: "system_options",
        title: |text| text.section_system,
        options: &SYSTEM_OPTIONS,
        button: None,
    },
];

/// Draws `option` as a checkbox, greyed when the form's rule says so, and stores a click.
fn option_checkbox(ui: &mut Ui, form: &mut SettingsForm, option: &OptionBox) {
    let enabled = option.gate.is_none_or(|gate| form.is_enabled(gate));
    let current = (option.value)(form.view());
    let label = (option.label)(labels_for(form.view()));
    if let Some(on) = ui.add_enabled_ui(enabled, |ui| checkbox(ui, label, current)).inner {
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

/// The about text, the last section. When `about_to_reveal` asks for it, the scroll area brings it into view once.
fn about_section(ui: &mut Ui, text: &Labels, about_to_reveal: &mut bool) {
    let title = section_title(ui, text.section_about);
    ui.label(text.about_name);
    ui.label(text.about_version);
    ui.label(text.about_license);
    ui.hyperlink(text.about_url);
    ui.label(text.about_credits);
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

/// The controls used every day, as grid cells: a label, then its control. Each hotkey editor sits in one cell and
/// draws the same parts in the same order, so the modifier boxes and key fields of the two hotkey rows line up.
fn control_rows(ui: &mut Ui, form: &mut SettingsForm) {
    let text = labels_for(form.view());
    let interface_language = form.view().system.interface_language;
    ui.label(text.input_method);
    if let Some(method) = combo(
        ui,
        "input_method",
        form.view().typing.input_method,
        &input_methods(text),
    ) {
        form.set_input_method(method);
    }
    ui.end_row();

    ui.label(text.code_table);
    if let Some(table) = combo(ui, "code_table", form.view().typing.code_table, &code_tables(text)) {
        form.set_code_table(table);
    }
    ui.end_row();

    ui.label(text.switch_hotkey);
    if let Some(hotkey) = show_hotkey(ui, "switch_hotkey", form.view().switching.hotkey, interface_language) {
        form.set_switch_hotkey(hotkey);
    }
    option_checkbox(ui, form, &BEEP);
    ui.end_row();

    ui.label(text.settings_hotkey);
    let settings_hotkey = form.view().behaviours.settings_hotkey;
    if let Some(hotkey) = show_hotkey(ui, "settings_hotkey", settings_hotkey, interface_language) {
        form.set_settings_hotkey(hotkey);
    }
    ui.end_row();

    ui.label(text.language);
    if let Some(language) = radios(ui, form.view().language, &typing_languages(text)) {
        form.set_language(language);
    }
    ui.end_row();

    ui.label(text.interface_language);
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
    let label = button_label(labels_for(form.view()), button, form.is_expanded());
    ui.add_sized(size, egui::Button::new(label)).clicked()
}

/// The buttons of the bottom bar, where the form puts them: the left one at the left edge, the others ending at the
/// right edge. They are added left to right, so the keyboard reaches them in reading order. Returns the button clicked.
fn bar_buttons(ui: &mut Ui, form: &SettingsForm) -> Option<Button> {
    let bar = form.bar();
    let size = egui::vec2(
        bar_button_width(ui, &bar_labels(labels_for(form.view()))),
        BUTTON_SIZE.y,
    );
    ui.horizontal(|ui| {
        let mut clicked = None;
        if let Some(button) = bar.left
            && bar_button(ui, form, button, size)
        {
            clicked = Some(button);
        }
        let spacing = ui.spacing().item_spacing.x;
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

/// What the window shows, then the bar pinned along its bottom edge, outside the scrolling sections. The bar is drawn
/// last so the keyboard reaches the controls first and the reset last. Returns the button clicked.
fn window_contents(
    ui: &mut Ui,
    form: &mut SettingsForm,
    about_to_reveal: &mut bool,
    actions: &mut Vec<UiAction>,
) -> Option<Button> {
    let (above, bar) = split_off_bar(ui);
    ui.scope_builder(egui::UiBuilder::new().max_rect(above), |ui| {
        ui.shrink_clip_rect(above);
        contents(ui, form, about_to_reveal, actions);
    });
    ui.scope_builder(egui::UiBuilder::new().max_rect(bar), |ui| bar_buttons(ui, form))
        .inner
}

/// The controls used every day, the message of the last refused change under them, and, when expanded, the other
/// sections in a scroll area below. In compact mode nothing scrolls.
fn contents(ui: &mut Ui, form: &mut SettingsForm, about_to_reveal: &mut bool, actions: &mut Vec<UiAction>) {
    egui::Grid::new("control_rows")
        .num_columns(3)
        .show(ui, |ui| control_rows(ui, form));
    if let Some(message) = form.error() {
        ui.colored_label(ui.visuals().error_fg_color, message);
    }
    if form.is_expanded() {
        egui::ScrollArea::vertical().show(ui, |ui| expanded_sections(ui, form, about_to_reveal, actions));
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
    /// The about text is to be scrolled into view on the next frame.
    about_to_reveal: bool,
}

impl SettingsWindow {
    #[must_use]
    pub const fn new(prefs: PreferencesView, platform: Platform) -> Self {
        Self {
            form: SettingsForm::new(prefs, platform),
            about_to_reveal: false,
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
        let frame = egui::Frame::central_panel(ui.style()).inner_margin(ui.spacing().window_margin);
        let clicked = egui::CentralPanel::default()
            .frame(frame)
            .show(ui, |ui| window_contents(ui, form, about_to_reveal, &mut actions))
            .inner;
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
