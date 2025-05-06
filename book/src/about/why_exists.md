# Why does uamp exist?

I was used to using Winamp on Windows. When I transitioned to Linux, I couldn't
find any music player that would satisfy my needs:

### Speed

I need a fast music player capable of managing large amounts of local songs (my
library has over 3000 songs). Many existing music players couldn't handle this
large number of songs. UAMP has no problem with this number of songs, and I
made it to delegate expensive operations to other threads so that it never
becomes unresponsive.

### Global keyboard shortcuts

I'm used to using global keyboard shortcuts to control my music player. Many
music players don't have global keyboard shortcuts or any alternative, or the
shortcuts don't work well. Uamp has no global keyboard shortcuts, but users can
control it with the CLI. The uamp CLI makes it very simple to create keyboard
shortcuts for the required actions in your window manager (e.g., Kwin).

### Fade play/pause

Since I discovered this feature in Winamp, I can't use play/pause without it.
It is very unpleasant when you pause/play your playback, and it just
transitions with silence immediately. This feature adds a few milliseconds
where the volume changes from the current to 0 in case of pause and from 0 to
the current volume, ensuring a smooth transition between playback and silence.
Uamp allows you to specify this fade duration, and if you don't like it, you
can disable this feature.

### Fine control over low volumes

It is very annoying if the volume controls are linear. I remember using a music
player where the volume I wanted was between 0 and 5 %, but the controls jumped
by a change of 5 %, making huge changes to the volume. And on the other end of
the scale, the difference between 80 % and 100 % was minimal. An ideal volume
slider would be logarithmic, but this slider doesn't have a good definition on
a finite range, and so it would be very unintuitive to use. I chose a quadratic
slider. It is a simple yet good approximation of the logarithmic slider on a
finite volume range. Thanks to the nonlinear volume slider, changes to low and
high volumes in uamp have a much more consistent impact. Uamp also allows you
to configure the volume jump so that you can set it to your liking.

### Endless playback

When I play music in the background, I usually play a random mix of all of my
songs. Many players make this operation quite hard, or it is not implemented
very well (it chooses random songs instead of using a randomly mixed queue). In
the end, every player has some way of achieving this. Uamp makes this very
simple and goes further by automating the process of reshuffling the playlist
when it finishes. Uamp has, by default, defined an alias that will do precisely
that:

*Randomly shuffle all songs. If the playlist ends, reshuffle it and continue
from the start. If new songs are added to the library, shuffle them into the
current playlist after the current song position.*

This is done through parameters on a playlist that define what will happen when
the playlist ends and if and how new songs should be added to the playlist.

### Returning to the previous queue

Sometimes I want to listen to a specific album. If I play this album, the
current queue will be discarded. Uamp has a stack of queues. You can play a
specific album and have uamp continue playing the previous queue without
modifying it.

### Playing an audio file that is not in the library

Sometimes I want to play an audio file that is not in my song library, and I
don't want to add it to the song library. I want to play it once. Uamp allows
you to do that. If you play any audio file that is not in your library, uamp
will pause the current playback and push the audio file to the new queue. When
it finishes playing, uamp will continue with the playback of the previous
queue. The previous queue will stay unmodified, and uamp will not add the audio
file to the library.

### Stability, reliability

If there is any bug in uamp, I can fix it. I use uamp daily, and if I encounter
any issue, I fix it. If the issue prevents me from using uamp the way I want, I
will push a new version of uamp with the fix immediately.

Some players stop the playback if there is any issue with playing a specific
song. Uamp will log that there was an issue and silently continue to the next
song.

Some players don't save their state when you forget to close them properly. If
you turn your PC off, they will forget all changes since they were open. If the
player, for some reason, crashes, it will also lose its state. Uamp correctly
handles all exit signals and saves its state before exiting. Uamp also
periodically saves any changes to its state so that if it crashes, the saved
state is still relatively recent.

Some players, for some reason, have issues playing when another demanding task
is running on the PC. One time, I was using it and stuttering the audio while
playing games. Since the beginning of uamp, I have never encountered any issue
of that kind with uamp. Even if the CPU is running at 100 %, uamp still plays.

## My dream project

A music player is also my dream project. Uamp is the realization of that dream
project. When I was using Winamp, I had no reason to create a fully functional
music player, but after I started using Linux, I had to make myself a usable
music player.

To create uamp, I had to make several Rust libraries (crates) because the
existing ones weren't good enough. The libraries include
[raplay](https://github.com/BonnyAD9/raplay) for playing audio,
[termal](https://github.com/BonnyAD9/termal) for colored printing to terminal
(and other terminal manipulation), and
[pareg](https://github.com/BonnyAD9/pareg) as a flexible command line argument
parser (and parser in general).
