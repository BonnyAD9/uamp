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
For now you cannot customize the shortcuts, so if you want tu use different
shortcuts you need to set global shortcuts to execute commands with the CLI.

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
Version 0.1.2

Usage:
  uamp
    starts the gui of the player

  uamp [action] [--] [action] ...
    does the given action

Actions:
  i  instance <instance-action> [--]
    operates on a running instance of uamp

  h  help  -h  -?  --help
    shows help, with no argument whole help, with arguments only help specific
    to the given option.
    Available options are: basic, i instance

Instance actions:
  pp  play-pause[=(play | pause)]
    play or pause, when without argument, toggle between the states playing and
    paused

  vu  vol-up  volume-up[=<mul>]
    increase the volume by the default amount, when mul is
    specified, multiply the volume increase by that number

  vd  vol-down  volume-down[=<mul>]
    decrease the volume by the default amount, when mul is
    specified, multiply the volume decrease by that number

  ns  next-song[=<N>]
    jump to the Nth next song in the playlist. By default,
    N is 1.

  ps  prev-song  previous-song[=<N>]
    jump to the Nth previous song in the playlist. By default,
    N is 1.

  v  vol  volume=<value>
    set the volume to the given value, value must be
    in range from 0 to 1

  mute[=(true | false)]
    mute/unmute, if the value is not specified, toggles between the states

  load-songs
    look for new songs

  shuffle  shuffle-playlist
    shuffles the current playlist

  pj  playlist-jump=<index>
    jumps to the given index in playlist, stops the playback if
    the index is out of range

  x  exit  close
    exits the instance
```

## How it looks
The gui is in state: at least there is gui. It looks horrible but it has the necesary elements.
![image](https://github.com/BonnyAD9/uamp/assets/46282097/02ec639c-9e5d-4c51-b831-e35a668bf53b)

## How to get it
To use the player you have to compile it yourself, but that shouldn't be any problem because all you need is `cargo`:
```
cargo build -r
```
the binary will be `./target/release/uamp`. It doesn't depend on any other files.

## Links
- **Author:** [BonnyAD9](https://github.com/BonnyAD9)
- **GitHub repository:** [BonnyAD9/makemake-rs](https://github.com/BonnyAD9/uamp)
- **My website:** [bonnyad9.github.io](https://bonnyad9.github.io/)
