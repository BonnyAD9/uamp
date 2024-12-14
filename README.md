# uamp
Universal Advanced Music Player written in Rust using Iced.

(May not be as advanced yet)

## Configuration
The configuration is saved in the efault configuration folder on your
platform, on linux that is `~/.config/uamp`.

The only file that should be edited by the user is `config.json`.

## CLI
See help of uamp to see all the possibilities:
```sh
uamp -h
```

### Examples

Show help with all the options (shortest version):
```sh
uamp h
```

To play/pause you can use the command:
```sh
uamp instance play-pause
```
or the short version
```sh
uamp i pp
```

To start uamp in backgound:
```sh
uamp run -d
```
or short:
```sh
uamp -R-d
```

Stop the running instance:
```sh
uamp i x
```

Show info about now playing:
```sh
uamp -Info
```

Set the playlist to all songs, shuffle and play:
```sh
uamp i sp sort=rng pj pp=play
```

Play file in the currently running instance:
```sh
uamp i play='path/to/file.flac'
```

## How it looks
Currently uamp has no GUI or TUI. The closest thing to gui that uamp has is the
output of `uamp -Info`:
![image](https://github.com/user-attachments/assets/3404a1f8-7463-4823-8cbe-888fb2b383fb)

## Possible uamp setup
This is the way that I have confugred and use uamp:

Make uamp start on startup with your cmomputer with the command `uamp`
(or `uamp -R-d` to make it detached).

Use your OS settings to bind global shortcuts to commands for controlling uamp.
For example:
- **`Ctrl` + `Alt` + `Home`:** `uamp i pp` (play/pause)
- **`Ctrl` + `Alt` + `PgUp`:** `uamp i ps` Previous song
- **`Ctrl` + `Alt` + `PgDown`:** `uamp i ns` Next song
- **`Ctrl` + `Alt` + `Up`:** `uamp i vu` Volume up
- **`Ctrl` + `Alt` + `Down`:** `uamp i vd` Volume down
- **`Ctrl` + `Alt` + `Left`:** `uamp i rw` Rewind
- **`Ctrl` + `Alt` + `Right`:** `uamp i ff` Fast forward

Now you can just start playing when you want to by using your global shortcut.

### Endless playback
If you want to setup endless playback of your songs shuffled you can also:

create alias in `config.json`:
```json
{
  // ...
  "control_aliases": {
    // ...
    "reset-playlist": "sp sort=rng pj pp=play pap=m spea=reset-playlist"
  }
}
```

Now you can start the playback with `uamp i al=reset-playlist` and you never
have to worry about it. It will shuffle all your songs into a playlist and when
the playlist ends it will reshuffle the playlist and start from the start.

When you will load new songs, they will also be automatically mixed in the
playlist after the currently playing song.

## How to get it
To use the player you have to compile it yourself, but that shouldn't be any
problem because all you need is `cargo`:
```sh
cargo build -r
```
the binary will be `./target/release/uamp`. It doesn't depend on any other
files.

## Links
- **Author:** [BonnyAD9](https://github.com/BonnyAD9)
- **GitHub repository:** [BonnyAD9/makemake-rs](https://github.com/BonnyAD9/uamp)
- **My website:** [bonnyad9.github.io](https://bonnyad9.github.io/)
