# Unreleased changes

## New features
- Colorful CLI
- Recursive search option
- Don't block when loading songs
- Option to save every N seconds (60 by default)
- Option to set fade play/pause duration (0.15 by default)
- New options to instance cli:
    - `shuffle-playlist`, `shuffle`: shuffles the current playlist
    - `playlist-jump`, `pj`: jumps to the given position in the playlist
    - `exit`, `close`, `x`: exits the instance

## Bugfixes
- Scroll widgets now remember their position while the app is runung
- Next on last song in playlist wouldn't stop playback
- Default volume is now always 1
- Scrollbar thumb size is not limited

## CLI changes
- change instance option for `find-songs`/`fs` to `load-songs`

## General changes
- Stability and performace improvements
- Better error messages
- Log is now more used
- Change default value of `update_library_on_start` to `true`
- Reduce IO operations
