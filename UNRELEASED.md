# Unreleased changes

## New features
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

## Bugfixes
- Mute would not work properly if you save muted player
- Errors when starting server are now logged
