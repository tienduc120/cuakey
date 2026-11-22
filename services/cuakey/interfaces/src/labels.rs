//! Every string the settings window shows. The text is Vietnamese.

pub const WINDOW_TITLE: &str = concat!("CuaKey ", env!("CARGO_PKG_VERSION"), " - Bộ gõ Tiếng Việt");

pub const SECTION_CONTROL: &str = "Điều khiển";
pub const SECTION_TYPING: &str = "Bộ gõ";
pub const SECTION_MACROS: &str = "Gõ tắt";
pub const SECTION_SYSTEM: &str = "Hệ thống";
pub const SECTION_ABOUT: &str = "Thông tin";

pub const INPUT_METHOD: &str = "Kiểu gõ:";
pub const INPUT_METHOD_TELEX: &str = "Telex";
pub const INPUT_METHOD_VNI: &str = "VNI";
pub const INPUT_METHOD_SIMPLE_TELEX: &str = "Simple Telex";

pub const CODE_TABLE: &str = "Bảng mã:";
pub const CODE_TABLE_UNICODE: &str = "Unicode dựng sẵn";
pub const CODE_TABLE_TCVN3: &str = "TCVN3 (ABC)";
pub const CODE_TABLE_VNI_WINDOWS: &str = "VNI Windows";
pub const CODE_TABLE_UNICODE_COMPOUND: &str = "Unicode tổ hợp";
pub const CODE_TABLE_CP1258: &str = "Vietnamese locale CP 1258";

pub const SWITCH_HOTKEY: &str = "Phím chuyển:";
pub const BEEP: &str = "Kêu Beep";
pub const SETTINGS_HOTKEY: &str = "Phím mở bảng điều khiển:";

pub const LANGUAGE: &str = "Chế độ gõ:";
pub const LANGUAGE_VIETNAMESE: &str = "Tiếng Việt";
pub const LANGUAGE_ENGLISH: &str = "Tiếng Anh";

pub const NEW_TONE_STYLE: &str = "Đặt dấu oà, uý (thay vì òa, úy)";
pub const FIX_AUTOCOMPLETE: &str = "Sửa lỗi gợi ý (trình duyệt, Excel)";
pub const SPELL_CHECK: &str = "Kiểm tra chính tả";
pub const RESTORE_INVALID_WORD: &str = "Tự khôi phục phím với từ sai";
pub const ALLOW_ZWJF_CONSONANTS: &str = "Cho phép \"z w j f\" làm phụ âm đầu";
pub const CAPITALIZE_SENTENCES: &str = "Viết Hoa chữ cái đầu câu";
pub const SMART_SWITCH: &str = "Loại trừ ứng dụng thông minh";
pub const REMEMBER_CODE_TABLE: &str = "Tự ghi nhớ bảng mã theo ứng dụng";
pub const WORK_IN_OTHER_LANGUAGES: &str = "Cho phép gõ trong các ngôn ngữ khác";
pub const KEEP_W: &str = "Giữ nguyên chữ w ở đầu từ";
pub const ONLY_TEXT_BOXES: &str = "Chỉ gõ tiếng Việt trong ô nhập chữ";
pub const ALWAYS_ON_APPS: &str = "Ứng dụng luôn bật...";

pub const USE_MACROS: &str = "Cho phép gõ tắt";
pub const MACROS_IN_ENGLISH: &str = "Gõ tắt cả khi tắt gõ tiếng Việt";
pub const MACRO_MATCH_CASE: &str = "Tự động viết hoa theo phím tắt";
pub const MACRO_TABLE: &str = "Bảng gõ tắt...";
pub const QUICK_TELEX: &str = "Gõ nhanh (cc=ch, gg=gi, nn=ng,...)";
pub const QUICK_FIRST_CONSONANT: &str = "Gõ tắt phụ âm đầu: f→ph, j→gi, w→qu";
pub const QUICK_LAST_CONSONANT: &str = "Gõ tắt phụ âm cuối: g→ng, h→nh, k→ch";

pub const START_WITH_WINDOWS: &str = "Khởi động cùng Windows";
pub const SHOW_SETTINGS_AT_STARTUP: &str = "Bật bảng này khi khởi động";
pub const DESKTOP_SHORTCUT: &str = "Tạo biểu tượng trên Desktop";
pub const RUN_AS_ADMINISTRATOR: &str = "Chạy với quyền Admin";
pub const STORE_APP_SUPPORT: &str = "Hỗ trợ ứng dụng Metro";
pub const CHROMIUM_FIX: &str = "Sửa lỗi trên Chromium (beta)";
pub const CLIPBOARD_INPUT: &str = "Dùng Clipboard để gửi phím";

pub const ABOUT_NAME: &str = "CuaKey - Bộ gõ Tiếng Việt nguồn mở";
pub const ABOUT_VERSION: &str = concat!("Phiên bản ", env!("CARGO_PKG_VERSION"));
pub const ABOUT_LICENSE: &str = "Giấy phép MIT";
pub const ABOUT_URL: &str = "https://github.com/tienduc120/cuakey";
pub const ABOUT_CREDITS: &str = "Tham khảo hành vi gõ của OpenKey";

pub const BUTTON_EXIT: &str = "Tắt CuaKey";
pub const BUTTON_RESET: &str = "Cài đặt gốc";
pub const BUTTON_CLOSE: &str = "Đóng";

pub const KEY_CTRL: &str = "Ctrl";
pub const KEY_ALT: &str = "Alt";
pub const KEY_WIN: &str = "Win";
pub const KEY_SHIFT: &str = "Shift";
pub const HOTKEY_NO_KEY: &str = "Không có phím";
pub const HOTKEY_OTHER_KEY: &str = "Phím khác";
pub const HOTKEY_FIELD_HINT: &str = "Bấm vào rồi nhấn một phím; Backspace để bỏ phím";
