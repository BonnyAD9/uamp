# CHANGELOG

## future
### New features
- Mpris support. On linux, OS will recognize uamp as running media player. This
  is enabled by default and may be disabled with a new option `system_player`.

### Changes
- `play` has been made into a control message and can now be used anywhere
  where control message is used.

### Fixes
- Fix issue where some images would accidentally use default background color
  in some terminals instead of the desired color.
- Properly round volume.

## v0.5.12
### New Features
- Show next and previous tracks length.
- Support verbose pretty list.
- Add base to the query.
- Add option to have songs unique by something in query.

### Fixes
- Properly tab complete `update`.

## v0.5.11
### Fixes
- Add missing tab complete
- Properly detect that uamp is already up to date.
- Properly print error when message fails to send.
- Fix blue tint when showing images.

## v0.5.10
### New features
- Add self update with `uamp update`.
- Support winamp way of looking up images.
- Add option to print aliases with `uamp cfg --aliases`.
- Add verbosity option. For now works only for `cfg --aliases`.
- Add option to reorder the playlist stack with `rps` or
  `reorder-playlist-stack`.

### Changes
- Playlist add will now use playlist add policy of all playlists in the stack.
- Escaping of image names in image cache may be slightly different.

### Fixes
- Fix function aliases.
- Fix `uamp i nfo` sometimes failing to retreive position in song.
- Recognize `.webp` files as images.
- Avoid printing with color when logging.
- Escape when looking up images.
- Fix issue where playlist stack was not properly saved and loaded.
- Properly print errors for arguments to `-R` and `-C`.

## v0.5.9
### New features
- Tab completion now can suggest aliases.
- Set the print style with `--print`. Add new print styles: `debug`, `json`.
- Add manpages. Open them with `uamp man`.
- In CLI `config` add `--path` as alias to `--print-path`.
- Option to restart uamp with `restart` control message.
- Uamp will automatically restart if its binary changes.

### Changes
- CLI now uses `{` and `}` instead of `[` and `]`. The old way is still
  supported.
- `uamp run -h` will now not start uamp.
- Change filter mode `+` to `-`.
- Review default configuration.
    - `save_playback_pos` is now by default `OnClose`.
    - `gapless` is by default enabled.
    - `client_image_lookup` is enabled by default.
- Add website link to help.

### Fixes
- Fix image scaling for images that are taller tan wide.
- Fix loading of images with incorrect file extension.
- Properly center images in pretty print.

## v0.5.8
### Changes
- Print additional newline before image.
- Properly support gapless playback by prefetching next song.
- Change the log directory.
- Cache images.

### Fixes
- Uamp will reselect new output device if the old is unavailable.
- Don't discard track position of next playlist when popping stopped playlist.

## v0.5.7
### New features
+ Log panics.
+ Add `i show` that will also clear the screen just before printing.
+ Add client side image lookup. Show images in `i nfo` and `i show`. (disabled
  by default)
### Changes
- Hint showing whole help.
- Add some default aliases (`repeat`, `repeat-once`, `pcont`, `endless-mix` and
  `palb`).
### Fixes
+ Properly exit with failure on errors.
+ Fix saving of function aliases.
+ Properly keep logger alive.
+ Fix incorrect elipsed printing.
+ Properly delete old logs.
+ Fix id aliasing when adding new songs at the same time as removing with
  library load.

## v0.5.6
+ Add ordering `same` for no ordering.
+ Add port aliases `default`, `debug` and `release`.
+ Add new action `config` to show and edit configuration.
+ Add new actions for shell features.
+ Add tab completion. Can be enabled with `` `uamp sh tab` ``.
- Show volume in `i nfo`.
- Better errors.
- Propagate errors much further.
- Allow to reverse reverse ordering.
- Allow empty ordering in query.
- Split help to more categories. Don't automatically show help for related
  categories.
- `uamp --version` now prints only the app id and version.
- Help now prints the app id instead of just `uamp` in the welcome.
- Allow to specify length of playlist to show for `i nfo`.
+ Fix displaying of playlist position (should start from 1, not 0).
+ Fix occasional panic when mixing new songs to playlist.
+ Fix issue when uamp wouldn't exit when save is due.
+ Fix issue when some temporary songs might not play.

## v0.5.5
- Add new functionality to filters (and, or, brackets).
- Add shorter aliases for sorting and filtering.
- Add option to sort songs as part of query
- Add terminal friendly alternatives for descending order.
- Aliases can have arguments.
- Print portion of playlist with `i nfo`
- Print more information about playlist with `i nfo`
- Fix bug where playlist would panic when removing deleted songs.

## v0.5.4
- Add option to enable/disable color in terminal.
- Add option to set default playlist end action.
- Add option to set add policy as property of playlist.
- Add new add policy: `none`/`-` - explicit *don't add to playlist*.
- Add option to select the sorting complexity from cli.
- Allow unlimited playlist stack.
- Add new filtering options.
- Add new options to edit the current playlist (`queue`, `play-next`,
  `push-with-cur`, `flatten`)
- Add option to list songs (`list`)
- Add 1 to playlist position when displaying with `-Info` (so it starts from 1
  instead of 0)
- Log playback errors.
- Fix overflow in `-Info`
- Fix bug where some audio files wouldn't play.

## v0.5.3
- Add aliases for control messages.
- Add playlist end actions.

## v0.5.2
- Add option to save position within song (`save_playback_pos`).
- Add secondary playlist.
- Support temorary songs that can exist only in playlist.
- Add option to directly play file.
- Add option to sort the playlist.
- Watch the config file for changes and automatically reload the configuration.

## v0.5.1
- Speed up searching for new songs.
- Support multiple arguments at once after instance.
- Add option to start the app in detached mode (`run -d`).
- Add option to send messages to multiple instances at once.
- Add alternative ways to use actions (`-I`, `-R`, ...).
- Add option to print version (`--version`).
- Handle exit signals.
- Improve info print.

## v0.5.0
- Option to start playing on start
- Remove gui and related features.
- Add non gui mode for running in background.
- Add options to add newly loaded songs to the current playlist.
- Add option for playlist jump from console.
- Add option to remove songs with non existing paths from library on library
  load, this is enabled by default.

## v0.4.0
- Add all options to settings including help
- Fancy volume icon
- Add option to trigger save (GUI and CLI)
- Add option to not shuffle the currently playing song
- Hide scrollbar when not necesary
- Option to reset setting to default value
- Option to modify previous behaviour (previous or rewind/previous)
- Add option to show remaining time instead of total time

### Bugfixes
- Clicking shuffle wouldn't instantly scroll to the current song
- Scrollbar sometimes wouldn't release
- Numbering in playlist starts from 1
- Songs are now loaded with correct time
- Some button text may not be fully shown
- No more ghosts
- Scrollbar buttons wouldn't work

## v0.3.0
This update was focused on a new gui.

### New features
- Add option to disable server
- Uamp now remembers its window position and size
- Library is saved on another thread
- New gui

### Bugfixes
- Use proper types: fast-forward, rewind, and seek_jump now use Duration
- You can now scroll with scrollbar after opening playlist
- Auto scroll would never scroll to the last item
- Errors when loading from json files weren't logged
- Some icons were incorrectly constructed

## v0.2.0

### New features
- Some instance cli actions can now accept argument to make their behaviour
  more exact (e.g. set state to play instead of toggeling the state)
- Define global shortcuts in config
- Option for gapless playback (disabled by default)
- Support for seeking (CLI, Hotkeys, GUI)
- Option to set how often the internal clock ticks
- Add fast-forward and rewind (CLI, Hotkeys, GUI)
- Add option for how much should fast-forward and rewind seek
- You can now select the port and address of the server in config
- Add option to specify the port and address of the server when starting uamp
    - When used when starting gui, this will disable config saves in the gui
- Get playback info from running instance with `uamp i info`
- Scroll playlist to show the currently playing song
- add option to delete old logs

### Bugfixes
- Mute would not work properly if you save muted player
- Errors when starting server are now logged

## v0.1.2
### New features
- Colorful CLI
- Recursive search option
- Don't block when loading songs
- Option to save every N seconds (60 by default)
- Option to set fade play/pause duration (0.15 by default)
- New options to instance cli:
    - `shuffle-playlist`, `shuffle`: shuffles the current playlist
    - `playlist-jump`, `pj`: jumps to the given position in the playlist
    - `exit`, `close`, `x`: exits the instance

### Other Changes
- Volume up and volume down commands now don't specify the multiplier, but the
  change.

### Bugfixes
- Scroll widgets now remember their position while the app is runung
- Next on last song in playlist wouldn't stop playback
- Default volume is now always 1
- Scrollbar thumb size is not limited

### CLI changes
- change instance option for `find-songs`/`fs` to `load-songs`

### General changes
- Stability and performace improvements
- Better error messages
- Log is now more used
- Change default value of `update_library_on_start` to `true`
- Reduce IO operations

## v0.1.1
## New features
- find new songs
    - button in gui (topright)
    - cli option (`uamp instance find-songs`, `uamp i fs`)

## Bugixes
- fix config file generation
- add mute to help

## v0.1.0
### Features:
- GUI
    - create playlist from all songs
    - shuffle playlist
    - Play/Pause, Next, Previous
    - Volume, mute
    - See now playing
    - See plalist
- Global shortcuts
    - Play/Pause
    - Next song
    - Previous song
    - Volume up/down
- Configuration
    - Config file
    - Set where to search for music
    - Set file extensions to try
    - Update library on start
    - Disable/Enable global shortcuts
    - Set how much to change volume with each volume up/down
- CLI
    - Show help
    - Control currently running instance
        - Play/Pause
        - Next song
        - Previous song
        - Volume up/down, set volume, mute
- State persistance
    - Volume, mute
    - Now playing song
    - Current playlist
