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
Version 0.1.0

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
  pp  play-pause
    toggle between the states playing and paused

  vu  vol-up  volume-up
    increase the volume by the default amount

  vd  vol-down  volume-down
    decrease the volume by the default amount

  ns  next-song
    go to the next song

  ps  prev-song  previous-song
    go to the previous song

  v  vol  volume=VALUE
    set the volume to the given VALUE, VALUE must be in range from 0 to 1
```

## Links
- **Author:** [BonnyAD9](https://github.com/BonnyAD9)
- **GitHub repository:** [BonnyAD9/makemake-rs](https://github.com/BonnyAD9/uamp)
- **My website:** [bonnyad9.github.io](https://bonnyad9.github.io/)
