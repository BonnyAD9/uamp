# CHANGELOG

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
