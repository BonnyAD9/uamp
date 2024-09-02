# CHANGELOG

## future
- Add option to enable/disable color in terminal.
- Add option to set default playlist end action.
- Add option to set add policy as property of playlist.
- Add new add policy: `none`/`-` - explicit *don't add to playlist*.
- Add option to select the sorting complexity from cli.
- Allow unlimited playlist stack.
- Add new filtering options.
- Add new options to edit the current playlist (`queue`, `play-next`,
  `push-with-cur`, `flatten`)
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
