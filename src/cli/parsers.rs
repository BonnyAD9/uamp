use std::time::Duration;

use crate::{
    core::msg::ControlMsg,
    library::{Filter, LoadOpts},
    parse_arg,
};

parse_arg! {ControlMsg as parse_control_message, auto_instance_help:
    ? "Play or pause, when without argument, toggle between the states
       playing and paused."
    "play-pause" | "pp" {= "play" -> true, "pause" -> false} => PlayPause;

    ? "Increase the volume by the given amount. If the parameter is not
       present, increase by the default amount"
    "volume-up" | "vol-up" | "vu" [=f32] => VolumeUp;

    ? "Decrease the volume by the given amount. If the parameter is not
       present, decrease by the default amount"
    "volume-down" | "vol-down" | "vd" [=f32] => VolumeDown;

    ? "Jump to the next song, arguments specifies how much to jump (e.g.
       with argument '2' skips one song and plays the next)."
    "next-song" | "ns" [=usize] => NextSong(1);

    ? "Jump to the previous song, arguments specifies how much to jump
       (e.g. with argument '2' skips the previous song and plays the
       second previous song)."
    "previous-song" | "ps" [=usize] => PrevSong;

    ? "Jump to the song at the given index in the playlist."
    "playlist-jump" | "pj" [=usize] => PlaylistJump(0);

    ? "Set the volume to the given value. Value must be in range from 0 to 1"
    "volume" | "vol" | "v" =f32 => SetVolume: |v| (0.0..=1.).contains(v);

    ? "Mute/Unmute, if the argument is not specified, toggles between
       the states"
    "mute" [=bool] => Mute;

    ? "Look for new songs. Load options are of the form `[r|l][e|n|m]`.
       - `r` enables removing of songs with invalid path for the load.
       - `l` disables removing of songs with invalid path for the load.
       - `e` adds the new songs to the end of the queue.
       - `n` adds the new songs after the current songs.
       - `m` randomly mixes the new songs after the current songs."
    "load-songs" [=LoadOpts] => LoadNewSongs(Default::default());

    ? "Shuffles the current playlist."
    "shuffle-playlist" | "shuffle" => Shuffle;

    ? "Exits the instance"
    "exit" | "close" | "x" => Close;

    ? "Seeks to the given timestamp. Timestamp is in format 'h:m:s'."
    "seek-to" | "seek" =Duration  => SeekTo;

    ? "Seeks forward by the given amout in seconds. If the parameter is not
       present, seek by the default amount."
    "fast-forward" | "ff" [=Duration] => FastForward;

    ? "Seeks backward by the given amout in seconds. If the parameter is not
       present, seek by the default amount."
    "rewind" | "rw" [=Duration] => Rewind;

    ? "Sets the playlist to all songs."
    "set-playlist" | "sp" [=Filter] => SetPlaylist(Filter::All);

    ? "Triggers save (saves only if there is change)"
    "save" => Save;
}
