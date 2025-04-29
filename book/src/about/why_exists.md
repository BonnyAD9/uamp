# Why does uamp exist?

I was used to using winamp on windows. When I transitioned to Linux, I couldn't
find any music player that would satisfy my needs:

### Speed

I need fast music player capable of managing large amounts of local songs (my
library has over 3000 songs). Many existing music players weren't able to
handle this large amount of songs. Uamp has no problem with this number of
songs, and is made to delegate expensive operations to other threads so that it
never becomes unresponsive.

### Global keyboard shortcuts

I'm used to using global keyboard shortcuts to control my music player. Many
music players don't have global keyboard shortcuts or any alternative or the
shortcuts don't work well. Uamp also doesn't have any global keyboard
shortcuts, but it can be controled with CLI. This makes it very simple to
create keyboard shortcuts for the required actions in your window manager (e.g.
Kwin).

### Fade play/pause

Since I discovered this feature in winamp, I can't use play/puase without it.
It is very unpleasant when you pause/play your playback and it just transitions
with silence immidietely. This feature adds few milliseconds where the volume
changes from the current to 0 in case of pause and from 0 to the current volume
ensuring smooth transition between playback and silence. Uamp allows you to
specify this fade duration and if you don't like it, you can just disable this
feature.

### Fine control over low volumes

It is very annoying if the volume controls are liner. I remember using music
player where the volume I wanted was between 0 and 5 % but the controls jumped
in change of 5 % making huge changes to ghe volume. And on the other end of the
scale, difference between 80 % and 100 % was minimal. Ideal volume slider would
be logarighmic, but this slider doesn't have good definition on finite range
and so it would be very unintuitive to use. This is why I chose quadratic
slider. This is simple yet good approximation of the logarithimc slider on
finite volume range. This is why changes to low and high volumes in uamp feels
to have much more consistant impact. Uamp also allows you to configure the
volume jump, so you can set it to your liking.

### Endless playback

When I play music on background I usually play random mix of all of my songs.
Many players make this operation quite hard or it is not implemented very well
(it chooses random songs instead of using randomly mixed queue). In the end,
every player has some way of achieving this. Uamp makes this very simple and
goes further by automating the process of reshuffling the playlist when it
finishes. Uamp has by default defined alias that will do exactly that:

*randomly shuffle all songs. If playlist ends, reshuffle the playlist and
continue from start. If new songs are added to the library, shuffle them into
the current playlist after the current song position.*

This is all done trough patemeters on playlist that allow define what will
happen when the playlist ends and if and how new songs should be also added to
the playlist.

### Returning to previous queue

Sometimes I want to listen to specific album. If I play this album, the current
queue will be discarted. Uamp has stack of queues. You can play the specific
album and have uamp continue with playing of the previous queue without
modyfing the previous queue.

### Playing audio file that is not in the library

Sometimes I just want to play audio file that is not in my song library and I
don't want to add it to the song library. I just want to play it now once. Uamp
allows you to do that. If you play any audio file that is not in your library,
uamp will pause the current playback, push the audio file as new queue and when
it finishes playing, uamp will continue with playback of the previous queue.
The previous queue will stay unmodified and the audio file will not be added to
the library.

### Stability, reliability

If there is any bug in uamp I can just fix it. I use uamp daily and if I
encounter any issue, I just fix it. If the issue in any way prevents me from
using uamp the way I want to, I just push new version of uamp with the fix
right away.

Some players stop the playback if there is any issue with playing specific
song. Uamp will just log that there was issue, and silently continue to the
next song.

Some players dont't save their state when you forget to properly close them. If
you just turn you PC off, they will forget all changes since they were open. If
they for some reason crash, the state will be also reset. Uamp properly handles
all exit signals and saves its state before exiting. Uamp also periodically
saves any changes to its state so that if it would crash, the saved state is
still quite recent.

Some players for some reason have issues playing when other demanding task is
running on the PC. One I was using was stuttering the audio when I was playing
games. Since the begining of uamp, I never encountered any issue of that kind
with uamp. Even if the CPU is running at 100 %, uamp still plays.

## My dream project

Music player is also my dream project. Uamp is the realization of that dream
project. When I was using winamp, I had no reason to create fully functional
music player but after I started using Linux, I had to make myself usable music
player.

In order to create uamp I had to make several rust libraries (crates) because
the existing ones weren't good enough. This includes
[raplay](https://github.com/BonnyAD9/raplay) for playing audio,
[termal](https://github.com/BonnyAD9/termal) for colored printing to terminal
(and other terminal manipulation) and
[pareg](https://github.com/BonnyAD9/pareg) as flexible command line argument
parser (and parser in general).
