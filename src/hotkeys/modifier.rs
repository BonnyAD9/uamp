use global_hotkey::hotkey::Modifiers;

/// Converts modifiers to a string
pub fn get_modifier_string(m: &Modifiers) -> String {
    let mut res = String::new();

    if m.contains(Modifiers::ALT) {
        res += "alt+";
    }
    if m.contains(Modifiers::ALT_GRAPH) {
        res += "alt_gr+";
    }
    if m.contains(Modifiers::CAPS_LOCK) {
        res += "caps_lock+";
    }
    if m.contains(Modifiers::CONTROL) {
        res += "ctrl+";
    }
    if m.contains(Modifiers::FN) {
        res += "fn+";
    }
    if m.contains(Modifiers::FN_LOCK) {
        res += "fn_lock+";
    }
    if m.contains(Modifiers::META) {
        res += "os+";
    }
    if m.contains(Modifiers::NUM_LOCK) {
        res += "num_lock+";
    }
    if m.contains(Modifiers::SCROLL_LOCK) {
        res += "scr_lk+";
    }
    if m.contains(Modifiers::SHIFT) {
        res += "shift+";
    }
    if m.contains(Modifiers::SYMBOL) {
        res += "symbol+";
    }
    if m.contains(Modifiers::SYMBOL_LOCK) {
        res += "symbol_lock+";
    }
    if m.contains(Modifiers::HYPER) {
        res += "hyper+";
    }
    if m.contains(Modifiers::SUPER) {
        res += "super+";
    }

    res.pop();

    res
}

/// Parses string into modifiers
pub fn string_to_modifier(s: &str) -> Option<Modifiers> {
    Some(match s {
        "alt" | "l_alt" | "lalt" | "left_alt" | "leftalt" => Modifiers::ALT,
        "alt_gr" | "altgr" | "r_alt" | "ralt" | "right_alt" | "rightalt"
        | "altright" | "altgraph" => Modifiers::ALT_GRAPH,
        "caps_lock" | "capslock" => Modifiers::CAPS_LOCK,
        "ctrl" | "^" | "control" => Modifiers::CONTROL,
        "fn" => Modifiers::FN,
        "fn_lock" => Modifiers::FN_LOCK,
        "scr_lk" | "scrlk" | "scrolllock" => Modifiers::SCROLL_LOCK,
        "shift" => Modifiers::SHIFT,
        "symbol" => Modifiers::SYMBOL,
        "symbol_lock" => Modifiers::SYMBOL_LOCK,
        "hyper" => Modifiers::HYPER,
        "super" => Modifiers::SUPER,
        _ => return None,
    })
}
