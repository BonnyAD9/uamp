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
    WB_SETTINGS,
    WB_STATE_COUNT
];
