# UAMP 1 2025-04-11

## NAME

uamp - Universal Advanced Music Player - command line interface

## SYNOPSIS

`uamp` [`-h`] [`-p` *port*] [`-a` *address*] [`--version`]
[`--color` *color-mode*] [`--print` *print-mode*] [*action* [*action-opt*] ...
[`--` [*action* ...]]]

`uamp` `help` [*help-section*] ...

`uamp` `instance` [`-h`] [`-p` *port*] [`-a` *address*] [*instance-message*]
... [*control-message*] ... [`--`]

`uamp` `run` [`-h`] [`-d`] [`-p` *port*] [`-a` *address*] [*control-message*]
... [`--`]

`uamp` `config` [`-h`] [`-e`] [`-p`] [`--default`] [`--`]

`uamp` `shell` [`-h`] [`-s`] [`tab`]

`uamp` `man` [`-h`] [`-p`] [*man-page*]

`uamp` `update` [`-h`] [`-f`] [`--remote` *remote*] [`--man`] [`--no-man`]
[`--`]

`uamp` `internal` `tab-complete` *opt-index* *uamp-bin* [*opt*] ...

## DESCRIPTION

`uamp` is pure CLI music player server without any other UI. It is ment to run
in background and be controled trough CLI. `uamp` is made to be robust,
efficient and light on system resources. Even though `uamp` doesn't have any UI
it is full of features such as fade play/pause, gapless playback and stack
based playlist (queue) management.

### Actions

Uamp has few categories of actions it can do. Specifying action will move
processing of arguments to mode specific to that action. The mode can be ended
with `--`.

For example in:

    uamp --print json instance -p debug next-song info -- --version run -d

arguments `--print json` are in the default mode. Argument `instance` is action
that will move processing to instance arguments. Arguments `-p debug`,
`next-song` and `info` are in the instance mode. The argument `--` will end the
instance mode and move argument processing back to the default mode. Argument
`--version` is again in the default mode. `run` is action that will move
processing to run arguments in which the argument `-d` is processed. Arguments
now end so there is no need to end the run mode with `--`.

### Playlists/Queues

Playlist and queue refer to the same thing. Queue is just the playlist that
plays right now. I prefer using the word playlist, even in places where you
would expect the word queue.

Uamp has unique playlist managment. It contains stack of playlists. Each
playlist has its songs, current position within playlist, current position
within the song and user action that will happen once the playlist ends.

The playlist at the top of the stack is the one that is currently playing. It
can be created for example by using `uamp i p=`*audio-file* or
`uamp i push=`*query*. The top playlist can be poped by using `uamp i pop`.
Popping the plalist doesn't change the playback state, so if uamp was playing
while the playlist was popped, it will continue to play from the new top
playlist at the current position within the current song.

Each playlist has asociated end action. The end action is just alias
invokation. If playlist ends, the playback state will change to stopped and
than the end action will be invoked if it is set. When creating playlist, the
end action will be set to the default end action specified in configuration.

By default, uamp has defined some aliases useful as end actions for playlists:

- `repeat-once` - when the playlist ends, repeat it one more time, than do the
  default end action.
- `repeat` - repeat the playlist indefinitely.
- `pcont` - pop the playlist and continue playing with the next playlist.

### Configuration

Configuration is usually saved in `~/.config/uamp`. You can get the
configuration path with `uamp config -p`.

If you wan't to edit the configuration you can use `uamp config`. This will
open the configuration in your default text editor. For detailed decription of
configuration see *uamp(5)*.

Running instances of uamp will get notified when the configuration file changes
and they will automatically reload their configuration.

When uamp server starts and either port or address is set from the CLI, the
server will disable saving of configuration and listening to changes in config
file. This is to avoid overwriting configuration with values from cli that are
not ment to be saved, and to avoid resetting the values in the server from the
configuration.

## OPTIONS

Uamp has few categories of actions it can do. If no action is specified, uamp
start as server, unless `-h` or `--version` is specified in the arguments.

These are the actions:

`h` *help-section* ... [`--`], `help` *help-section* ... [`--`]
  Print help for the given sections. See *Action help* for more info.

`i` *instance-opt* ... [`--`],
`instance`  *instance-opt* ... [`--`]
  Send messages to running instance of uamp. See *Action instance* for more info.

`run` *run-opt* ... [`--`]
  Start new instance of uamp. See *Action run* for more info.

`cfg` *config-opt* ... [`--`], `conf` *config-opt* ... [`--`],
`config` *config-opt* ... [`--`]
  Manage uamp configuration. See *Action config* for more info.

`sh` *shell-opt* ... [`--`], `shell` *shell-opt* ... [`--`]
  Get shell integration script for uamp.

`internal` *internal-opt* ...
  Internal commands used for integration. Usualy not useful to be used directly
  by users. See *Action internal* for more info.

These are the core options:

`-h`, `-?`, `--help`
  Show short (basic) help for uamp.

`-p` *port*, `--port` *port*
  Set port for http server which is used for communication with other instances
  of uamp.

  *port* may be:

  - number from `0` to `65535`
  - `-` or `default` (default) - use the default port uamp uses in this build
    mode.
  - `debug` - use port 33284. Default port used by debug builds of uamp.
  - `release` or `uamp` - use port 8267. Default port used by release builds
    of uamp.

`-a` *address*, `--address` *address*
  Set address used for http server which is used for communication with other
  instances of uamp.

  This may be any IPv4, IPv6 or DNS address.

  By default uamp uses value `127.0.0.1` for local server.

`--version`
  Print the version of uamp.

`-I`*arg*
  Equivalent to:
    `instance` *arg* `--`

  It is useful as shorthand if you want to do single instance action and than
  follow it by other arguments.

`-R`*arg*
  Equivalent to:
    `run` *arg* `--`

  It is useful as shorthand if you want to do single run action and than follow
  it by other arguments.

`-H`*arg*
  Equivalent to:
    `help` *arg* `--`

  It is useful as shorthand if you want to do single help actoin and than
  follow it by other arguments.

`--color` *color-mode*, `--colour` *color-mode*
`--color=`*color-mode*, `--colour=`*color-mode*
  Set the color mode.

  *color-mode* may be one of:

  - `auto` (default) - use color if printing to terminal.
  - `always` - use color.
  - `never` - don't use color.

  Help actions are evaulated immidietely, so they will use the last color mode
  before the help action. Other printing will use the last mode set. So when
  using:

    uamp --color always --help instance info -- --color never

  Help will be printed in color, but the instance info will be printed without
  color.

`--print` *print-mode*
  Sets the print mode for information from running instance.

  The print mode may be on of:

  - `pretty` (default) - print in human friendly format.
  - `debug` - print the exact received information using rust debug
    implementation.
  - `json` - print the exact received information in json format.

`-v`*verbosity*, `--verbose`
  Set the verbosity. The default verbosity is `0`. If this is present without
  specific verbosity, it will be set to `1`. The verbosity may also be set to
  any positive or negative integer (in the range of 32 bit signed integer).

`--config` *config-path*
  Set custom path to config file. Note that this configuration will not apply
  when starting detached uamp. Detached uamp will always load the default
  config file.

### Action `help`

`help` [*help-section*] ... [`--`]

`h` [*help-section*] ... [`--`]

This will show the help for the given actions. If the action `help` is the last
argument, it will show the basic help. If there are no *help-section*s
specified, and the help action is ended with `--`, this will print only the
help header.

The help header is printed always exactly once with the help action. It
contains the build mode that is either `uamp` or `uamp_debug`, author nick
(BonnyAD9) version of uamp and basic description of what uamp is.

Help is printed for each of the sections in the order in which they are
specified. If there are duplicates or informational overlaps, the duplicates
and overlaps will be also in the output.

The available sections are:

`all`, `elp`
  Print all the sections in sensible order.

`basic`
  Print the basic help. This is the default.

`i`, `instance`
  Print help specific to instance action.

`run`
  Print help specific to run action.

`cfg`, `conf`, `config`
  Print help specific to config action.

`sh`, `shell`
  Print help specific to shell action.

`internal`
  Print help specific to internal action.

`h`, `help`, `-h`, `-?`, `--help`
  Print help specific to this help action.

`man`
  Print help specific to man action.

`update`
  Print help specific to update action.

`cmsg`, `control-msg`, `control-messages`
  Print help for all control messages.

`format`, `formats`
  Print help for all formats.

`port`
  Print help for port format.

`query`
  Print help for query format.

`base`
  Print help for base format.

`filter`
  Print help for filter format.

`order`
  Print help for order format.

`unique`
  Print help for unique format.

### Action `instance`

`instance` [`-h`] [`-p` *port*] [`-a` *address*] *instance-message* ...
*control-message* ... [`--`]

`i` [`-h`] [`-p` *port*] [`-a` *address*] *instance-message* ...
*control-message* ... [`--`]

Instance action will communicate with running instance of uamp. It will send
messages over HTTP to running uamp server. It supports every *control-message*
and some additional messages specific to `instance` action.

For *control-message*s see *Message control*.

These are options available for instance:

`-h`, `-?`, `--help`
  Print help for instance. The help is equivalent to what would be printed
  with:

    uamp help instance

`-p` *port*, `--port` *port*
  Sets port for the communication with running uamp instance. This may have the
  same values as `--port` in core options. If not specified, port from core
  options will be used.

`-a` *address*, `--address` *address*
  Sets address for communication with running uamp instance. This may have the
  same values as `--address` in the core options. If not specified, address
  from the core options will be used.

`-v`*verbosity*, `--verbose`
  Set the verbosity. The default verbosity is `0`. If this is present without
  specific verbosity, it will be set to `1`. The verbosity may also be set to
  any positive or negative integer (in the range of 32 bit signed integer).

These are instance messages:

`nfo`[`=`[`-`*before*]..[*after*]], `info`[`=`[`-`*before*]..[*after*]]
  Request information about current playback from the running instance.
  *before* and *after* are numbers specifying how much songs in the current
  queue before and after the current song should be sent. The default value
  for *before* and *after* is `0`. If the range is not specified, the default
  range `-1..3` will be used.

  The print format of the output is specified by the core option `--print`. If
  print mode is set to `pretty`, color is enabled and client side image lookup
  is enabled in configuration, uamp will also lookup image and print it using
  ansi colored blocks.

`show`[`=`[`-`*before*]..[*after*]]
  Same as `info`, but it will also clear the screen if in print mode is set to
  `pretty`.

  This is useful to minimize the blank screen time in simple scripts such as:

    while uamp i show; do sleep 1; done

`l`[`=`*query*], `list`[`=`*query*], `query`[`=`*query*]
  Search in all songs managed by running instance of uamp. *query* specifies
  filter for the songs and their order. See *Format query* for more info.
  
  The amount of printed information is affected by verbosity.

### Action `run`

`run` [`-h`] [`-d`] [`-p` *port*] [`-a` *address*] [*control-message*] ...
[`--`]

Run new instance of uamp server. The instance must have unique combination of
port and address so that it can create HTTP server. The control messages will
run on the server when it starts.

If either port or address is specified (here or in the core options), the new
instance will not save its configuration and will not react to configuration
changes.

The server will exit when it receives close message, or when it receives
terminating signal. If the server will receive four terminating signals, it
will end itself forcefully. So you are free to exit non-detached uamp with
Ctrl+D, it will handle the signal and exit correctly.

For *control-message*s see *Message control*.

Run action accepts the following options:

`-h`, `-?`, `--help`
  Show help for usage of run. If this is present without any other options, the
  server will not start.

`-d`, `--detach`
  Run uamp in background as detached process.

`-p` *port*, `--port` *port*
  Set port for the HTTP server of the new instance. The new instance will not
  save cafiguration or load it when it updates to preserve different
  configuration in both places.

`-a` *address*, `--address` *address*
  Set address for the HTTP server of the new instance. The new instance will
  not save cafiguration or load it when it updates to preserve different
  configuration in both places.

`-b`, `--background`
  Set run mode to `Background`: start the uamp server. Will block if `-d` is
  not specified. This is the default run mode.

`-w`, `--web`
  Set run mode to `WebClient`: start detached uamp server if not already
  running and than open the web client. Shouldn't block - this is behaviour is
  dependant on the environment, but in most environments it won't.

### Action `config`

`config` [`-h`] [`-e`] [`-p`] [`--default`] [`--`]

`conf` [`-h`] [`-e`] [`-p`] [`--default`] [`--`]

`cfg` [`-h`] [`-e`] [`-p`] [`--default`] [`--`]

Manage configuration of uamp. If no options are specified it is as if only the
option `-e` was specified.

Config action accepts the following options:

`-h`, `-?`, `--help`
  Print help for config action.

`-e`, `--edit`, `--edit-file`
  Open the configuration file in your default editor.

`-p`, `--print-path`
  Print path to the configuration file.

`--default`
  Print the default configuration in json format (same as the actual format of
  the configuration file).
  
`--aliases`
  Print all the aliases sorted alphabetically. If verbosity is at least 1, the
  definitions for the aliases will be also printed.

`-v`*verbosity*, `--verbose`
  Set the verbosity. The default verbosity is `0`. If this is present without
  specific verbosity, it will be set to `1`. The verbosity may also be set to
  any positive or negative integer (in the range of 32 bit signed integer).

### Action `shell`

`shell` [`-h`] [`-s`] [`tab`]

`sh` [`-h`] [`-s`] [`tab`]

Print script for shell integration. Right now the only shell intergration
script is for tab completion. The shell scripts are verified to work in bash
and zsh.

By default only short runner script is printed.

The tab completoion script can be integrated just by using the following shell
command:

    `uamp sh tab`

The shell action accepts the following options:

`-h`, `-?`, `--help`
  Print help for shell action.

`-s`, `--script`
  Print long script instead of short script runner. The two scripts don't have
  to be different.

The following shell integrations are supported:

`tab`, `tab-completion`
  Adds tab completion for uamp CLI. Verified to work in `bash` and `zsh`.

### Action `man`
`man` [`-h`] [`-p`] [*man-page*]

Open the given man page with the program `man`. The man page doesn't have to be
installed, but the program `man` must exist.

It accepts the following flags:

`-h`, `-?`, `--help`
  Print help for the man page command.

`-p`, `--print`
  Print the man page directly to stdout instead of using `man`.

The following *man-page* arguments are accepted:

`1`, `cli`
  Show the man page for section `1` that describes CLI. It is this manpage.

`5`, `cfg`, `conf`, `config`
  Show man page for section `5` that describes configuration file.

### Action `update`

`update` [`-h`] [`-f`] [`--remote` *remote*] [`--man`] [`--no-man`] [`--`]

Updates uamp. The path to the updated library will be same as the currently
running executable. This is disabled and requires the option `--force` if uamp
was installed from a repository and not from github. The update mode is
selected in configuration.

This may require sudo.

Update accepts the following options:

`-h`, `-?`, `--help`
  Shows help for update. If this is present, uamp will not update and only show
  the help.

`--enabled`
  Checks if self update is enabled. Prints either `yes` or `no`.

`-f`, `--force`
  Force the update even if it has been disabled.

`--remote` *remote*
  Select remote repository for the update. If not specified, value from config is
  used.

`--man`
  Do install man pages. By default man pages are enabled to install on unix
  (linux). On windows the path to man pages is unspecified so it will not work.

`--no-man`
  Disable installing man pages.

`-m` *mode*, `--mode` *mode*
  Choose update mode. *mode* may be:

  - `tag`, `latest-tag`, `LatestTag`: Update to the latest tag on the remote
    repository.
  - `commit`, `latest-commit`, `LatestCommit`: Update to the latest commit on
    the main branch on the remote repository.
  - `branch=`*branch*, `Branch=`*branch*: Update to the latest commit on the
    given branch on the remote repository.

### Action `internal`

`internal` `tab-complete` *opt-index* *uamp-bin* [*opt*] ...

CLI ment to be used internally with integrations. This mode cannot be ended
with `--`. The only internal integration is tab completion.

Integrations:

`tab-complete` *opt-index* *uamp-bin* [*opt*] ...
  Gets tab completion suggestions for uamp. *uamp-bin* is path to uamp. This is
  here only to simplify implementation of integrations, and is actually
  ignored.

  *opt* are command line arguments for uamp for which the completion will be
  generated. The exact argument for which the completion should be generated
  is given by *opt-index*.

  All arguments after argument given by *opt-index* are ignored.

`install` [*flags*]
  Install uamp according to the flags. Must be run with CWD being the uamp repository.

  This is the default installation with no flags on linux. On other OSs, you
  should specify at least `--exe`.

  Available flags:

  `--man` `true`|`false`
    Enable/disable man page installation.

    Even if enabled, does nothing on non-unix systems.

  `--root` *path*
    Specify root (fakeroot) for the installation. By default, this is the
    system root.

  `--exe` *path*
    Specify path at which to place the installed executable. This is always
    relative to `root` even if the path is absolute. By default this is
    `/usr/bin/uamp` which may not be the correct place for other OSs than
    linux.

`open` [*audio-files*]
  Play the given audio files in a running instance. If there is no runing
  instance, start new one. If there are no audio paths, open the web client.

### Message control

`pp`[`=`*play-state*], `play-pause`[`=`*play-state*]
  Play/Pause playback. Without *play-state* toggles between the states.

  *play-state* can have values:

  - `play` the playback will play.
  - `pause` the playback will pause.
  
`stop`
  Stop the playback.
  
  Song will stop playing and seek to the start.

`vu`[`=`*volume*], `vol-up`[`=`*volume*], `volume-up`[`=`*volume*]
  Increases the volume by amount given by *volume*. If the amount was not given
  increase by the default amount given in configuration. The actual volume is
  clamped to value from `0` to `1`.

`vd`[`=`*volume*], `vol-down`[`=`*volume*], `volume-down`[`=`*volume*]
  Decreases the volume by amount given by *volume*. If the amount was not given
  decrease by the default amount given in configuration. The actual volume is
  clamped to value from `0` to `1`.

`ns`[`=`*N*], `next-song`[`=`*N*]
  Jump to the *N*th next song in the playlist. If not specified, *N* is `1`.

`ps`[`=`*N*], `previous-song`[`=`*N*]
  Jump to the *N*th previous song in the playlist. If not specified, *N* is
  `1`.

`pj`[`=`*N*], `playlist-jump`[`=`*N*]
  Jump to the *N*th song in the playlist. The value will be clamped to value
  from `0` to playlist length. The first song in the playlist has index `0`.
  If not specified, *N* is `0`.

`v=`*volume*, `vol=`*volume*, `volume=`*volume*
  Set volume to *volume*. *volume* must be value from `0` to `1`.

`mute`[`=`*B*]
  Mute/Unmute. If *B* is not specified, toggle between the states.

  *B* may be:

  - `true` - mute.
  - `false` - unmute.

`p`[`=`*audio-files*], `play`[`=`*audio-files*]
  Load the audio files given by *audio-file* as temporary song into uamp and
  push it as new playlist to the playlist stack.

  *audio-files* is comma separated list of paths.

`load-songs`[`=`[`l`|`r`][`-`|`e`|`n`|`m`]]
  Load new songs to library from folders specified in configuration. The value
  specifies load mode and what should be done with any of newly loaded songs.
  If not specified, defaults from playlist/configuration are used.

  There are the following load modes:

  - `l` - don't remove songs from library with invalid paths.
  - `r` - remove songs from library with invalid paths.

  And there are the following modes for adding new songs to playlist:

  - `-` - don't add the new songs to the playlist.
  - `e` - add the new songs to the end of the playlist.
  - `n` - add the new songs as next (after the current song) in the playlist.
  - `m` - mix the new songs randomly into the unplayed part of the playlist.

`remove-from-library=`*query*
  Remove songs given by query from library. The files are not deleted. The
  songs may be readded with next library load.

`shuffle`, `shuffle-playlist`
  Shuffles the current playlist.

  If `shuffle_current` in configuration is set to `true`, the current song will
  be shuffled into the playlist, and so the playlist position will likely not be
  `0`.

  If `shuffle_current` in configuration is set to `false`, the current song
  will be moved to index `0` in the playlist.

  Difference from `sort=rng` is that `sort=rng` will not respect the config
  setting `shuffle_current`.

`sort=`*order*, `sort-playlist=`*order*
  Sort the current playlist according to criteria given in *order*. See *Format
  order* for more information. The current song will not change, but the index
  of it in playlist will be likely to change.

  `sort=rng` will not respect the setting `shuffle_current`. If you want to
  randomy shuffle the playlist and respect the setting, use `shuffle`.

`x`, `exit`, `close`
  Exit uamp.

`seek=`*timestamp*, `seek-to=`*timestamp*
  Seek to the given *timestamp* within the current song. For the format of
  *timestamp* see *Format duration/timestamp*.

`ff`[`=`*duration*], `fast-forward`[`=`*duration*]
  Fast forward in current song by the given *duration*. If *duration* is not
  specified, fast forward by the default amount given in configuration.

  See *Format duration/timestamp for more info about the format of *duration*.

`rw`[`=`*duratoin*], `rewind`[`=`*duration*]
  Rewind the current song by the given *duration*. If *duration* is not
  specified, rewdind by the default amount given in configuration.

  See *Format duration/timestamp for more info about the format of *duration*.

`sp`[`=`*query*], `set-playlist`[`=`*query*]
  Set the current playlist to songs resulting from the *query*. If *query* is
  not specified, set the playlist to all songs in library in the order in which
  they are in library.

  See *Format query* for more information on *query*.

`push`[`=`*query*], `push-playlist`[`=`*query*]
  Push new playlist to the playlist stack. The playlist is created from the
  given *query*. If *query* is not specified, all songs are added to the new
  playlist.

  See *Format query* for more information on *query*.

`pc`[`=`*query*], `push-cur`[`=`*query*], `push-with-cur`[`=`*query*]
  Same as `push`. Additionaly, the current song will be moved from the old
  playlist to the start of the new playlist.

  See *Format query* for more information on *query*.

`pop`[`=`*N*], `pop-playlist`[`=`*N*]
  Pop *N* playlists from the playlist stack, but leave at least one. If *N* is
  0, leave only the last playlist. This will not change the playback status.

`flat`[`=`*N*], `flatten`[`=`*N*]
  Insert the current playlist into the next playlist on the stack at the
  position of current song. Pop the top playlist (the inserted). Do this *N*
  times. If not specified, *N* is `1`.

`q`[`=`*query*], `queue`[`=`*query*]
  Adds songs resulting from *query* to the end of the current playlist.

  See *Format query* for more information on *query*.

`qn`[`=`*query*], `queue-next`[`=`*query*], `play-next`[`=`*query*]
  Insert songs resulting from *query* into the current playlist after the
  current song.

  See *Format query* for more information on *query*.

`save`
  Trigger save. Saves are lazy and this will do nothing if there is no change
  from the previous save. If the instance has disabled config saves, this will
  not save the configuration.

`al=`*alias*, `alias=`*alias*
  Invoke the given alias. Arguments to the alias are passed inside `{` and `}`
  and are separated by `,`. For example:

    al=palb{trench}

`spea`[`=`*alias*], `pl-end`[`=`*alias*], `playlist-end`[`=`*alias*],
`playlist-end-action`[`=`*alias*]
  Set playlist end action to the given alias invokation. *alias* is same as in
  `alias=`*alias*. If *alias* is not specified, unsets the playlist end action.

`pap`[`=`*add-policy*], `add-policy`[`=`*add-policy*],
`playlist-add-policy`[`=`*add-policy*]
  Sets the playlist add policy. *add-policy* is by default `none`.

  *add-policy* may be one of:

  - `-`, `none` - don't add newly loaded songs to playlist.
  - `e`, `end` - add newly loaded songs to the end of the playlist.
  - `n`, `next` - add newly loaded songs after the current song in the
    playlist.
  - `m`, `mix`, `mix-in` - mix the newly loaded songs into the unplayed part
    of the playlist.

`restart`[`=`*binary-path*]
  Restart the uamp instance. Without the argument, uamp will use its current
  executable. If *binary-path* is present, uamp will use its as the newly
  restarted binary of uamp.
  
`rps=`*order*, `reorder-playlist-stack=`*order*
  Reorders the playlist stack according to the given order. The order is comma
  separated list of indexes into the stack where the index 0 is the current
  playlist. The first index in the order will be the new current playlist and
  the nest will follow. The indexes that are not in the order will be moved to
  the bottom of the stack without changing their relative order.
  
  For example the order `3,2,1,0` will reverse the top 4 playlists in the
  stack.

## FORMATS

This section describes formats referenced in other parts of this document.

### Format duration

[*days*`d`][*hours*]`:`[*minutes*]`:`[*seconds*][`.`*frac*]

[*days*`d`][[*minutes*]`:`][*seconds*][`.`*frac*]

Duration describes duration in time. It has precision from years to
nanoseconds. If you are unsure, you can just type the seconds as you are used
to (e.g `120.5` for two minutes and half a second).

*days* describe the number of days in the duration. `1d` is equivalent to
`24::`.

*hours* describe to number of yours in the duration. `1::` is equivalent to
`60:`.

*minutes* describe the number of minutes in the duration. `1:` is equivalent to
`60`.

*seconds* describe the number of whole seconds.

*frac* describes the decimal part of seconds. Only the 10 most significant
digits are considered. The 9 most significant digits are stored precisely and
the 10th digit will be rounded.

Even though the convention may suggest that it is necesary to use the largest
component possible, it is not required. All the following values are valid and
have the same value: `1d` == `24::` == `1440:` == `86400`.

### Format filter

*field*[*:*[`/`]*value*[`/`]]

`{`*filter*`}`

*filter*`+`*filter*

*filter*`.`*filter*

Filter is used to filter list of songs. Basic filter consists of *field* that
will be matched, the matching mode *:* and the pattern given in *value*. Some
field don't have *value* and some ignore the matching mode *:*. If value should
contain characters that would be normally interpreted, you can enclose it with
`/`. If enclosed you can use `//` to represent single `/`. Several such filters
may be joined together using `+` (or) or `.` (and). `.` (and) is evaluated
first. The precedence of these operators may be modified with brackets `{` and
`}`.

Here is list of supported fields to match and their meaning:

`any`
  All songs pass this filter.

`none`
  No songs pass this filter.

`s`*:pattern*, `an`*:pattern*, `any-name`*:pattern*
  Matches all songs where either title, artist or album matches *pattern* in
  mode *:*.

`n`*:pattern*, `tit`*:pattern*, `title`*:pattern*, `name`*:pattern*
  Matches all songs where the title matches *pattern* in mode *:*.

`p`*:pattern*, `art`*:pattern*, `artist`*:pattern*, `performer`*:pattern*,
`auth`*:pattern*, `author`*:pattern*
  Matches all songs where the artist matches *pattern* in mode *:*.

`a`*:pattern*, `alb`*:pattern*, `album`*:pattern*
  Matches all songs where the album matches *pattern* in mode *:*.

`t:`*uint*, `trk:`*uint*, `track:`*uint*, `track-number:`*uint*
  Matches all songs where the track number is *uint*. The mode is ignored.

`d:`*uint*, `disc:`*uint*
  Matches all songs where the disc number is *uint*. The mode is ignored.

`y:`*int*, `year:`*uint*
  Matches all songs where the year is *uint*. The mode is ignored.

`g`*:pattern*, `genre`*:pattern*
  Matches all songs where the genre matches *pattern* in mode *:*.

These are the available pattern matching modes *:*:

`=`
  The string must match exactly.

`-`
  The string must contain exact match of the pattern.

`:`
  The lowercase ascii representation of the string without whitespace must
  equal the pattern.

`~`
  The lowercase ascii representation of the string without whitespace must
  contain the pattern.

`@` is not allowed in filters, so it must be escaped using `/`.

Example filter to match all songs where the album title is  `smoke+mirrors` or
`trench`:

    alb:/smoke+mirrors/+alb:trench

### Format order

[`<`|`>`|`/`|`\`|`~`][`+`|`-`]*field*

Order is used to sort songs in ascending or descending order using *field*.
The ascending or descending order is given by the first optional character. If
it is not given, uamp wil sort in ascending order. Uamp supports two ways of
ordering, simple or complex. The second optional character determines which of
these will be used. If not present, the default value from configuration will
be used.

In simple ordering only the actual field is considered. If the values are same
their order will be preserved. If complex ordering is enabled, same values may
be sorted according to other related fields.

Here is list of available fields for sorting:

`same`
  Don't change the order.

`rev`, `reverse`
  Reverse the order of the songs.

`rng`, `rand`, `random`, `randomize`
  Randomly shuffle the songs.

`path`
  Sort by the path to the audio file.

`n`, `tit`, `title`, `name`
  Sort by the title of the song.

`p`, `art`, `artist`, `performer`, `auth`, `author`
  Sort by the artist.

  If complex sorting is enabled, also sort by year, album name, disc and track
  number.

`a`, `alb`, `album`
  Sort by the album name.

  If complex sorting is enabled, also sort by disc and track number.

`t`, `trk`, `track`, `track-number`
  Sort by track number.

`d`, `disc`
  Sort by disc number.

  If complex sorting is enabled, also sort by track number.

`y`, `year`
  Sort by year.

  If complex sorting is enabled, also sort by album name, disc and track
  number.

`len`, `length`
  Sort by the length of the song.

`g`, `genre`
  Sort by the genre of the song.

Prefix chars meaning:

`<`, `/`
  Sort in ascending order.

`>`, `\`, `~`
  Sort in descending order.

`+`
  Use complex sorting.

`-`
  Use simple sorting.

### Format query

[`,`*base*][*filter*][`@`*order*[`@`*unique*]]

Query is just combination of *base*, *filter*, *order* and *unique*. Song list
will be created from sources specified in *base*. This is by default all songs
in the main library. Than the songs are filtered by *filter* and than ordered
by *order*. See *Format base*, *Format filter*, *Format order* and
*Format unique* for more info.

### Format base

[*source* [`,` *source* [`,` ...]]]

Base is just comma separated list of sources for songs. The sources may be:

`lib`, `library`
  All songs in the main library. This is the default.

`tmp`, `temporary`
  All temporary songs in the library.

`all`
  All songs in the library.

`none`
  No songs.

*index*
  Playlist from the playlist stack at the *index*. 0 is the current playlist, 1
  is the next playlist and so on.
  
### Format unique

*unique*

Specifies that all resulting songs mus have some property unique. Songs that
would repeat this property will be removed. The property may be:

`u`, `unique`, `id`, `path`, `songs`
  Each song is unique.

`n`, `tit`, `title`, `name`
  Each song title is unique.

`p`, `art`, `artist`, `performer`, `auth`, `author`
  Each artist is unique.

`a`, `alb`, `album`
  Each album name is unique.

`t`, `trk`, `track`, `track-number`
  Each track number is unique.

`d`, `disc`
  Each disc number is unique.

`y`, `year`
  Each year is unique.

`len`, `length`
  Each song length is unique.

`g`, `genre`
  Each genre is uniuqe.

## FILES

*~/.config/uamp/config.json*
  Configuration file for uamp. Running instances of uamp will get notified when
  the file is notified. This file is expected to be edited by user.

*~/.config/uamp/library.json*
  Contains loaded library songs. The path to this file may be modified in
  config file. This file is not expected to be modified by user.

*~/.config/uamp/player.json*
  Contains information about playback. The path to this file may be modified in
  config file. This file is not expected to be modified by user.

*~/.local/share/uamp/log/*
  This folder contains log files generated by uamp. Uamp will automatically
  delete old log files after the time specified in configuration.

*~/.cache/uamp/cache/cover256/*
  This folder contains cached images of albums scaled so that their larger
  dimension is 256 pixels wide.

  Contents of this file may be safely deleted, but this may have slight impact
  on performance of uamp, and uamp will likely generate the images again.

**path to uamp**
  Uamp will watch its ouwn executable. If it is updated, uamp will restart
  itself.

## HTTP server

The HTTP server is primarly used for uamp to comunicate with running instances.
But the api is made to be usable also by other programs or manually.

These are the GET endpoints:

`/api/ctrl`
  Used for receiving control messages. The control messages are passed as query
  and their syntax is very simmilar to the CLI syntax.
  
  For example the command `uamp i pp q=a:clancy@a` is equivalent to HTTP GET
  request `/api/ctrl?pp&q=a%3Aclancy%40a`. Note that the values must be
  properly url encoded.
  
`/api/req`
  Used for one time requests for information. Again, the the syntax is very
  simmilar to the cli sintax.
  
  For example the command `uamp i info l=a:clancy@a` is equivalent to HTTP GET
  request `/api/req?info&l=a%3Aclancy%40a`. Note that the values must be
  properly url encoded.
  
  The server will reply with json same as the would be outputed with `--json`
  except that it will not send the version and all the responses will be in an
  array (even if it is only one response). Additionaly if only some of the
  responses will fail to be fulfiled, there may be error response in its place.
  The array with responses have the responses in the same order as the
  requested data.

`/api/sub`
  Endpoint for server sent events.

`/api/marco`
  Ping endpoint. It will always respond with `polo`.

`/api/img`
  Get image.

`/app`
  Application.

POST endpoints:

`/api/ctrl`
  Send control message.

## ENVIRONMENT

`RUST_LOG`
  Configures logger used by uamp. For decription on format of this variable
  see [flexi logger documentation](https://docs.rs/flexi_logger/latest/flexi_logger/struct.LogSpecification.html).

`VISUAL`
  This may contain editor used when uamp will try to edit file. This has the
  highest precedence when choosing editor.

`EDITOR`
  This may contain editor used when uamp will try to edit file. This has the
  second highest precedence when choosing editor (after `VISUAL`).

`RUST_BACKTRACE`
  When this is set to `0`, uamp will print whole backtrace when panics.

## DIAGNOSTICS

Invalid command line arguments are reported with user friendly message
describing the problem and highlighting the incorrect argument.

Uamp will fail to start new instance of HTTP server if the combination of
address and port is unavailable. If you are using the defaults, it is possible
that instance of uamp is already running and using that address and port.

Uamp will fail to communicate with existing instance if it will fail to connect
to that instance HTTP server. This may be because there is no running instance
of uamp, or the port and address of the instance is different than what was
uamp trying to connect to.

## EXAMPLES

### Useful commands

Here are some useful uamp commands to get you started:

`uamp run -d`
  Run uamp in detached mode (in background).

`uamp i x`
  Exit running instance of uamp.

`uamp i info`
  Print playback information.

`uamp i pp`
  Toggle play/pause.

`uamp i v=.5`
  Set the volume to 50%.

`uamp i p=song.mp3`
  Play the file `song.mp3` as new pushed playlist.

`uamp i al=reset-playlist`
  Set current playlist to all songs in the library shuffled and play. If the
  playlist ends, it will reshuffle all the songs again and star over. If new
  songs are added to the library, they will get mixed into the unplayed part of
  the playlist.

`uamp i al=palb{/trench/}`
  Push album with the name `trench` as new playlist. Once the album ends,
  playback will continue where it ended in the previous playlist.

`uamp i spea=repeat`
  Set the current playlist to repeat.

## SEE ALSO

uamp(5), [uamp github](https://github.com/BonnyAD9/uamp),
[uamp website](https://bonnyad9.github.io/uamp)

## BUGS

Found a bug? Please report it at
[github](https://github.com/BonnyAD9/uamp/issues).

## AUTHOR

[Jakub Antonín Štigler](https://github.com/BonnyAD9)
