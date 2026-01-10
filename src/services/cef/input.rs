use cef::ImplBrowserHost;
use cef_dll_sys::cef_key_event_type_t;

pub fn key_to_windows_code(key: &str) -> Option<i32> {
    match key {
        "backspace" => Some(8),
        "tab" => Some(9),
        "enter" => Some(13),
        "escape" => Some(27),
        "space" => Some(32),
        "left" => Some(37),
        "up" => Some(38),
        "right" => Some(39),
        "down" => Some(40),
        "delete" => Some(46),
        "home" => Some(36),
        "end" => Some(35),
        "pageup" => Some(33),
        "pagedown" => Some(34),
        _ if key.len() == 1 => {
            let c = key.chars().next().unwrap();
            if c.is_ascii_alphabetic() {
                Some(c.to_ascii_uppercase() as i32)
            } else if c.is_ascii_digit() {
                Some(c as i32)
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn modifiers_to_cef(modifiers: &gpui::Modifiers) -> u32 {
    let mut flags = 0u32;
    if modifiers.control { flags |= 0x0004; }
    if modifiers.shift { flags |= 0x0008; }
    if modifiers.alt { flags |= 0x0010; }
    flags
}

pub fn send_key_event(host: &cef::BrowserHost, key_code: i32, modifiers: u32, down: bool) {
    let event = cef::KeyEvent {
        type_: if down {
            cef_key_event_type_t::KEYEVENT_KEYDOWN.into()
        } else {
            cef_key_event_type_t::KEYEVENT_KEYUP.into()
        },
        modifiers,
        windows_key_code: key_code,
        native_key_code: key_code,
        size: std::mem::size_of::<cef::KeyEvent>(),
        ..Default::default()
    };
    host.send_key_event(Some(&event));
}

pub fn send_char_event(host: &cef::BrowserHost, ch: char, modifiers: u32) {
    let event = cef::KeyEvent {
        type_: cef_key_event_type_t::KEYEVENT_CHAR.into(),
        character: ch as u16,
        modifiers,
        size: std::mem::size_of::<cef::KeyEvent>(),
        ..Default::default()
    };
    host.send_key_event(Some(&event));
}

pub const SCROLL_MULTIPLIER: i32 = 53;
