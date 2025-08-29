# UAMP 5 2025-04-11

## NAME

uamp - Universal Advanced Music Player - JSON configuration file format.

## SYNTAX

Uamp configuration file uses the JSON format. The format's specification can
be found at [json.org](https://www.json.org).

## LOCATION

If the configuration is not present, it will be created on UNIX systems at:
1. `$XDG_CONFIG_HOME/uamp/config.json`
2. `$HOME/.config/uamp/config.json`

On windows it will be at:
1. `{FOLDERID_RoamingAppData}\uamp\config.json` which usually is
   `%APPDATA%\uamp\config.json`

On macOS it is at:
1. `$HOME/Library/Application Support/uamp/config.json`

## OPTIONS

This section documents all options that can be present in configuration file.

### Library

This section documents all options related to library.

`search_paths`
  This option is list of absolute paths to folders where uamp will search for
  music. By default uamp uses the default music folder on your system.

  On UNIX systems, default music folder is:
  1. `$XDG_MUSIC_DIR`

  On windows it is:
  1. `{FOLDERID_Music}` which usually is `%USERPROFILE%\Music`

  On macOS the folder is:
  1. `$HOME/Music`

  Example default value on linux:

    "search_paths": [
        "/home/alice/music"
    ]


`audio_extensions`
  This is list of file extensions (without dot) that uamp will recognize as
  audio files. It is used to speed up library load by not examining files with
  other file extensions.

  The default value is:

    "audio_extensions": [
        "flac",
        "mp3",
        "m4a",
        "mp4"
    ]

`recursive_search`
  If this is set to `true`, uamp will search for songs in library paths
  recursively (traversing all subdirectories). This may be slow in some cases,
  but it is usually the expected behaviour.

  Default value:

    "recursive_search": true

`update_library_on_start`
  If this is set to `true`, uamp will search library paths every time on
  startup. This was slow in previous versions, but it has been sice optimized
  so unless you have really huge library of songs, there is no reason to
  disable this. (My library of ~3000 songs loads in imperceptible time).

  Default value:

    "update_library_on_start": true

`remove_missing_on_load`
  Uamp caches(/stores) its library in library file `library.json`. The file
  contains all songs in the library with paths to the files and relevant
  extracted information. This library is updated on start if
  `update_library_on_start` is set to `true` or when user requests it. This
  option decides what should be done if path to song (audio file) from the
  library no longer exists.

  If set to `true` uamp will remove nonexisting songs from library. If set to
  `false` the songs will remain in library (but won't be able to play, because
  their audio files no longer exist).

  You may want to set this to `false` if your library is on portable media that
  is not connected to your PC all the time.

  When loading on user request from CLI, this default value may be overriden.

  Default value:

    "remove_missing_on_load": true

### Playlists

This section documents options related to manipulating playlists.

`control_aliases`
  This list defines aliases. Alias is a set of control messages that will be
  used in its place. It can be used with the contol message `alias`. See
  *uamp(1)* for list of all control messages. Aliases defined in this list may
  be simple collections of other control messages, or they may have arguments.

  Simple alias will contain invokations of control messages the same was as it
  would be written in cli after e.g. `uamp instance`.

  If alias has arguments, the arguments are specified with `{arg1,arg2}:`
  before the control messages invocations. `arg1` and `arg2` are names of the
  arguments. The arguments may be referenced in the control messages with `${}`
  (e.g `${arg1}`). The value of the argument is placed directly at that
  position before the control messages are fully parsed.

  The default value is:

    "control_aliases": {
        "pcont": "pop 'pp=play'",
        "repeat": "'pj=0' 'pp=play' 'spea=repeat'",
        "repeat-once": "'pj=0' 'pp=play' spea",
        "endless-mix":
            "'sp=any@rng' 'pj=0' 'pp=play' 'pap=m' 'spea=endless-mix'",
        "palb": "{name}:'push=a:${name}@+a' 'pp=play' 'spea=pcont'"
    }

  The order of the aliases doesn't matter.

  As you can see, uamp has by default some useful predefined aliases. Other
  players nay have specific actions for what uamp has just defined alias.

  The default aliases have meaning:

  - `pcont`: Useful as playlist end action. Pop the current playlist and
    continue playing the next. If there is no next playlist, play from start.
  - `repeat`: Useful as playlist end action. Repeat the current playlist
    indefinitely.
  - `repeat-once`: Useful as playlist end action. Repeat the current playlist
    once, and than end playback.
  - `endless-mix`: Play random songs indefinitelly.
  - `palb`: Play album with the given name. Than continue with the previous
    playback.

`default_playlist_end_action`
  This sets alias invocation that will be set as default playlist end action.
  If this is `null`, playlist will by default have no end action.

  Default value:

    "default_playlist_end_action": null

`simple_sorting`
  Uamp supports two types of sorting songs. Simple and complex. This will
  select the default behaviour if it is not specified in the sorting operation.
  `true` - use simple sorting, `false` - use complex sorting.

  When using simple sorting, uamp will sort only by the given field. If two
  songs have same value in the field, their relative position will stay the
  same.

  When using complex sorting, songs with same field will additionally be sorted
  by other fields specific to the primary field. For more info about sorting
  see *uamp(1)* section *Format order*.

  Default value:

    "simple_sorting": false

`shuffle_current`
  This changes the behaviour of the control message `shuffle`. If it is set to
  `true`, `shuffle` will randomly shuffle all the songs in playlist. If it is
  set to `false`, the currently playing song will be moved to the first
  position in the playlist after shuffling.

  Default value:

    "shuffle_current": true

### Playback

This section contains options related to playback.

`play_on_start`
  When this is set to  `true`, uamp will continue playing when it starts.

  Default value:

    "play_on_start": false

`volume_jump`
  This setting specifies the default change of volume. If you use the message
  `volume-up` or `volume-down` without specifying the amount, this amount will
  be used.

  The value has no unit, it is value in range from 0 to 1.

  Default value (2.5 %):

    "volume_jump": 0.025

`save_playback_pos`
  This determines whether uamp will retain position within current track after
  exiting.

  The available values are:
  - `"Never"`: uamp will never save position within current song for the
    current playlist.
  - `"OnClose"`: uamp will save position within current song when it exits but
    not on periodic saves.
  - `"Always"`: uamp will always save position within current song.

  Default value:

    "save_playback_pos": "OnClose"

`fade_play_pause`
  When you play or pause playback when something loud is playing, it is usually
  not plesent when it stops immidietely. When this is set to nonzero value,
  uamp will change smoothly decrease the volume to 0 before pausing, and it
  will smoothly transition from volume 0 to the set volume before playing.

  This setting is the duration of the transition. For more info about its
  format see *Format duration*.

  Default value (0.15 seconds):

    "fade_play_pause": "00:00.15"

`gapless`
  When encoding audio, encoders sometimes insert small silence before or after
  the audio. If this is set to `true`, uamp will configure its decoder to
  automatically remove this silence.

  Uamp also prefetches the next song if the current is ending so that there is
  no gap between songs. But the prefetching can be used only if the channel
  count and sample rate of the two consecutive songs match. If they don't match
  the audio device has to be reconfigured and this will result in small gap
  that may be noticable to some listeneres if the two songs are ment to follow
  up each other. This is usually not problem though, because songs that are
  meth to follow up each other will come from the same source and so they will
  have the same number of channels and the same sample rate (e.g. songs from
  the same album ripped from the same cd).

  Default value:

    "gapless": true

`seek_jump`
  This setting specifies the default amount to seek by if it is not specified
  in the `fast-forward` or `rewind` messages.

  Default value (10 seconds):

    "seek_jump": "00:10"

`previous_timeout`
  This option determines the behaviour of moving to the previous song. If this
  is not `null`, uamp may jump to the start of the current song instead and
  will jump to the previous song only if you go to the previous song twice in a
  row.

  The value of this setting determines the position within song when this
  behaviour changes. If you jump to the previous song when current song
  position is less than `previous_timeout`, uamp will jump to the previous
  song. Otherwise uamp will jump to the start of this song instead. The value
  may be `null` or duration. For the format of duration see *Format duration*.

  Default value (disabled):

    "previous_timeout": null

### Server

This section contains options related to the server created by uamp running in
background.

`server_address`
  Uamp creates HTTP server for communication across uamp instances. This is the
  way that you can control uamp that is running in background using terminal
  commands. This is the address of that server. The instance that runs the
  player (e.g. in background) will create server at this address. Instances
  that connect to the server will than use this as default address to connect
  to when trying to communicate with running instance of uamp.

  The default value is:

    "server_address": "127.0.0.1"

`port`
  When uamp creates or connects to uamp server (see `server_address`) it will
  use this port.

  The default value is:

    "port": "8267"

`enable_server`
  Currently uamp has no UI and so the TCP server is necessary. This mode will
  allow the user to disable the server for when uamp is run with UI. For non-UI
  modes, this setting has no effect and server is always enabled.

  Because currently uamp has no UI mode, this setting has no effect.

  Default value:

    "enable_server": true

`skin`
  Defines path for the skin for HTTP GUI. Should be absolute for future
  backwards compatibility.

  Default value:

  On unix systems, default path is:
  1. `$XDG_DATA_HOME/uamp/skins/default-uamp.tar`
  2. `$HOME/.local/share/uamp/skins/default-uamp.tar`
    
  On macOS, default path is:
  1. `$HOME/Library/Application Support/uamp/skins/default-uamp.tar`
  2. `/Users/Alice/Library/Application Support/uamp/skins/default-uamp.tar`

  On windows, default path is:
  1. `{FOLDERID_LocalAppData}\uamp\skins\default-uamp.tar` which usually is
     `%appdata%\uamp\skins\default-uamp.tar`

  Example value on linux:

    "skin": "/home/alice/.local/share/uamp/skins/default-uamp.tar"
    
`system_player`
  When enabled, uamp will integrate with the system as media player.

  This is currently supported only on linux where this uses mpris.

  Default value:

    "system_player": true

### Update

This section controls options related to uamp self update.

`update_mode`
  Determines how uamp will update. It may have the folowing values:

  - `"LatestTag"`: update to the latest tag in the remote repository.
  - `"LatestCommit"`: update to the latest commit in the remote repository
    master branch.
  - `{ "Branch": "branch" }`: update to the latest commit on the branch
    `branch`.

  Default value:

    "update_mode": "LatestTag"

`update_remote`
  Remote git repository from which updates will be downloaded.

  Default value:

    "update_remote": "https://github.com/BonnyAD9/uamp.git"

`auto_restart`
  Uamp will automatically restart running instance if it detects that the it
  updated (binary has changed). This setting can be used to disable this
  feature.

  Default value:

    "auto_restart": true

### Other

This section contains options that don't cleary fit to any of the previous
sections. These options are usually little more advanced to the previous
options.

`save_timeout`
  Uamp periodically saves any changes to its state. This option determines how
  ofter these save will occur. If there is no change in state of uamp, there
  will always be no save. The value is duration, for more info about its format
  see *Format duration*.

  When this is set to `null`, periodic saves will be disabled.

  Default value (every minute):

    "save_timeout": "01:00"

`delete_logs_after`
  Uamp stores its logs in your systems data folder. Logs are usually useful
  when there is some problem with uamp. They can tell the developer what went
  wrong, but there is no reason to store old logs, because they are irelevant
  to any new issue that may occur. This is why uamp will automatically delete
  old logs so that they don't take up space on your system. This setting will
  determine how old the log must be so that it is delted. The value is
  duration, for more info about its format see *Format duration*.

  Default value (3 days):

    "delete_logs_after": "3d00:00"

`client_image_lookup`
  If this is set to `true`, uamp will try to show song image in terminal when
  using instance message `info` or `show`.

  Default value:

    "client_image_lookup": true

`default_run_mode`
  Selects the default run mode of uamp when without arguments. It may have the
  following values:

  - `Background` run the uamp server. This will block.
  - `WebClient` run the uamp server if not running and open the web client.
    This won't block in most environments.

  Default value:

    "default_run_mode": "WebClient"

`web_client_command`
  Sets the command to run when opening web client. If null use the default
  browser.

  `${ADDRESS}` in the command will be replaced with the address to the client.

  Default value:

    "web_client_command": null

### Advanced

This section contains advanced options that normal user has no reason to
change.

`library_path`
  This is path to file where library will be saved. I don't really see why
  would user change this, but the option is available. By default it is stored
  alongside the configuration.

  If this is `null`, uamp will not save or load library.

  On UNIX systems, default library file path is:
  1. `$XDG_CONFIG_HOME/uamp/library.json`
  2. `$HOME/.config/uamp/library.json`

  On windows it is:
  1. `{FOLDERID_RoamingAppData}\uamp\library.json` which usually is
     `%APPDATA%\uamp\library.json`

  On macOS the file is:
  1. `$HOME/Library/Application Support/uamp/library.json`

  Example default value on linux:

    "library_path": "/home/alice/.config/uamp/library.json"

`player_path`
  This is path to file where player will store its playlist and state such as
  volume or song position. The playlist references information in library file
  and so it cannot be transferred to be used another library (even if it is on
  the same system). I don't really see why would user change this, but the
  option is available. By default it is stored alongside the library and
  configuration.

  If this is `null`, uamp will not save or load playback information.

  On UNIX systems, default player file path is:
  1. `$XDG_CONFIG_HOME/uamp/player.json`
  2. `$HOME/.config/uamp/player.json`

  On windows it is:
  1. `{FOLDERID_RoamingAppData}\uamp\player.json` which usually is
     `%APPDATA%\uamp\player.json`.

  On macOS the file is:
  1. `$HOME/Library/Application Support/uamp/player.json`

  Example default value on linux:

    "player_path": "/home/alice/.config/uamp/player.json"

`cache_path`
  This is path to folder where uamp will store precomputed data such as scaled
  images. Deleting the folder will only temporary impact performance. Uamp will
  recreate the folder and use it if it is not created. By default it uses the
  systems cache folder.

  On UNIX systems, default cache folder is:
  1. `$XDG_CACHE_HOME$/uamp/cache`
  2. `$HOME/.cache/uamp/cache`

  On windows it is:
  1. `{FOLDERID_LocalAppData}\uamp\cache`

  On macOS the file is:
  1. `$HOME/Library/Caches/uamp/cache`

  Example default value on linux:

    "cache_path": "/home/alice/.cache/uamp/cache"

`$schema`
  This link to json schema of the configuration document. It is not read by
  uamp, but it is set by uamp when it writes the file.

  Default value:

    "$schema": "https://raw.githubusercontent.com/BonnyAD9/uamp/master/other/json_schema/config_schema.json"

## Formats

This section describes formats of values referenced in previous section.

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

## EXAMPLES

To see the default configuration you can use the command `uamp conf --default`.
The default configuration may be on a linux system:

```
{
    "$schema": "https://raw.githubusercontent.com/BonnyAD9/uamp/master/other/json_schema/config_schema.json",
    "search_paths": [
        "/home/kubas/music"
    ],
    "library_path": "/home/kubas/.config/uamp_debug/library.json",
    "player_path": "/home/kubas/.config/uamp_debug/player.json",
    "cache_path": "/home/kubas/.cache/uamp_debug/cache",
    "audio_extensions": [
        "flac",
        "mp3",
        "m4a",
        "mp4"
    ],
    "server_address": "127.0.0.1",
    "control_aliases": {
        "pcont": "pop 'pp=play'",
        "endless-mix": "'sp=any@rng' 'pj=0' 'pp=play' 'pap=m' 'spea=endless-mix'",
        "repeat-once": "'pj=0' 'pp=play' spea",
        "repeat": "'pj=0' 'pp=play' 'spea=repeat'",
        "palb": "{name}:'push=a:${name}@+a' 'pp=play' 'spea=pcont'"
    },
    "default_playlist_end_action": null,
    "update_mode": "LatestTag",
    "update_remote": "https://github.com/BonnyAD9/uamp.git",
    "simple_sorting": false,
    "play_on_start": false,
    "shuffle_current": true,
    "recursive_search": true,
    "update_library_on_start": true,
    "remove_missing_on_load": true,
    "volume_jump": 0.025,
    "save_playback_pos": "OnClose",
    "save_timeout": "01:00",
    "fade_play_pause": "00:00.15",
    "gapless": true,
    "seek_jump": "00:10",
    "port": 33284,
    "delete_logs_after": "3d00:00",
    "enable_server": true,
    "previous_timeout": null,
    "client_image_lookup": true,
    "system_player": true,
    "auto_restart": true
}
```

## SEE ALSO

uamp(1), [uamp github](https://github.com/BonnyAD9/uamp),
[uamp website](https://bonnyad9.github.io/uamp)

## BUGS

Found a bug? Please report it at
[github](https://github.com/BonnyAD9/uamp/issues).

## AUTHOR

[Jakub Antonín Štigler](https://github.com/BonnyAD9)
