# TODO
- mpris player (https://crates.io/crates/mpris-server)
- Custom format for listing songs.
- TUI
- Tag editor
- HTTP client
- GUI
- Images
- Grououping by albums and artists

# known issues
- Config file change works only once. This is issue with some editors that
  remove the file when writing it. Workaround may be necesary.
- When playing some flac files, log will show error with end of stream. This is
  bug has been fixed in symphona, but it is not yet released.
- When output device doesn't support required sample rate, aliasing may occur.
  (this doesn't happen on any normal system).
