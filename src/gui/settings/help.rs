use iced_core::{
    alignment::Vertical,
    Length::{Fill, Shrink},
};

use crate::{
    col,
    gui::wid::{column, container, line_text, text, Element},
    row,
};

#[derive(Debug)]
pub struct SetHelp {
    title: &'static str,
    json_field_name: Option<&'static str>,
    value_type: Option<&'static str>,
    default_value: Option<&'static str>,
    description: &'static str,
}

pub const SEARCH_FOR_NEW_SONGS: SetHelp = SetHelp {
    title: "Search for new songs",
    json_field_name: None,
    value_type: None,
    default_value: None,
    description:
        "This will go trough all the files in the folders specified in \
'Library search paths'. It will examine only files with the \
extensions specified in 'Song extensions'.

If any of the files is not in the library, it will try to load its \
metadata into the library.

This will run on separate thread, so that the gui doesn't freeze.",
};

pub const RECURSIVE_SEARCH_FOR_NEW_SONGS: SetHelp = SetHelp {
    title: "Recursive search for new songs",
    json_field_name: Some("recursive_search"),
    value_type: Some("bool"),
    default_value: Some("true"),
    description: "When enabled, 'Search for new songs' will search \
recursively. That means that instead of searching only in the immidiate \
folder specified in 'Library search paths', it will also search in all \
subfolders, and their subfolders, and their subfolders...

Dont worry the algorithm is not implemented recursively :).",
};

pub const UPDATE_LIBRARY_ON_START: SetHelp = SetHelp {
    title: "Update library on start",
    json_field_name: Some("update_library_on_start"),
    value_type: Some("bool"),
    default_value: Some("true"),
    description: "When enabled, uamp will automatically start search for new \
songs when it starts. It is the same search as if you pressed the 'Search for \
new songs' button.",
};

pub const LIBRARY_SEARCH_PATHS: SetHelp = SetHelp {
    title: "Library search paths",
    json_field_name: Some("search_paths"),
    value_type: Some("Vec<Path>"),
    default_value: Some("Your music folder or CWD (Current Working Director)"),
    description: "This contains all the paths where uamp will search for new \
songs. If the path doesn't exist, it will be just skipped. By removing a path \
from here, no songs are removed from your library.",
};

pub const SONG_EXTENSIONS: SetHelp = SetHelp {
    title: "Song extensions",
    json_field_name: Some("audio_extensions"),
    value_type: Some("Vec<String>"),
    default_value: Some("flac, mp3, m4a, mp4"),
    description: "As a optimization when loading new songs to library, uamp \
will check only files with one of these extensions. If you add a extension of \
a file format that is not supported, uamp will try to load that file and \
fails. That is no problem, but it can slow the library load if there are a \
lot of these files in the library search paths (such as images).",
};

pub const GAPLESS_PLAYBACK: SetHelp = SetHelp {
    title: "Gapless playback",
    json_field_name: Some("gapless"),
    value_type: Some("bool"),
    default_value: Some("false"),
    description: "Some tracks have silence at the beggining or at the end. \
If the gapless playback is enabled, uamp will skip that silence.

This will also affect the displayed length of the track, but it will update \
only when the track is played.",
};

pub const FADE_PLAY_PAUSE: SetHelp = SetHelp {
    title: "Fade play/pause",
    json_field_name: Some("fade_play_pause"),
    value_type: Some("Duration"),
    default_value: Some("00:00.15"),
    description: "When you pause song, it sounds as if the song volume was \
suddently 0, and when you play it, the volume is immidietly back. This can \
change that by gradually silencing the song before it is paused, and \
gradually increase the volume when the song is played.

This setting sets the duration of the silencing/increasing of the volume. \
If this behaviour is not what you want, set the value to 0.",
};

pub const VOLUME_JUMP: SetHelp = SetHelp {
    title: "Volume jump",
    json_field_name: Some("volume_jump"),
    value_type: Some("float"),
    default_value: Some("2.5"),
    description: "When you use a shortcut or CLI for increasing/decreasing \
the volume it changes the volume by some amount. This option specifies that \
amount.

Note that here (in the GUI) the value is in range 0 to 100 (as 0% to 100%),
but in json, the value is scaled to range from 0 to 1",
};

pub const SEEK_JUMP: SetHelp = SetHelp {
    title: "Seek jump",
    json_field_name: Some("seek_jump"),
    value_type: Some("Duration"),
    default_value: Some("00:10"),
    description: "This value specifies how much the song should be rewinded/\
fast forwarded when you click on the rewind/fast forward button or use \
shortcut or the CLI.",
};

pub const ENABLE_GLOBAL_HOTKEYS: SetHelp = SetHelp {
    title: "Enable/disable global hotkeys",
    json_field_name: Some("register_global_hotkeys"),
    value_type: Some("bool"),
    default_value: Some("false"),
    description: "Uamp can register global hotkeys to do things such as play/\
pause the plaback or change the volume. This option determines whether the \
global hotkeys should be registered.

Note that uamp may behave strangely or \
even crash on startup if there are multiple instances that register global \
hotkeys. This is why it is disabled by default.

On linux, this uses X11 to register the the hotkeys, so if you are using \
Wayland the hotkeys may not work before you allow it in your system settings.

As you can see, the hotkeys may not be very reliable. Because of that it is \
recomended that you set hotkeys in your system settings using the uamp CLI \
(there is no setting for that in Windows without third party apps as far as I \
know).",
};

pub const GLOBAL_HOTKEY: SetHelp = SetHelp {
    title: "Global hotkeys",
    json_field_name: Some("global_hotkeys"),
    value_type: Some("Keys: Vec<instance action>"),
    default_value: Some(
        "ctrl+alt+home: pp
ctrl+alt+pg_down: ns
ctrl+alt+pg_up: ps
ctrl+alt+up: vu
ctrl+alt+down: vd
ctrl+alt+left: rw
ctrl+alt+right: ff",
    ),
    description:
        "Here you can add/remove global hotkeys and choose what they \
do. Each hotkey consists of Keys and Actions.

Keys are combination of any \
number of modifier keys (e.g. ctrl, alt, shift,...) and single Code (e.g. \
c, home, up, numpad5...).

Action is the same as instance action in CLI. Try 'uamp h i' to see all the \
actions. There can be any number of the actions for single hotkey, separated \
by whitespace. If there are multiple actions they are executed in the same \
order as they are written.

If you have multiple same Keys, the actions are combined, but there is no \
way to know the order of the actions so it is not recommended to do that.",
};

pub const ENABLE_SERVER_FOR_CLI: SetHelp = SetHelp {
    title: "Enable server for CLI",
    json_field_name: Some("enable_server"),
    value_type: Some("bool"),
    default_value: Some("true"),
    description: "Here you can disable the TCP server that uamp uses to \
comunicate between instances (mostly in CLI).

If you disable this, any of the instance options in CLI won't work.",
};

pub const SERVER_PORT: SetHelp = SetHelp {
    title: "Server port",
    json_field_name: Some("port"),
    value_type: Some("integer"),
    default_value: Some("8267 (33284 in debug)"),
    description: "Uamp creates TCP server so it can comunicate between \
instances. This is mainly used by the CLI. Here you can choose the port on \
which the server listens.",
};

pub const SERVER_ADDRESS: SetHelp = SetHelp {
    title: "Server address",
    json_field_name: Some("server_address"),
    value_type: Some("address"),
    default_value: Some("127.0.0.1"),
    description:
        "This controls the address on which the TCP server that uamp \
creates listens. The default value is equivalent to 'localhost'.",
};

pub const SAVE_BUTTON: SetHelp = SetHelp {
    title: "Save",
    json_field_name: None,
    value_type: None,
    default_value: None,
    description:
        "By default, uamp saves its changes (such as settings or the \
currently playing song and playlist) periodicaly, and every time you close \
it. However you may want to trigger the save (for example you change port for \
server and you want to use CLI). This button does that.",
};

pub const SAVE_TIMEOUT: SetHelp = SetHelp {
    title: "Save timeout",
    json_field_name: Some("save_timeout"),
    value_type: Some("Option<Duration>"),
    default_value: Some("01:00"),
    description:
        "When you change settings, volume or start a new playlist or \
modify the library, the changes are applied immidietly, but not saved. All \
the changes (if any) are always saved when you close uamp (with the 'x' \
button). But sometimes uamp is closed by different means (you turn off your \
computer, or uamp crashes). This is why uamp perodicaly saves. This value \
tells uamp how often it should save.

If you don't want uamp to save periodically, you can confirm empty field (or \
set tis to 'null' in json) and periodical saves will be disabled.",
};

pub const DELETE_LOGS_AFTER: SetHelp = SetHelp {
    title: "Delete logs after",
    json_field_name: Some("delete_logs_after"),
    value_type: Some("Duration"),
    default_value: Some("3d00:00"),
    description: "Uamp logs all internal errors that would otherwise be \
ignored into a file so that it is easier to diagnoze any potential problems. \
But there is no reason why would you need the history of all the logs that \
uamp may create. This is why uamp automatically deletes old logs when you \
close it. This setting sets how old the logs must be so that uamp deletes \
them. It is 3 days by default.",
};

pub const TICK_LENGTH: SetHelp = SetHelp {
    title: "Tick length",
    json_field_name: Some("tick_length"),
    value_type: Some("Duration"),
    default_value: Some("00:01"),
    description: "Uamp has internal clock that updates things such as the \
seek slider, or checks if it should save. With this setting you can change
how often the internal clock ticks.

Setting this to large values (e.g. 00:10) or very small values (e.g. \
00:00.001) is not recomended.

Don't wory about the clock precision, it is made in a way that errors don't \
accumulate.",
};

pub const SHUFFLE_NOW_PLAYING: SetHelp = SetHelp {
    title: "Shuffle now playing",
    json_field_name: Some("shuffle_current"),
    value_type: Some("bool"),
    default_value: Some("true"),
    description: "When you click shuffle, the now playing is shuffled into \
the playlist. If you disable this, the now playing song will be the first in \
the playlist.",
};

pub const SHOW_HELP: SetHelp = SetHelp {
    title: "Show help",
    json_field_name: Some("Show help"),
    value_type: Some("bool"),
    default_value: Some("true"),
    description: "If you disable this, you won't see this help."
};

impl SetHelp {
    pub fn get_element(&self) -> Element {
        let mut items: Vec<Element> = Vec::new();

        items.push(
            line_text(self.title)
                .size(20)
                .height(40)
                .vertical_alignment(Vertical::Center)
                .into(),
        );

        if let Some(jfn) = self.json_field_name {
            items.push(
                row![
                    text("In json:").height(Shrink).width(Shrink),
                    text(jfn).height(Shrink),
                ]
                .height(Shrink)
                .spacing(5)
                .into(),
            )
        }

        if let Some(vt) = self.value_type {
            items.push(
                row![
                    text("Type:").height(Shrink).width(Shrink),
                    text(vt).height(Shrink),
                ]
                .height(Shrink)
                .spacing(5)
                .into(),
            )
        }

        if let Some(dv) = self.default_value {
            items.push(
                col![
                    text("Default value:").height(Shrink),
                    container(text(dv).height(Shrink))
                        .height(Shrink)
                        .padding([0, 0, 0, 20]),
                ]
                .height(Shrink)
                .into(),
            );
        }

        items.push(text(self.description).height(Shrink).into());

        column(items)
            .width(Fill)
            .height(Shrink)
            .spacing(10)
            .padding([0, 0, 20, 0])
            .into()
    }
}
