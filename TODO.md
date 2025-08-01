# TODO
- Custom format for listing songs.
- TUI
- Tag editor
- HTTP client
- GUI
- Images
- Grououping by albums and artists

# known issues
- `ERROR [uamp::core::err] Failed to send mpris update: I/O error: Broken pipe (os error 32)`
  `ERROR [uamp::core::err] Failed to send mpris signal: I/O error: Broken pipe (os error 32)`
- When playing some flac files, log will show error with end of stream. This is
  bug has been fixed in symphona, but it is not yet released.
- When output device doesn't support required sample rate, aliasing may occur.
  (this doesn't happen on any normal system).
