# TODO
- TUI
- Tag editor
- Add manpage
- AUR release
- HTTP client
- GUI
- Images
- Grououping by albums and artists

# known issues
- When playing some flac files, log will show error with end of stream. This is
  bug has been fixed in symphona, but it is not yet released.
- When output device doesn't support required sample rate, aliasing may occur.
  (this doesn't happen on any normal system).
- `uamp i nfo` will sometimes fail to get current playback position. (this is
  bug in raplay that has been fixed but not yet released)
