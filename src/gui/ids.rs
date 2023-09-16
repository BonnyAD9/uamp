/// Can be used to generate consecutive ids
macro_rules! make_ids {
    ($($i:ident),+ $(,)?) => {
        count_macro::count!{$(pub const $i: usize = _int_;)+}
    };
}

// Wrapbox state ids
make_ids![
    WB_SONGS,
    WB_PLAYLIST,
    WB_SETTINGS_LIBRARY,
    WB_SETTINGS_PLAYBACK,
    WB_SETTINGS_SERVER,
    WB_SETTINGS_HOTKEYS,
    WB_SETTINGS_OTHER,
    WB_SETTINGS_HELP,
    WB_STATE_COUNT
];
