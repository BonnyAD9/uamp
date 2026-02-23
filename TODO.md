# TODO
- Proper support for multidisc albums.
- Song tags (categories)
- Custom format for listing songs.
- TUI
- Tag editor
- GUI
- Grououping by albums and artists

# known issues
- When playlist obruptly ends (calling `ns` on last song or `end-playlist`),
  the play buffer is not flushed will play part of the last song.
- Mpris integration often doesn't show image. Seems to happen at random. (maybe
  try providing lower resolution?)
- When playing some flac files, log will show error with end of stream. This is
  bug has been fixed in symphona, but it is not yet released.
- When output device doesn't support required sample rate, aliasing may occur.
  (this doesn't happen on any normal system).
