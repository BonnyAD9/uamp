# uamp
[![version][aur-badge]][aur]

Universal Advanced Music Player written in Rust.

Uamp is a minimal music player that doesn't get in your way. It is fast,
reliable, and powerful.

Uamp currently works as playback server controled with CLI, [mpris][mpris]
which may be integrated into your system or web GUI:

<img width="2050" height="1210" alt="image" src="https://github.com/user-attachments/assets/c18e061d-69eb-47e0-be97-5e1444ad747b" />

## Configuration
The configuration is saved in the efault configuration folder on your
platform. You can use `uamp config` to open the config file with your default
editor.

## CLI
See short help of uamp:
```sh
uamp -h
```

### Examples

Show help with all the options (shortest version):
```sh
uamp h all
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
Uamp how has web gui:

### Library page
<img width="2050" height="1210" alt="image" src="https://github.com/user-attachments/assets/37d3a224-350d-412a-b2c2-d782d6d964c2" />

### Now playing
<img width="2050" height="1210" alt="image" src="https://github.com/user-attachments/assets/81eb7a7b-3821-43df-8915-a6f718bcce73" />

### Playlist
<img width="2050" height="1210" alt="image" src="https://github.com/user-attachments/assets/bac7554b-6b43-4d5e-b1e0-27e035fbecd8" />

### Albums
<img width="2050" height="1210" alt="image" src="https://github.com/user-attachments/assets/52492c23-ea66-4c7f-a860-07ddecc81050" />

### Album page
<img width="2050" height="1210" alt="image" src="https://github.com/user-attachments/assets/36d9704b-f8d9-475c-8856-737b5e063d31" />

### Artists
<img width="2050" height="1210" alt="image" src="https://github.com/user-attachments/assets/eb8c0411-b6c5-4a53-bb5c-4af2c617cd32" />

### Artist page
<img width="2050" height="1210" alt="image" src="https://github.com/user-attachments/assets/ec92de01-4c98-4cf1-8143-57a8caf117df" />

### Settings
<img width="2050" height="1210" alt="image" src="https://github.com/user-attachments/assets/dcf83be0-d2c5-4359-884e-26828dbe42e2" />

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

### Open gui as app with chrome
Set the setting `web_client_command` to the following value to run uamp web
client in chrome as app (replace `/chrome/binary` with the binary for chrome.
It is usually `google-chrome` or `chromium` or use whatever other browser
that supports web apps):
```json
{
  // ...
  "web_client_command": "/chrome/binary --app=${ADDRESS} --class=uamp"
}
```

### Endless playback
Now you can start the playback with the default alias `uamp i al=endless-mix`
and you never have to worry about it. It will shuffle all your songs into a
playlist and when the playlist ends it will reshuffle the playlist and start
from the start.

When you will load new songs, they will also be automatically mixed in the
playlist after the currently playing song.

### Custom tab complete
Uamp can also provide custom tab completion for any bash like shell (works in
zsh). To do that add this to your `.bashrc`/`.zshrc`/...:
```sh
`uamp sh tab`
```
And that is it. Uamp will now have custom tab completion.

## How to get it

### Arch linux
If you have arch, you can install it from [aur][aur].

### Linux single line install
This install script will ask for sudo privilages to move the build files to
their apropriate locations.

```sh
wget -nv -O - https://raw.githubusercontent.com/BonnyAD9/uamp/master/packages/script/install.sh | sh
```

### Build from source

If you don't have arch, you have to compile it yourself, but that shouldn't be
any problem because all you need is `cargo`:
```sh
cargo build -r
```
the binary will be `./target/release/uamp`. It doesn't depend on any other
files.

## Links
- **Author:** [BonnyAD9][author]
- **GitHub repository:** [BonnyAD9/uamp][github]
- **My website:** [bonnyad9.github.io][my-web]
- **Project website:** [bonnyad9.github.io/uamp][uamp-web]
- **Aur package:** [aur.archlinux.org][aur]


[author]: https://github.com/BonnyAD9
[github]: https://github.com/BonnyAD9/uamp
[my-web]: https://bonnyad9.github.io/
[uamp-web]: https://bonnyad9.github.io/uamp/
[aur]: https://aur.archlinux.org/packages/uamp
[aur-badge]: https://img.shields.io/aur/version/uamp
[mpris]: https://specifications.freedesktop.org/mpris-spec/latest/
