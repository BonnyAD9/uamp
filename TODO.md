# TODO
- Custom format for listing songs.
- TUI
- Tag editor
- HTTP client
- GUI
- Images
- Grououping by albums and artists

# known issues
- Sending `pj seek=value pp` ignores the seek when freshly started:
  `ERROR [uamp::background_app] error: Cannot operate on a source because there is no source playing`
- When playlist ends with no end action, it might jump to different than the
  first song. This is fixed in future version of raplay that is not yet
  released.
- mpris seek not properly notified (Can't reproduce, will see in next release
  if it is still issue. If yes, doesn't seem to be uamp related.)
- When playing some flac files, log will show error with end of stream. This is
  bug has been fixed in symphona, but it is not yet released.
- When output device doesn't support required sample rate, aliasing may occur.
  (this doesn't happen on any normal system).
