# TODO
- Fix playing opus.
- Add option to disable remote access over HTTP.
- Don't send song paths to client.
- Migrate off of astral-tokio-tar.
- Save changes with swap files.
- Push doesn't change playback state (continue playing).
- Avoid redundant tag information in library when saving to json.
- Sort by album artist.
- Proper support for multidisc albums.
- Transition from json.
- Refactor cli code with new pareg features.
- Generic CLI interface (checks if instance runs, if not realizes without it)
- Custom format for listing songs.
- TUI
- Tag editor
- GUI
- Grououping by albums and artists
- Migrate off of tokio.

# known issues
- When playlist obruptly ends (calling `ns` on last song or `end-playlist`),
  the play buffer is not flushed will play part of the last song.
- Mpris integration often doesn't show image. Seems to happen at random. (maybe
  try providing lower resolution?)
- When output device doesn't support required sample rate, aliasing may occur.
  (this doesn't happen on any normal system).
