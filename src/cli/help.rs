use std::borrow::Cow;

use termal::{eprintacln, gradient, printmcln};

use super::Args;

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Parses help arguments.
pub fn help<'a>(args: &mut impl Iterator<Item = &'a str>, res: &mut Args) {
    res.should_exit = true;

    let arg = match args.next() {
        Some(a) => a,
        None => {
            help_all(res.stdout_color);
            return;
        }
    };

    help_version(res.stdout_color);

    let mut show_cmsg = false;
    let mut cmsg_shown = false;

    for arg in Some(arg).into_iter().chain(args) {
        match arg {
            "basic" => print_basic_help(res.stdout_color),
            "i" | "instance" => {
                print_instance_help(res.stdout_color);
                show_cmsg = true;
            }
            "run" => {
                print_run_help(res.stdout_color);
                show_cmsg = true;
            }
            "all" | "elp" => print_help(res.stdout_color),
            "control-message" | "control-msg" | "cmsg" => {
                print_control_messages_help(res.stdout_color);
                cmsg_shown = true;
            }
            "--" => break,
            a => eprintacln!("{'m}warning: {'_}Invalid help option {a}"),
        }
    }

    if show_cmsg && !cmsg_shown {
        print_control_messages_help(res.stdout_color);
    }
}

/// Prints the whole help.
pub fn help_all(color: bool) {
    help_version(color);
    print_help(color);
}

/// Prints the help for the instance action.
pub fn help_instance(color: bool) {
    help_version(color);
    print_instance_help(color);
    print_control_messages_help(color);
}

/// Prints help for the run action.
pub fn help_run(color: bool) {
    help_version(color);
    print_run_help(color);
    print_control_messages_help(color);
}

/// Prints the help header and version.
pub fn help_version(color: bool) {
    let v = option_env!("CARGO_PKG_VERSION").unwrap_or("unknown");
    let signature: Cow<str> = if color {
        gradient("BonnyAD9", (250, 50, 170), (180, 50, 240)).into()
    } else {
        "BonnyAD9".into()
    };

    printmcln!(
        color,
        "Welcome in {'i g}uamp{'_} by {signature}{'_}
Version {v}
",
    )
}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

fn print_help(color: bool) {
    print_basic_help(color);
    print_instance_help(color);
    print_run_help(color);
    print_control_messages_help(color);
}

/// Prints the basic usage help
fn print_basic_help(color: bool) {
    printmcln!(
        color,
        "{'g}Usage:{'_}
  {'c}uamp{'_}
    Starts the background player.

  {'c}uamp {'gr}[{'dy}flags{'gr}] [{'db}action{'gr}] [-- [{'db}action{'gr}] \
  ...]{'_}
    Does the given actions.

{'g}Flags:{'_}
  {'y}-h  -?  --help{'_}
    Prints all of the help.

  {'y}-p  --port {'w}<port>{'_}
    Sets the port for the server comunication. If used when starting gui, it
    will disable config saves.

  {'y}-a  --address {'w}<address>{'_}
    Sets the server address for the comunication. If used when starting gui, it
    will disable config saves.

  {'y}--version{'_}
    Print the version.

  {'y}-I{'gr}[arg]{'_}
    Equivalent to `{'b}i {'gr}[arg]`.

  {'y}-R{'gr}[arg]{'_}
    Equivalent to `{'b}run {'gr}[arg]`.

  {'y}-H{'gr}[arg]{'_}
    Equivalent to `{'b}h {'gr}[arg]`.

  {'y}--color  --colour {'w}<(auto|always|never)>
  {'y}--color  --colour{'w}=<(auto|always|never)>{'_}
    Enable/disable color in stdout. This will apply for help only when
    specified before.

{'g}Actions:{'_}
  {'b}i  instance {'gr}[instance arguments] [--]{'_}
    Operates on a running instance of uamp.

  {'b}run {'gr}[run arguments] [--]{'_}
    Runs new instance of uamp.

  {'b}h  help {'gr}[help aguments] [--]{'_}
    Shows help. With no arguments whole help, with arguments only help specific
    to the given arguments. Possible arguments are:
    - `{'r}all  elp{'_}`                           print whole help.
    - `{'r}basic{'_}`                              print the basic help.
    - `{'r}i  instance{'_}`                        print help for
                                           {'b}instance{'_}.
    - `{'r}run{'_}`                                print help for {'b}run{'_}.
    - `{'r}control-message  control-msg  cmsg{'_}` print help for control
                                           messages.
    Note that using {'w}--{'_} without any of the help arguments will not print
    the whole help but only the help header.
",
    )
}

/// Prints the instance help
fn print_instance_help(color: bool) {
    printmcln!(
        color,
        "{'g}Instance usage:
  {'c}uamp {'b}i {'gr}[{'dy}flags{'gr}] [{'dr}messages{'gr}] [--]{'_}
    Sends the messages to a running instance of uamp using the server.

{'g}Instance flags:
  {'y}-h  -?  --help{'_}
    Prints the instance help.

  {'y}-p  --port {'w}<port>{'_}
    Sets the port on which is server of the instance.

  {'y}-a  --address {'w}<address>{'_}
    Sets address of the server of the instance.

{'g}Instance messages:{'_}
  Any {'g}control message{'_} or:

  {'r}info nfo{'_}
    Shows the info about the playback of the currently runing instance.

  {'r}query  list  l{'gr}[={'bold}<filter>{'_bold}]{'_}
    Print all songs that pass the filter. See {'w bold}filter{'_} in
    {'g}formats{'_} for more info. Without value, lists all songs.

  {'r}play p{'w}=<file path>{'_}
    Play the given file in the secondary playlist.
",
    );
}

fn print_run_help(color: bool) {
    printmcln!(
        color,
        "{'g}Run usage:
  {'c}uamp {'b}run {'gr}[{'dy}flags{'gr}] [{'dr}messages{'gr}] [--]{'_}
    Runs new instance of uamp. The given messages are executed on the new
    instance.

{'g}Run flags:
  {'y}-h  -?  --help{'_}
    Prints the run help.

  {'y}-d  --detach{'_}
    Runs uamp in detached mode without console.

  {'y}-p  --port {'w}<port>{'_}
    Sets the port number of server for the new instance. This will disable
    config saves for the new instance.

  {'y}-a  --address {'w}<address>{'_}
    Sets the server address of for the new instance. Thiss will disable config
    saves for the new instance.

{'g}Run messages:{'_}
  Any {'g}control message{'_}.
",
    )
}

fn print_control_messages_help(color: bool) {
    printmcln!(
        color,
        "{'g}Control messages:
  {'r}play-pause  pp{'gr}[=(play|pause)]{'_}
    Play or pause. When without argument, toggle.

  {'r}volume-up  vol-up  vu{'gr}[=<amount>]{'_}
    Increase the volume by the given amount. When without argument, increase by
    the default amount.

  {'r}volume-down  vol-down  vd{'gr}[=<amount>]{'_}
    Decrease the volume by the given amount. When without argument, decrease by
    the default amount.

  {'r}next-song  ns{'gr}[=<N>]{'_}
    Jump to the Nth next song. By default N is 1 (jump to next song).

  {'r}previous-song  ps{'gr}[=<N>]{'_}
    Jump to the Nth previous song. By default N is 1 (jump to previous song).

  {'r}playlist-jump  pj{'gr}[=<N>]{'_}
    Jump to the Nth song in the playlist. By default N is 0 (first song).

  {'r}volume  vol  v{'w}=<volume>{'_}
    Sets the volume to the given value. Value must be in range from 0 to 1.

  {'r}mute{'gr}[=(true|false)]{'_}
    Mute/Unmute. When without argument, toggle.

  {'r}load-songs{'gr}[=[l|r][-|e|n|m]]{'_}
    Look for new songs. This can be modifed with the load options of the form
    `{'i}[r|l][-|e|n|m]{'_}`:
      Remove songs with invalid path:
      - `{'i}r{'_}` also remove songs with invalid path.
      - `{'i}l{'_}` don't remove songs with invalid path.
      Add policy (overrides playlist property):
      - `{'i}-{'_}` don't add the songs to playlist.
      - `{'i}e{'_}` add new songs to the end of the queue.
      - `{'i}n{'_}` add new songs after the current song.
      - `{'i}m{'_}` randomly mix the songs in after the current song.

  {'r}shuffle-playlist  shuffle{'_}
    Shuffle the current playlist. The difference from {'r}sort{'w}=rng{'_} is
    that {'r}shuffle{'_} will respect the setting shuffle current.

  {'r}sort-playlist  sort{'w}={'bold}<order>{'_bold}{'_}
    Sorts the songs in the current playlist. See {'w bold}order{'_} in
    {'g}formats{'_} for more info.

  {'r}exit  close  x{'_}
    Gracefully close the instance.

  {'r}seek-to  seek{'w}=<timestamp>{'_}
    Seeks to the given timestamp.

  {'r}fast-forward  ff{'gr}[=<duration>]{'_}
    Seek forward by the given duration. Without argument seeks by the default
    duration.

  {'r}rewind  rw{'gr}[=<duration>]{'_}
    Seek back by the given duration. Without artument seeks by the default
    duration.

  {'r}set-playlist  sp{'gr}[={'bold}<query>{'_bold}]{'_}
    Loads subset as the current playlist. Without value for {'w}filter{'_}
    loads all songs. See {'w bold}query{'_} in {'g}formats{'_} for more info.

  {'r}push-playlist  push{'gr}[={'bold}<query>{'_bold}]{'_}
    Push new playlist on top of the current one. Without value for
    {'w}filter{'_} pushes all songs. See {'w bold}query{'_} in {'g}formats{'_}
    for more info.

  {'r}push-with-cur  push-cur  pc{'gr}[={'bold}<query>{'_bold}]{'_}
    Seamlessly push new playlist on top of the current one by moving the
    currently playing song to the start of the new playlist. See
    {'w bold}filter{'_} in {'g}formats{'_} for more info.

  {'r}pop-playlist  pop{'_}
    Remove the secondary playlist and restore the primary playlist.

  {'r}flatten  flat{'gr}[=<cnt>]{'_}
    Seamlessly insert the top playlist to the next playlist on the stack. The
    currently playing song doesn't change. Do this {'i}cnt{'_} times. The
    default value for {'i}cnt{'_} is 1. 0 means flatten the whole stack.

  {'r}queue  q{'gr}[={'bold}<query>{'_bold}]{'_}
    Adds songs to the end of the queue (current playlist). See {'w bold}filter
    {'_}in {'g}formats{'_} for more info about selecting songs to queue.
    Without value, queues all songs.

  {'r}play-next  queue-next  qn{'gr}[={'bold}<query>{'_bold}]{'_}
    Adds songs after the currently playing in the current playlist. See
    {'w bold}filter{'_} in {'g}formats{'_} for more info. Without value, queues
    all songs.

  {'r}save{'_}
    Triggers save (but saves only if there is change).

  {'r}alias  al{'w}=<alias name>{'gr}[`[`<arg1>,<arg2>,...`]`]{'_}
    Runs the actions given by the alias. You can pass arguments to the alias.

  {'r}playlist-end-action  playlist-end  pl-end  spea{'gr}[=<alias name>]{'_}
    Sets the playlist end action of the current playlist to actions specified
    by the alias. Without value, unsets the playlist end action.

  {'r}playlist-add-policy  add-polocy  pap{'gr}[=<add policy>]{'_}
    Sets the playlist add policy. It is one of the following:
    - `{'i}-  none{'_}`        don't add new songs to the playlist.
    - `{'i}e  end{'_}`         add new sobgs to the end of the playlist.
    - `{'i}n  next{'_}`        add new songs after the currently playing song.
    - `{'i}m  mix  mix-in{'_}` randomly mix the the songs after the currently
                       playing song.
    Without value it is the same as setting it to `{'i}none{'_}`.

{'g}Formats:
  {'w bold}query:{'_}
    Query is just combination of filter and order. It has the form:
      {'gr}[{'bold}<filter>{'_bold}][@[{'bold}<order>{'_bold}]]

  {'w bold}filter:{'_}
    Specifies how to filter songs. These are the kinds of filters:
      {'r}any{'_}
        All songs pass this filter.

      {'r}none{'_}
        No songs pass this filter.

      {'r}s an  any-name{'w}:<pattern>{'_}
        Matches all songs where either title, artist or album matches.

      {'r}n tit  title  name{'w}:<pattern>{'_}
        Matches all songs where title matches.

      {'r}p art  artist  performer  auth  author{'w}:<pattern>{'_}
        Matches all songs where the performer name matches.

      {'r}a alb  album{'w}:<pattern>{'_}
        Matches all songs where the album name matches.

      {'r}t trk  track  track-number{'w}:<uint>{'_}
        Matches all songs with the given track number.

      {'r}d disc{'w}:<uint>{'_}
        Matches all songs with the given disc number. {'w}0{'_} means no disc
        number.

      {'r}y  year{'w}:<int>{'_}
        Matches all songs with release within the given year.

      {'r}g  genre{'w}:<pattern>{'_}
        Matches all songs which genre that matches.

    Instead of `{'i}:{'_}` you can use different separator to change the
    comparison:
      `{'i}={'_}` The string must match exactly.
      `{'i}+{'_}` The string must contain the exact pattern.
      `{'i}:{'_}` The strings converted to lowercase ascii without whitespace
          must match.
      `{'i}~{'_}` The string converted to lowercase asci without whitespace
          must contain the pattern (also converted in the same way).

    You can combine filters using:
      `{'i}+{'_}`    Or - one of the filters must pass.
      `{'i}.{'_}`    And - Both of the filters must pass.
      `{'i}[  ]{'_}` Brackets to change the precedence.

    `{'i}.{'_}` (And) is evaluated first so `{'i}an:a+an:b.an:c{'_}` is the
    same as `{'i}an:a+[an:b.an:c]{'_}`.

    You can use `{'i}/{'_}` to enclose string literals. Inside the string
    literals you can use `{'i}//{'_}` to escape single /.

    @ is not allowed in filters so you need to use string literal to express
    this character in the filter.

    Example filters:
      `{'i}alb:/smoke+mirrors/+alb:trench{'_}`

  {'w bold}order:{'_}
    Specifies how to sort songs. It has this form:
      {'gr}[<|>|/|\\|~][+|-]{'w}<ord>{'_}

    The value of ord sets the parameter by which the songs are sorted. Possible
    values are:
      {'r}rev  reverse{'_}
        Reverse the songs.

      {'r}rng  rand  random  randomize{'_}
        Shuffle the songs.

      {'r}path{'_}
        Sort by the song path.

      {'r}n tit  title  name{'_}
        Sort by the song title.

      {'r}p art  artist  performer  auth  author{'_}
        Sort by the artist.

      {'r}a alb  album{'_}
        Sort by the album name.

      {'r}t trk  track  track-number{'_}
        Sort by the track number.

      {'r}d disc{'_}
        Sort by the disc number.

      {'r}y year  date{'_}
        Sort by the release date.

      {'r}len  length{'_}
        Sort by the length of the song.

      {'r}g genre{'_}
        Sort by the genre.

    You can alter the sorting with one of the following options (some
    parameters ignore some of this):
      `{'i}<  /{'_}`    sort in ascending order (this is the default).
      `{'i}>  \\  ~{'_}` sort in descending order.
      `{'i}+{'_}`       use complex sorting.
      `{'i}-{'_}`       use simple sorting.

    If the complexity of the sorting is not set, it will use the default from
    settings.
",
    )
}
