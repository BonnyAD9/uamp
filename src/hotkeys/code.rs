use global_hotkey::hotkey::Code;

/// Translates key code to a string
pub fn get_code_string(code: &Code) -> &'static str {
    match code {
        Code::Backquote => "`",
        Code::Backslash => "\\",
        Code::BracketLeft => "[",
        Code::BracketRight => "]",
        Code::Comma => ",",
        Code::Digit0 => "0",
        Code::Digit1 => "1",
        Code::Digit2 => "2",
        Code::Digit3 => "3",
        Code::Digit4 => "4",
        Code::Digit5 => "5",
        Code::Digit6 => "6",
        Code::Digit7 => "7",
        Code::Digit8 => "8",
        Code::Digit9 => "9",
        Code::Equal => "=",
        Code::IntlBackslash => "l\\",
        Code::IntlRo => "ro",
        Code::IntlYen => "yen",
        Code::KeyA => "A",
        Code::KeyB => "B",
        Code::KeyC => "C",
        Code::KeyD => "D",
        Code::KeyE => "E",
        Code::KeyF => "F",
        Code::KeyG => "G",
        Code::KeyH => "H",
        Code::KeyI => "I",
        Code::KeyJ => "J",
        Code::KeyK => "K",
        Code::KeyL => "L",
        Code::KeyM => "M",
        Code::KeyN => "N",
        Code::KeyO => "O",
        Code::KeyP => "P",
        Code::KeyQ => "Q",
        Code::KeyR => "R",
        Code::KeyS => "S",
        Code::KeyT => "T",
        Code::KeyU => "U",
        Code::KeyV => "V",
        Code::KeyW => "W",
        Code::KeyX => "X",
        Code::KeyY => "Y",
        Code::KeyZ => "Z",
        Code::Minus => "-",
        Code::Period => ".",
        Code::Quote => "'",
        Code::Semicolon => ";",
        Code::Slash => "/",
        Code::AltLeft => "alt",
        Code::AltRight => "alt_gr",
        Code::Backspace => "backspace",
        Code::CapsLock => "caps_lock",
        Code::ContextMenu => "context",
        Code::ControlLeft => "ctrl",
        Code::ControlRight => "r_ctrl",
        Code::Enter => "enter",
        Code::MetaLeft => "os",
        Code::MetaRight => "r_os",
        Code::ShiftLeft => "shift",
        Code::ShiftRight => "r_shift",
        Code::Space => "space",
        Code::Tab => "tab",
        Code::Convert => "convert",
        Code::KanaMode => "kana_mode",
        Code::Lang1 => "hangul_mode",
        Code::Lang2 => "hanja",
        Code::Lang3 => "katakana",
        Code::Lang4 => "hiragana",
        Code::Lang5 => "lang5",
        Code::NonConvert => "non_convert",
        Code::Delete => "del",
        Code::End => "end",
        Code::Help => "help",
        Code::Home => "home",
        Code::Insert => "ins",
        Code::PageDown => "pg_dn",
        Code::PageUp => "pg_up",
        Code::ArrowDown => "down",
        Code::ArrowLeft => "left",
        Code::ArrowRight => "right",
        Code::ArrowUp => "up",
        Code::NumLock => "num_lock",
        Code::Numpad0 => "num0",
        Code::Numpad1 => "num1",
        Code::Numpad2 => "num2",
        Code::Numpad3 => "num3",
        Code::Numpad4 => "num4",
        Code::Numpad5 => "num5",
        Code::Numpad6 => "num6",
        Code::Numpad7 => "num7",
        Code::Numpad8 => "num8",
        Code::Numpad9 => "num9",
        Code::NumpadAdd => "num_add",
        Code::NumpadBackspace => "num_backspace",
        Code::NumpadClear => "num_ac",
        Code::NumpadClearEntry => "num_ce",
        Code::NumpadComma => "num,",
        Code::NumpadDecimal => "num.",
        Code::NumpadDivide => "num/",
        Code::NumpadEnter => "num_enter",
        Code::NumpadEqual => "num=",
        Code::NumpadHash => "num#",
        Code::NumpadMemoryAdd => "num_madd",
        Code::NumpadMemoryClear => "num_mc",
        Code::NumpadMemoryRecall => "num_mr",
        Code::NumpadMemoryStore => "num_ms",
        Code::NumpadMemorySubtract => "num_m-",
        Code::NumpadMultiply => "num*",
        Code::NumpadParenLeft => "num(",
        Code::NumpadParenRight => "num)",
        Code::NumpadStar => "num_star",
        Code::NumpadSubtract => "num-",
        Code::Escape => "esc",
        Code::F1 => "f1",
        Code::F2 => "f2",
        Code::F3 => "f3",
        Code::F4 => "f4",
        Code::F5 => "f5",
        Code::F6 => "f6",
        Code::F7 => "f7",
        Code::F8 => "f8",
        Code::F9 => "f9",
        Code::F10 => "f10",
        Code::F11 => "f11",
        Code::F12 => "f12",
        Code::Fn => "fn",
        Code::FnLock => "fn_lock",
        Code::PrintScreen => "prt_scr",
        Code::ScrollLock => "scr_lk",
        Code::Pause => "pause",
        Code::BrowserBack => "browser_back",
        Code::BrowserFavorites => "browser_favorites",
        Code::BrowserForward => "browser_forward",
        Code::BrowserHome => "browser_home",
        Code::BrowserRefresh => "browser_refresh",
        Code::BrowserSearch => "browser_search",
        Code::BrowserStop => "browser_stop",
        Code::Eject => "eject",
        Code::LaunchApp1 => "launch_app1",
        Code::LaunchApp2 => "launch_app1",
        Code::LaunchMail => "launch_mail",
        Code::MediaPlayPause => "media_play_pause",
        Code::MediaSelect => "media_select",
        Code::MediaStop => "media_stop",
        Code::MediaTrackNext => "media_next",
        Code::MediaTrackPrevious => "media_prev",
        Code::Power => "power",
        Code::Sleep => "sleep",
        Code::AudioVolumeDown => "vol_down",
        Code::AudioVolumeMute => "mute",
        Code::AudioVolumeUp => "vol_up",
        Code::WakeUp => "wake",
        Code::Hyper => "hyper",
        Code::Super => "super",
        Code::Turbo => "turbo",
        Code::Abort => "abort",
        Code::Resume => "resume",
        Code::Suspend => "suspend",
        Code::Again => "again",
        Code::Copy => "copy",
        Code::Cut => "cut",
        Code::Find => "find",
        Code::Open => "open",
        Code::Paste => "paste",
        Code::Props => "props",
        Code::Select => "select",
        Code::Undo => "undo",
        Code::Hiragana => "hiragana",
        Code::Katakana => "katakana",
        Code::Unidentified => "unident",
        Code::F13 => "f13",
        Code::F14 => "f14",
        Code::F15 => "f15",
        Code::F16 => "f16",
        Code::F17 => "f17",
        Code::F18 => "f18",
        Code::F19 => "f19",
        Code::F20 => "f20",
        Code::F21 => "f21",
        Code::F22 => "f22",
        Code::F23 => "f23",
        Code::F24 => "f24",
        Code::BrightnessDown => "bright_down",
        Code::BrightnessUp => "bright_up",
        Code::DisplayToggleIntExt => "display_toggle_int_ext",
        Code::KeyboardLayoutSelect => "keyboard_layout_sel",
        Code::LaunchAssistant => "launch_assist",
        Code::LaunchControlPanel => "launch_control_panel",
        Code::LaunchScreenSaver => "launch_screen_saver",
        Code::MailForward => "mail_forward",
        Code::MailReply => "mail_reply",
        Code::MailSend => "mail_send",
        Code::MediaFastForward => "media_fast_forward",
        Code::MediaPause => "media_pause",
        Code::MediaPlay => "media_play",
        Code::MediaRecord => "media_record",
        Code::MediaRewind => "media_rewind",
        Code::MicrophoneMuteToggle => "mic_mute_toggle",
        Code::PrivacyScreenToggle => "privacy_screen_toggle",
        Code::SelectTask => "sel_task",
        Code::ShowAllWindows => "show_all_windows",
        Code::ZoomToggle => "zoom_toggle",
    }
}

/// Translates key code string to Code
pub fn string_to_code(s: &str) -> Option<Code> {
    Some(match s {
        "`" | "`~" | "backquote" => Code::Backquote,
        "\\" | "|" | "#~" | "backslash" => Code::Backslash,
        "[" | "[{" | "bracket_left" | "bracketleft" => Code::BracketLeft,
        "]" | "}]" | "bracket_right" | "bracketright" => Code::BracketRight,
        "," | ",<" | "comma" => Code::Comma,
        "0" | "0)" | "digit0" | "digit_0" => Code::Digit0,
        "1" | "1!" | "digit1" | "digit_1" => Code::Digit1,
        "2" | "2@" | "digit2" | "digit_2" => Code::Digit2,
        "3" | "3#" | "digit3" | "digit_3" => Code::Digit3,
        "4" | "4$" | "digit4" | "digit_4" => Code::Digit4,
        "5" | "5%" | "digit5" | "digit_5" => Code::Digit5,
        "6" | "6^" | "digit6" | "digit_6" => Code::Digit6,
        "7" | "7&" | "digit7" | "digit_7" => Code::Digit7,
        "8" | "8*" | "digit8" | "digit_8" => Code::Digit8,
        "9" | "9(" | "digit9" | "digit_9" => Code::Digit9,
        "=" | "=plus" | "equal" => Code::Equal,
        "l\\" | "\\|" | "l_\\" | "intlbackslash" => Code::IntlBackslash,
        "ro" | "intl_ro" | "intlro" => Code::IntlRo,
        "yen" | "¥" | "intl_yen" | "intlyen" => Code::IntlYen,
        "a" | "key_a" | "keya" => Code::KeyA,
        "b" | "key_b" | "keyb" => Code::KeyB,
        "c" | "key_c" | "keyc" => Code::KeyC,
        "d" | "key_d" | "keyd" => Code::KeyD,
        "e" | "key_e" | "keye" => Code::KeyE,
        "f" | "key_f" | "keyf" => Code::KeyF,
        "g" | "key_g" | "keyg" => Code::KeyG,
        "h" | "key_h" | "keyh" => Code::KeyH,
        "i" | "key_i" | "keyi" => Code::KeyI,
        "j" | "key_j" | "keyj" => Code::KeyJ,
        "k" | "key_k" | "keyk" => Code::KeyK,
        "l" | "key_l" | "keyl" => Code::KeyL,
        "m" | "key_m" | "keym" => Code::KeyM,
        "n" | "key_n" | "keyn" => Code::KeyN,
        "o" | "key_o" | "keyo" => Code::KeyO,
        "p" | "key_p" | "keyp" => Code::KeyP,
        "q" | "key_q" | "keyq" => Code::KeyQ,
        "r" | "key_r" | "keyr" => Code::KeyR,
        "s" | "key_s" | "keys" => Code::KeyS,
        "t" | "key_t" | "keyt" => Code::KeyT,
        "u" | "key_u" | "keyu" => Code::KeyU,
        "v" | "key_v" | "keyv" => Code::KeyV,
        "w" | "key_w" | "keyw" => Code::KeyW,
        "x" | "key_x" | "keyx" => Code::KeyX,
        "y" | "key_y" | "keyy" => Code::KeyY,
        "z" | "key_z" | "keyz" => Code::KeyZ,
        "_" | "__" | "minus" => Code::Minus,
        "." | ".>" | "dot" | "period" => Code::Period,
        "'" | "'\"" | "quote" => Code::Quote,
        ";" | ":;" | "semicolon" => Code::Semicolon,
        "/" | "/?" | "slash" => Code::Slash,
        "alt" | "l_alt" | "lalt" | "left_alt" | "leftalt" => Code::AltLeft,
        "alt_gr" | "altgr" | "r_alt" | "ralt" | "right_alt" | "rightalt"
        | "altright" | "altgraph" => Code::AltRight,
        "backspace" => Code::Backspace,
        "caps_lock" | "capslock" => Code::CapsLock,
        "context" | "context_menu" | "contextmenu" => Code::ContextMenu,
        "ctrl" | "^" | "lctrl" | "controlleft" | "control" => {
            Code::ControlLeft
        }
        "r_ctrl" | "rctrl" | "controlright" => Code::ControlRight,
        "enter" | "return" => Code::Enter,
        "os" | "l_os" | "los" | "metaleft" => Code::MetaLeft,
        "r_os" | "ros" | "metaright" => Code::MetaRight,
        "shift" | "l_shift" | "lshift" | "shiftleft" => Code::ShiftLeft,
        "r_shift" | "rshift" | "shiftright" => Code::ShiftRight,
        "space" => Code::Space,
        "tab" | "tabulator" => Code::Tab,
        "convert" => Code::Convert,
        "kana_mode" | "kanamode" => Code::KanaMode,
        "lang1" | "lang_1" => Code::Lang1,
        "lang2" | "lang_2" => Code::Lang2,
        "lang3" | "lang_3" => Code::Lang3,
        "lang4" | "lang_4" => Code::Lang4,
        "lang5" | "lang_5" => Code::Lang5,
        "non_convert" | "nonconvert" => Code::NonConvert,
        "del" | "delete" => Code::Delete,
        "end" => Code::End,
        "help" => Code::Help,
        "home" => Code::Home,
        "ins" | "insert" => Code::Insert,
        "pg_dn" | "pgdn" | "pg_down" | "pgdown" | "page_down" | "pagedown" => {
            Code::PageDown
        }
        "pg_up" | "pgup" | "page_up" | "pageup" => Code::PageUp,
        "down" | "arrow_down" | "arrowdown" => Code::ArrowDown,
        "left" | "arrow_left" | "arrowleft" => Code::ArrowLeft,
        "right" | "arrow_right" | "arrowright" => Code::ArrowRight,
        "up" | "arrow_up" | "arrowup" => Code::ArrowUp,
        "num_lock" | "numlock" => Code::NumLock,
        "num0" | "num_0" | "numpad0" | "numpad_0" => Code::Numpad0,
        "num1" | "num_1" | "numpad1" | "numpad_1" => Code::Numpad1,
        "num2" | "num_2" | "numpad2" | "numpad_2" => Code::Numpad2,
        "num3" | "num_3" | "numpad3" | "numpad_3" => Code::Numpad3,
        "num4" | "num_4" | "numpad4" | "numpad_4" => Code::Numpad4,
        "num5" | "num_5" | "numpad5" | "numpad_5" => Code::Numpad5,
        "num6" | "num_6" | "numpad6" | "numpad_6" => Code::Numpad6,
        "num7" | "num_7" | "numpad7" | "numpad_7" => Code::Numpad7,
        "num8" | "num_8" | "numpad8" | "numpad_8" => Code::Numpad8,
        "num9" | "num_9" | "numpad9" | "numpad_9" => Code::Numpad9,
        "num_add" | "numadd" | "numpadadd" => Code::NumpadAdd,
        "num_backspace" | "numbackspace" | "numpadbackspace" => {
            Code::NumpadBackspace
        }
        "num_ac" | "numac" | "numpadclear" => Code::NumpadClear,
        "num_ce" | "numce" | "numpadclearentry" => Code::NumpadClearEntry,
        "num," | "numpadcomma" => Code::NumpadComma,
        "num." | "numpaddecimal" => Code::NumpadDecimal,
        "num/" | "numpaddivide" => Code::NumpadDivide,
        "num_enter" | "numenter" | "numpadenter" => Code::NumpadEnter,
        "num=" | "numpadequal" => Code::NumpadEqual,
        "num#" | "numpadhash" => Code::NumpadHash,
        "num_madd" | "nummadd" | "numpadmemoryadd" => Code::NumpadMemoryAdd,
        "num_mc" | "nummc" | "numpadmemoryclear" => Code::NumpadMemoryClear,
        "num_mr" | "nummr" | "numpadmemoryrecall" => Code::NumpadMemoryRecall,
        "num_ms" | "numms" | "numpadmemorystore" => Code::NumpadMemoryStore,
        "num_m_" | "numm_" | "numpadmemorysubtract" => {
            Code::NumpadMemorySubtract
        }
        "num*" | "numpadmultiply" => Code::NumpadMultiply,
        "num(" | "numpadparenleft" => Code::NumpadParenLeft,
        "num)" | "numpadparenright" => Code::NumpadParenRight,
        "num_star" | "numstar" | "numpadstar" => Code::NumpadStar,
        "num_" | "numpadsubtract" => Code::NumpadSubtract,
        "esc" | "escape" => Code::Escape,
        "f1" => Code::F1,
        "f2" => Code::F2,
        "f3" => Code::F3,
        "f4" => Code::F4,
        "f5" => Code::F5,
        "f6" => Code::F6,
        "f7" => Code::F7,
        "f8" => Code::F8,
        "f9" => Code::F9,
        "f11" => Code::F11,
        "f12" => Code::F12,
        "f13" => Code::F13,
        "f14" => Code::F14,
        "f15" => Code::F15,
        "f16" => Code::F16,
        "f17" => Code::F17,
        "f18" => Code::F18,
        "f19" => Code::F19,
        "f20" => Code::F20,
        "f21" => Code::F21,
        "f22" => Code::F22,
        "f23" => Code::F23,
        "f24" => Code::F24,
        "fn" => Code::Fn,
        "fn_lock" | "fnlock" => Code::FnLock,
        "prt_scr" | "prtscr" => Code::PrintScreen,
        "scr_lk" | "scrlk" | "scrolllock" => Code::ScrollLock,
        "pause" => Code::Pause,
        "browser_back" | "browserback" => Code::BrowserBack,
        "browser_favorites" | "browserfavorites" => Code::BrowserFavorites,
        "browser_forward" | "browserforward" => Code::BrowserForward,
        "browser_home" | "browserhome" => Code::BrowserHome,
        "browser_refresh" | "browserrefresh" => Code::BrowserRefresh,
        "browser_search" | "browsersearch" => Code::BrowserSearch,
        "browser_stop" | "browserstop" => Code::BrowserStop,
        "eject" => Code::Eject,
        "launch_app1" | "launchapp1" => Code::LaunchApp1,
        "launch_app2" | "launchapp2" => Code::LaunchApp2,
        "launch_mail" | "launchmail" => Code::LaunchMail,
        "media_play_pause" | "mediaplaypause" => Code::MediaPlayPause,
        "media_select" | "mediaselect" => Code::MediaSelect,
        "media_stop" | "mediastop" => Code::MediaStop,
        "media_next" | "medianext" | "mediatracknext" => Code::MediaTrackNext,
        "media_prev" | "mediaprev" | "mediatrackprevious" => {
            Code::MediaTrackPrevious
        }
        "power" => Code::Power,
        "sleep" => Code::Sleep,
        "vol_down" | "voldown" | "audiovolumedown" => Code::AudioVolumeDown,
        "mute" | "audiovolmuemute" => Code::AudioVolumeMute,
        "vol_up" | "volup" => Code::AudioVolumeUp,
        "wake" | "wakeup" => Code::WakeUp,
        "hyper" => Code::Hyper,
        "super" => Code::Super,
        "turbo" => Code::Turbo,
        "abort" => Code::Abort,
        "resume" => Code::Resume,
        "suspend" => Code::Suspend,
        "again" => Code::Again,
        "copy" => Code::Copy,
        "cut" => Code::Cut,
        "find" => Code::Find,
        "open" => Code::Open,
        "paste" => Code::Paste,
        "props" => Code::Props,
        "select" => Code::Select,
        "undo" => Code::Undo,
        "hiragana" => Code::Hiragana,
        "katakana" => Code::Katakana,
        "unident" | "unidentified" => Code::Unidentified,
        "bright_down" | "brightdown" | "brightnessdown" => {
            Code::BrightnessDown
        }
        "bright_up" | "brightup" | "brightnessup" => Code::BrightnessUp,
        "display_toggle_int_ext" | "displaytoggleintext" => {
            Code::DisplayToggleIntExt
        }
        "keyboard_layout_sel" | "keybordlayoutselect" => {
            Code::KeyboardLayoutSelect
        }
        "launch_assistant" | "launchassistant" => Code::LaunchAssistant,
        "launch_control_panel" | "launchcontrolpanel" => {
            Code::LaunchControlPanel
        }
        "launch_screen_saver" | "launchscreensaver" => Code::LaunchScreenSaver,
        "mail_forward" | "mailforward" => Code::MailForward,
        "mail_reply" | "mailreply" => Code::MailReply,
        "mail_send" | "mailsend" => Code::MailSend,
        "media_fast_forward" | "mediafastforward" => Code::MediaFastForward,
        "media_pause" | "mediapause" => Code::MediaPause,
        "media_record" | "mediarecord" => Code::MediaRecord,
        "media_rewind" | "mediarewind" => Code::MediaRewind,
        "mic_mute_toggle" | "micmutetoggle" | "microphonemutetoggle" => {
            Code::MicrophoneMuteToggle
        }
        "privacy_screen_toggle" | "privacyscreentoggle" => {
            Code::PrivacyScreenToggle
        }
        "sel_task" | "seltask" | "selecttask" => Code::SelectTask,
        "show_all_windows" | "showallwindows" => Code::ShowAllWindows,
        "zoom_toggle" | "zoomtoggle" => Code::ZoomToggle,
        _ => return None,
    })
}
