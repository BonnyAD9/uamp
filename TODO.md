# TODO
- Custom format for listing songs.
- TUI
- Tag editor
- HTTP client
- GUI
- Images
- Grououping by albums and artists

# known issues
- When playlist ends with no end action, it might jump to different than the
  first song. This is fixed in future version of raplay that is not yet
  released.
- When playing some flac files, log will show error with end of stream. This is
  bug has been fixed in symphona, but it is not yet released.
- When output device doesn't support required sample rate, aliasing may occur.
  (this doesn't happen on any normal system).
