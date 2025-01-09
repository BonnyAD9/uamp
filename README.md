# uamp
Universal Advanced Music Player written in Rust using Iced.

(May not be as advanced yet)

## Configuration
The configuration is saved in the efault configuration folder on your
platform, on linux that is `~/.config/uamp`.

The only file that should be edited by the user is `config.json`.

## Global shortcuts
If you enable shortcuts in the configuration, this is what they are:
- **`Ctrl` + `Alt` + `Home`:** Play/Pause
- **`Ctrl` + `Alt` + `PgUp`:** Previous song
- **`Ctrl` + `Alt` + `PgDown`:** Next song
- **`Ctrl` + `Alt` + `Up`:** Volume up
- **`Ctrl` + `Alt` + `Down`:** Volume down
- **`Ctrl` + `Alt` + `Left`:** Rewind
- **`Ctrl` + `Alt` + `Right`:** Fast forward
You can customize the shortcuts in `config.json`

For example to play/pause you can use the command:
```
uamp instance play-pause
```
or the short version
```
uamp i pp
```

## CLI
This is the output of help:
```
Welcome in uamp by BonnyAD9
Version 0.4.0

Usage:
  uamp
    starts the gui of the player

  uamp [action] [--] [action] ... [flags]
    does the given action

Flags:
  -p  --port <port>
    Sets the port for the server comunication. If used when starting gui, it
    will disable config saves.

  -a  --address <address>
    Sets the server address for the comunication. If used when starting gui, it
    will disable config saves.

Actions:
  i  instance <instance-action> [--]
    operates on a running instance of uamp

  h  help  -h  -?  --help
    shows help, with no argument whole help, with arguments only help specific
    to the given option.
    Available options are: basic, i instance

Instance actions:
  info
    Shows the info about the playback of the currently runing instance.

  play-pause  pp[=(play|pause)]
    Play or pause, when without argument, toggle between the states
    playing and paused.

  volume-up  vol-up  vu[=<f32>]
    Increase the volume by the given amount. If the parameter is not
    present, increase by the default amount

  volume-down  vol-down  vd[=<f32>]
    Decrease the volume by the given amount. If the parameter is not
    present, decrease by the default amount

  next-song  ns[=<usize>]
    Jump to the next song, arguments specifies how much to jump (e.g.
    with argument '2' skips one song and plays the next).

  previous-song  ps[=<usize>]
    Jump to the previous song, arguments specifies how much to jump
    (e.g. with argument '2' skips the previous song and plays the
    second previous song).

  volume  vol  v=<f32>
    Set the volume to the given value. Value must be in range from 0 to 1

  mute[=<bool>]
    Mute/Unmute, if the argument is not specified, toggles between
    the states

  load-songs
    Look for new songs.

  shuffle-playlist  shuffle
    Shuffles the current playlist.

  exit  close  x
    Exits the instance

  seek-to  seek=<Duration>
    Seeks to the given timestamp. Timestamp is in format 'h:m:s'.

  fast-forward  ff[=<Duration>]
    Seeks forward by the given amout in seconds. If the parameter is not
    present, seek by the default amount.

  rewind  rw[=<Duration>]
    Seeks backward by the given amout in seconds. If the parameter is not
    present, seek by the default amount.

  save
    Triggers save (saves only if there is change)
```

## How it looks
![image](https://github.com/BonnyAD9/uamp/assets/46282097/639c9849-f0f2-4fad-91f3-949ef68e9a3e)

## How to get it
To use the player you have to compile it yourself, but that shouldn't be any
problem because all you need is `cargo`:
```
cargo build -r
```
the binary will be `./target/release/uamp`. It doesn't depend on any other
files.

## Links
- **Author:** [BonnyAD9](https://github.com/BonnyAD9)
- **GitHub repository:** [BonnyAD9/makemake-rs](https://github.com/BonnyAD9/uamp)
- **My website:** [bonnyad9.github.io](https://bonnyad9.github.io/)
