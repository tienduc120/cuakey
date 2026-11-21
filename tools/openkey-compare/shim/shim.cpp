// MIT License, Copyright (c) 2026 Luu Tien Duc.
//
// Glue between the comparison tool and OpenKey's typing engine. The engine is compiled from sources fetched into
// target/openkey/ at build time and is never part of this repository; this file only calls it through its public
// header. The tool asks for one key at a time and gets back, in a plain C layout, what the host would do on screen.

#include <cstdint>

#include "Engine.h"

// The engine reads its settings from globals its host defines. The tool is the host here: these are its settings,
// set by shim_configure. Features the comparison leaves out (macros, smart switch, hotkeys) stay off.
int vLanguage = 1;
int vInputType = 0;
int vFreeMark = 0;
int vCodeTable = 0;
int vSwitchKeyStatus = 0;
int vCheckSpelling = 1;
int vUseModernOrthography = 0;
int vQuickTelex = 0;
int vRestoreIfWrongSpelling = 1;
int vFixRecommendBrowser = 0;
int vUseMacro = 0;
int vUseMacroInEnglishMode = 0;
int vAutoCapsMacro = 0;
int vUseSmartSwitchKey = 0;
int vUpperCaseFirstChar = 0;
int vTempOffSpelling = 0;
int vAllowConsonantZFWJ = 0;
int vQuickStartConsonant = 0;
int vQuickEndConsonant = 0;
int vRememberCode = 0;
int vOtherLanguage = 1;
int vTempOffOpenKey = 0;

namespace {

// The most characters one reply carries.
constexpr int kMaxSends = 32;

// The code tables, numbered as the engine numbers them.
constexpr int kUnicode = 0;
constexpr int kUnicodeCompound = 3;

vKeyHookState* reply_state = nullptr;

}  // namespace

extern "C" {

struct ShimSettings {
    int32_t input_method;  // 0 Telex, 1 VNI, 2 Simple Telex
    int32_t code_table;    // 0 Unicode, 1 TCVN3, 2 VNI Windows, 3 Unicode compound, 4 CP1258
    int32_t spell_check;
    int32_t restore;
    int32_t modern_tones;
    int32_t quick_telex;
    int32_t quick_first;
    int32_t quick_last;
    int32_t capitals;
};

// One character the host sends: one or two code units, or, when count is 0, the key `key` itself.
struct ShimSend {
    uint16_t units[2];
    uint16_t count;
    uint16_t key;
};

// What to do with one key. action: 0 let the key through, 1 delete then send, 2 delete, send, then type the key,
// 3 as 2 and the next key starts a new word, 4 a macro, -1 anything else. after: 1 the key ended the word, 2 it
// deleted a character, 3 it typed one, 0 none of these.
struct ShimReply {
    int32_t action;
    int32_t backspaces;
    int32_t after;
    int32_t count;
    ShimSend sends[kMaxSends];
};

}  // extern "C"

namespace {

void click() {
    vKeyHandleEvent(vKeyEvent::Mouse, vKeyEventState::MouseDown, 0);
}

// The units a single-byte table sends for `code`: its low byte, then its high byte when there is one.
ShimSend single_byte_send(uint16_t code) {
    ShimSend send{};
    send.units[0] = static_cast<uint16_t>(code & 0xff);
    send.count = 1;
    const auto high = static_cast<uint16_t>(code >> 8);
    if (high > 32) {
        send.units[1] = high;
        send.count = 2;
    }
    return send;
}

// Unicode compound packs the tone in the top three bits: the letter, then the tone's combining mark.
ShimSend compound_send(uint16_t code) {
    ShimSend send{};
    send.units[0] = static_cast<uint16_t>(code & 0x1fff);
    send.count = 1;
    const auto tone = static_cast<uint16_t>(code >> 13);
    if (tone > 0) {
        send.units[1] = _unicodeCompoundMark[tone - 1];
        send.count = 2;
    }
    return send;
}

// How the host sends one character of the engine's reply in the current code table.
ShimSend send_of(Uint32 datum) {
    ShimSend send{};
    if ((datum & CHAR_CODE_MASK) == 0) {
        const Uint16 typed = keyCodeToCharacter(datum);
        if (typed == 0) {
            send.key = static_cast<uint16_t>(datum & 0xffff);
        } else {
            send.units[0] = typed;
            send.count = 1;
        }
        return send;
    }
    const auto code = static_cast<uint16_t>(datum & 0xffff);
    if (vCodeTable == kUnicode) {
        send.units[0] = code;
        send.count = 1;
        return send;
    }
    return vCodeTable == kUnicodeCompound ? compound_send(code) : single_byte_send(code);
}

int32_t action_of(Byte code) {
    switch (code) {
        case vDoNothing:
            return 0;
        case vWillProcess:
            return 1;
        case vRestore:
            return 2;
        case vRestoreAndStartNewSession:
            return 3;
        case vReplaceMaro:
            return 4;
        default:
            return -1;
    }
}

}  // namespace

extern "C" {

// A click, a letter and another click: the engine forgets the word, the words before it, and the spaces after them,
// as it was when it started.
void shim_start_line() noexcept {
    click();
    click();
    vKeyHandleEvent(vKeyEvent::Keyboard, vKeyEventState::KeyDown, KEY_A, 0, false);
    click();
    click();
}

void shim_configure(const ShimSettings* settings) noexcept {
    vInputType = settings->input_method;
    vCodeTable = settings->code_table;
    vCheckSpelling = settings->spell_check;
    vRestoreIfWrongSpelling = settings->restore;
    vUseModernOrthography = settings->modern_tones;
    vQuickTelex = settings->quick_telex;
    vQuickStartConsonant = settings->quick_first;
    vQuickEndConsonant = settings->quick_last;
    vUpperCaseFirstChar = settings->capitals;
    // Initialising again makes the engine take the new spell check setting as the one it returns to.
    reply_state = static_cast<vKeyHookState*>(vKeyInit());
    shim_start_line();
}

// One key pressed: `vk` is its Windows virtual-key code; caps_status is 1 for Shift, 2 for Caps Lock, 0 for neither
// or both; `control` is non-zero while Ctrl or Alt is held.
void shim_key(uint16_t vk, uint8_t caps_status, int32_t control, ShimReply* reply) noexcept {
    vKeyHandleEvent(vKeyEvent::Keyboard, vKeyEventState::KeyDown, vk, caps_status, control != 0);
    reply->action = action_of(reply_state->code);
    reply->backspaces = reply_state->backspaceCount;
    reply->after = reply_state->extCode <= 3 ? reply_state->extCode : 0;
    const int count = reply_state->newCharCount <= kMaxSends ? reply_state->newCharCount : 0;
    reply->count = count;
    // The reply lists the characters last first; they are sent first to last.
    for (int sent = 0; sent < count; ++sent) {
        reply->sends[sent] = send_of(reply_state->charData[count - 1 - sent]);
    }
}

// One mouse button event; a click is two, the button going down and coming up.
void shim_click() noexcept {
    click();
}

void shim_new_session() noexcept {
    startNewSession();
}

// How the host sends the key `vk` itself after a reply, in upper case when `caps` is non-zero.
void shim_key_send(uint16_t vk, int32_t caps, ShimSend* send) noexcept {
    *send = send_of(vk | (caps != 0 ? CAPS_MASK : 0));
}

}  // extern "C"
