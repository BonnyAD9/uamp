# TODO
- Custom format for listing songs.
- TUI
- Tag editor
- HTTP client
- GUI
- Images
- Grououping by albums and artists

# known issues
- Restarting Http server takes long time. This is some kind of issue with
  mio/tokio where the future for accepting connection blocks for long time.
- When playing some flac files, log will show error with end of stream. This is
  bug has been fixed in symphona, but it is not yet released.
- When output device doesn't support required sample rate, aliasing may occur.
  (this doesn't happen on any normal system).
