use std::borrow::Cow;

use pareg::Pareg;
use termal::{eprintacln, gradient, printmcln};

use crate::core::config::{APP_ID, VERSION_STR};

use super::Args;

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Parses help arguments.
pub fn help(args: &mut Pareg, res: &mut Args) {
    res.should_exit = true;

    if args.remaining().is_empty() {
        help_short(res.props.color);
        return;
    }

    help_version(res.props.color);

    let mut formats_header = false;

    while let Some(arg) = args.next() {
        match arg {
            "basic" => {
                print_basic_help(res.props.color);
                formats_header = false;
            }
            "i" | "instance" => {
                print_instance_help(res.props.color);
                formats_header = false;
            }
            "run" => {
                print_run_help(res.props.color);
                formats_header = false;
            }
            "cfg" | "conf" | "config" => {
                print_config_help(res.props.color);
                formats_header = false;
            }
            "sh" | "shell" => {
                print_shell_help(res.props.color);
                formats_header = false;
            }
            "internal" => {
                print_internal_help(res.props.color);
                formats_header = false;
            }
            "h" | "help" | "-h" | "-?" | "--help" => {
                print_help_help(res.props.color);
                formats_header = false;
            }
            "man" => {
                print_man_help(res.props.color);
                formats_header = false;
            }
            "update" => {
                print_update_help(res.props.color);
                formats_header = false;
            }
            "all" | "elp" => {
                print_help(res.props.color);
                formats_header = true;
            }
            "control-message" | "control-msg" | "cmsg" => {
                print_control_messages_help(res.props.color);
                formats_header = false;
            }
            "format" | "formats" => {
                print_formats_help(res.props.color, formats_header);
                formats_header = true;
            }
            "port" => {
                print_port_help(res.props.color, formats_header);
                formats_header = true;
            }
            "query" => {
                print_query_help(res.props.color, formats_header);
                formats_header = true;
            }
            "base" => {
                print_base_help(res.props.color, formats_header);
                formats_header = true;
            }
            "filter" => {
                print_filter_help(res.props.color, formats_header);
                formats_header = true;
            }
            "order" => {
                print_order_help(res.props.color, formats_header);
                formats_header = true;
            }
            "unique" => {
                print_unique_help(res.props.color, formats_header);
                formats_header = true;
            }
            "--" => break,
            a => eprintacln!("{'m}warning: {'_}Invalid help option {a}"),
        }
    }

    help_footer(res.props.color);
}

/// Prints the short help.
pub fn help_short(color: bool) {
    help_version(color);
    print_basic_help(color);
    help_footer(color);
}

/// Prints the help for the instance action.
pub fn help_instance(color: bool) {
    help_version(color);
    print_instance_help(color);
    help_footer(color);
}

/// Prints help for the run action.
pub fn help_run(color: bool) {
    help_version(color);
    print_run_help(color);
    help_footer(color);
}

/// Print help for configuration.
pub fn help_config(color: bool) {
    help_version(color);
    print_config_help(color);
    help_footer(color);
}

pub fn help_shell(color: bool) {
    help_version(color);
    print_shell_help(color);
    help_footer(color);
}

pub fn help_internal(color: bool) {
    help_version(color);
    print_internal_help(color);
    help_footer(color);
}

pub fn help_man(color: bool) {
    help_version(color);
    print_man_help(color);
    help_footer(color);
}

pub fn help_update(color: bool) {
    help_version(color);
    print_update_help(color);
    help_footer(color);
}

/// Print the help footer.
pub fn help_footer(color: bool) {
    printmcln!(
        color,
        " \
 “ {'i}The one who has found his life will lose it, and the
   one who has lost his life on My account will find it. {'_}”
                                           {'w bold}✝ Matthew 10:39{'_}
"
    )
}

/// Prints the help header and version.
pub fn help_version(color: bool) {
    let signature: Cow<str> = if color {
        gradient("BonnyAD9", (250, 50, 170), (180, 50, 240)).into()
    } else {
        "BonnyAD9".into()
    };

    printmcln!(
        color,
        "Welcome in {'i g}{APP_ID}{'_} by {signature}{'_}
Version {VERSION_STR}

uamp (short for Universal Advanced Music Player) is CLI only music player and
music library manager without UI. For more information about how uamp works see
{'i}uamp(1){'_}.
"
    )
}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

fn print_help(color: bool) {
    print_basic_help(color);
    print_instance_help(color);
    print_run_help(color);
    print_config_help(color);
    print_shell_help(color);
    print_internal_help(color);
    print_man_help(color);
    print_update_help(color);
    print_help_help(color);
    print_control_messages_help(color);
    print_formats_help(color, false);
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
    Sets the port for the server comunication. If used when starting instance,
    it will disable config saves. Value may be port number or port name. See
    `{'c}uamp {'b}h {'w bold}port{'_}` for more info.

  {'y}-a  --address {'w}<address>{'_}
    Sets the server address for the comunication. If used when starting
    instance, it will disable config saves.

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

  {'y}--print {'w}<(pretty|debug|json)>
    Select print mode. Default is pretty.

  {'y}--verbose
  -v{'gr}[verbosity]{'_}
    Set the verbosity. The default verbosity is 0. If this argument is present
    without specific verbosity the verbosity is set to 1. You may also specify
    any other (positive or negative) verbosity.

{'g}Actions:{'_}
  {'b}i  instance {'gr}[instance arguments] [--]{'_}
    Operates on a running instance of uamp. See `{'c}uamp {'b}h {'b}i{'_}` for
    more info.

  {'b}run {'gr}[run arguments] [--]{'_}
    Runs new instance of uamp. See `{'c}uamp {'b}h {'b}run{'_}` for more info.

  {'b}cfg  conf  config {'gr}[config arguments] [--]{'_}
    Edit/show configuration.

  {'b}sh  shell {'gr}[shell arguments] [--]{'_}
    Uamp integration with shell (e.g. custom tab completion).

  {'b}internal {'gr}[internal arguments]{'_}
    Used internally by uamp, but nothing will stop you from using it.

  {'b}man {'gr}[man arguments]{'_}
    See the manual page.

  {'b}update {'gr}[update arguments] [--]{'_}
    Self update uamp from github. Don't use this if you installed uamp with
    your distro package manager.

  {'b}h  help {'gr}[help aguments] [--]{'_}
    Shows help. With no arguments basic help. With arguments help for the given
    categories. Use `{'c}uamp {'b}h h{'_}` to get more info about help \
    categories.

    Use `{'c}uamp {'b}h {'r}all{'_}` to show the whole help.

For detailed description see {'i}uamp(1){'_} and {'i}uamp(5){'_} or do
`{'c}uamp {'b}man{'_}` and `{'c}uamp {'b}man {'w}5{'_}`. For more info about
uamp in general see {'b u}https://bonnyad9.github.io/uamp/{'_}.
",
    )
}

fn print_help_help(color: bool) {
    printmcln!(
        color,
        "{'g}Help usage:
  {'c}uamp {'b}h {'gr}[{'dr}categories{'gr}] [--]{'_}
    Print help for all the given categories.

  {'g}Help categories:
    {'r}all  elp{'_}
      Whole help.

    {'r}basic{'_}
      Basic help for general usage.

    {'r}i  instance{'_}
      Help for {'b}instance{'_} action.

    {'r}run{'_}
      Help for {'b}run{'_} action.

    {'r}cfg  conf  config{'_}
      Help for {'b}config{'_} action.

    {'r}sh  shell{'_}
      Help for {'b}shell{'_} action.

    {'r}internal{'_}
      Help for internal actions.

    {'r}h  help  -h  -?  --help{'_}
      Show all help categories and usage.

    {'r}control-message  control-msg  cmsg{'_}
      Help for all {'g}control messages{'_}.

    {'r}format  formats{'_}
      Help for all {'g}formats{'_}.

    {'r}port{'_}
      Help for {'w bold}port{'_} format.

    {'r}query{'_}
      Help for {'w bold}query{'_} format.
      
    {'r}base{'_}
      Help for the format of {'w bold}base{'_}.

    {'r}filter{'_}
      Help for {'w bold}filter{'_} format.

    {'r}order{'_}
      Help for {'w bold}order{'_} format.
      
    {'r}unique{'_}
      Help for {'w bold}unique{'_} format.
",
    );
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
    Sets the port on which is server of the instance. It may be port number or
    port name. See `{'c}uamp {'b}h {'w bold}port{'_}` for more info.

  {'y}-a  --address {'w}<address>{'_}
    Sets address of the server of the instance.

  {'y}--verbose
  -v{'gr}[verbosity]{'_}
    Set the verbosity. The default verbosity is 0. If this argument is present
    without specific verbosity the verbosity is set to 1. You may also specify
    any other (positive or negative) verbosity.

{'g}Instance messages:{'_}
  Any {'g}control message{'_} (See `{'c}uamp {'b}h {'g}cmsg{'_}`) or:

  {'r}info nfo{'gr}[=[-<before>]..[<after>]]{'_}
    Shows the info about the playback of the currently runing instance.
    {'w}before{'_} and {'w}after{'_} tells the number of songs to show in the
    current playlist before the current song and after the current song
    respectively. This is by default `{'i w}-1..3{'_}`.

  {'r}show{'gr}[=[-<before>]..[<after>]]{'_}
    Same as {'r}info{'_} but will also clear the screen before printing.

  {'r}query  list  l{'gr}[={'bold}<query>{'_bold}]{'_}
    Print all songs that pass the query. Without value, lists all songs. The
    amount of printed information is affected by verbosity. See
    `{'c}uamp {'b}h {'w bold}query{'_}` for more info.

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
    config saves for the new instance. It may be port number or port name. See
    `{'c}uamp {'b}h {'w bold}port{'_}` for more info.

  {'y}-a  --address {'w}<address>{'_}
    Sets the server address of for the new instance. Thiss will disable config
    saves for the new instance.

{'g}Run messages:{'_}
  Any {'g}control message{'_}. See `{'c}uamp {'b}h {'g}cmsg{'_}` for more info.
",
    )
}

fn print_config_help(color: bool) {
    printmcln!(
        color,
        "{'g}Config usage:
  {'c}uamp {'b}conf {'gr}[--]{'_}
    Open configuration in the default editor.

  {'c}uamp {'b}conf {'gr}[{'dy}flags{'gr}] [--]{'_}
    Do what the flags say about the configuration.

{'g}Config flags:
  {'y}-h  -?  --help{'_}
    Prints the config help.

  {'y}-e  --edit  --edit-file{'_}
    Open configuration in the default editor.

  {'y}-p  --path  --print-path{'_}
    Print path to the default config file location.

  {'y}--default{'_}
    Print the default configuration to stdout.
    
  {'y}--aliases{'_}
    Print all aliases sorted alphabetically. If the verbosity is 1 or more, the
    definitions of aliases will also be printed.

  {'y}--verbose
  -v{'gr}[verbosity]{'_}
    Set the verbosity. The default verbosity is 0. If this argument is present
    without specific verbosity the verbosity is set to 1. You may also specify
    any other (positive or negative) verbosity.
",
    )
}

fn print_shell_help(color: bool) {
    printmcln!(
        color,
        "{'g}Shell usage:
  {'c}uamp {'b}sh {'gr}[{'dy}flags{'gr}] [{'dr}integrations{'gr}] [--]{'_}
    Get shell integrations.

{'g}Shell flags:
  {'y}-h  -?  --help{'_}
    Prints the shell help.

  {'y}-s  --script{'_}
    Print the long script instead of short script runner.

{'g}Shell integrations:
  {'r}tab  tab-completion{'_}
    Custom tab completion for uamp.
",
    )
}

fn print_internal_help(color: bool) {
    printmcln!(
        color,
        "{'g}Internal usage:
  {'c}uamp {'b}internal {'w}({'y}-h{'_}|{'y}-?{'_}|{'y}--help{'w}){'_}
    Show internal help.

  {'c}uamp {'b}internal {'r}tab-complete {'w}<arg-idx> <uamp-bin> \
           {'gr}[uamp args]{'_}
    Get uamp tab completion for the given arguments.
",
    )
}

fn print_man_help(color: bool) {
    printmcln!(
        color,
        "{'g}Man usage:
  {'c}uamp {'b}man {'gr}[{'dy}flags{'gr}] {'gr}[{'dr}page{'gr}]{'_}
    See manual page. If there is no manual page, open the main man page.

{'g}Man flags:
  {'y}-h  -?  --help{'_}
    See help for man pages.

  {'y}-p  --print{'_}
    Print the man page in troff format instead of showing the man page with
    man.

{'g}Man pages:
  {'r}1  cli{'_}
    Man page about the cli. Basically detailed help.

  {'r}5  cfg  conf  config{'_}
    Man page about configuration.
"
    );
}

fn print_update_help(color: bool) {
    printmcln!(
        color,
        "{'g}Update usage:
  {'c}uamp {'b}update {'gr}[{'dy}flags{'gr}] [--]{'_}
    Self update uamp. Don't use this if you installed uamp with your distros
    package manager.

{'g}Update flags:
  {'y}-h  -?  --help{'_}
    See help for update.

  {'y}-f  --force{'_}
    Force self update even if it was disabled. DONT DO THIS unless you know
    what you are doing.

  {'y}--remote {'w}<remote>{'_}
    Choose remote repository from which to update uamp.

  {'y}--man{'_}
    Enable installing man pages. On unix (linux) this is the default.

  {'y}--no-man{'_}
    Disable installing man pages.

  {'y}-m  --mode {'w}<mode>{'_}
    Update uamp in the given mode. Mode may be:
    - `{'i}tag  latest-tag  LatestTag{'_}`          update to the latest tag on
                                            the remote repository.
    - `{'i}commit  latest-commit  LatestCommit{'_}` update to the latest commit
                                            on the remote repository.
"
    );
}

fn print_control_messages_help(color: bool) {
    printmcln!(
        color,
        "{'g}Control messages:
  {'r}play-pause  pp{'gr}[=(play|pause)]{'_}
    Play or pause. When without argument, toggle.
    
  {'r}stop{'_}
    Stops the playback. Playing will start from the start of the track.

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
    Sorts the songs in the current playlist. See
    `{'c}uamp {'b}h {'w bold}order{'_}` for more info.

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
    loads all songs. See `{'c}uamp {'b}h {'w bold}query{'_}` for more info.

  {'r}push-playlist  push{'gr}[={'bold}<query>{'_bold}]{'_}
    Push new playlist on top of the current one. Without value for
    {'w}filter{'_} pushes all songs. See `{'c}uamp {'b}h {'w bold}query{'_}`
    for more info.

  {'r}push-with-cur  push-cur  pc{'gr}[={'bold}<query>{'_bold}]{'_}
    Seamlessly push new playlist on top of the current one by moving the
    currently playing song to the start of the new playlist. See
    `{'c}uamp {'b}h {'w bold}query{'_}` for more info.

  {'r}pop-playlist  pop{'_}
    Remove the secondary playlist and restore the primary playlist.

  {'r}flatten  flat{'gr}[=<cnt>]{'_}
    Seamlessly insert the top playlist to the next playlist on the stack. The
    currently playing song doesn't change. Do this {'i}cnt{'_} times. The
    default value for {'i}cnt{'_} is 1. 0 means flatten the whole stack.

  {'r}queue  q{'gr}[={'bold}<query>{'_bold}]{'_}
    Adds songs to the end of the queue (current playlist). Without value,
    queues all songs. See `{'c}uamp {'b}h {'w bold}port{'_}` for more info.

  {'r}play-next  queue-next  qn{'gr}[={'bold}<query>{'_bold}]{'_}
    Adds songs after the currently playing in the current playlist. Without
    value, queues all songs. See `{'c}uamp {'b}h {'w bold}port{'_}` for more \
    info.

  {'r}save{'_}
    Triggers save (but saves only if there is change).

  {'r}alias  al{'w}=<alias name>{'gr}[{{<arg1>,...}}]{'_}
    Runs the actions given by the alias. You can pass arguments to the alias.

  {'r}playlist-end-action  playlist-end  pl-end
  spea{'gr}[=<alias name>]{'gr}[{{<arg1>,...}}]{'_}
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

  {'r}restart{'gr}[=<exec-path>]{'_}
    Restart uamp. The optional argument gives path to new uamp binary that will
    take place of the current binary. If not present, uamp will try to get the
    path to the current binary.
    
  {'r}rps  reorder-playlist-stack{'w}=<order>{'_}
    Reorders the playlist stack according to the given order. The order is
    comma separated list of indexes into the stack where the index 0 is the
    current playlist. The first index in the order will be the new current
    playlist and the nest will follow. The indexes that are not in the order
    will be moved to the bottom of the stack without changing their relative
    order.
",
    )
}

fn print_formats_header(color: bool) {
    printmcln!(color, "{'g}Formats:{'_}");
}

fn print_formats_help(color: bool, header: bool) {
    print_port_help(color, header);
    print_query_help(color, true);
    print_filter_help(color, true);
    print_order_help(color, true);
}

fn print_port_help(color: bool, header: bool) {
    if !header {
        print_formats_header(color);
    }

    printmcln!(
        color,
        "  {'w bold}port:{'_}
    May be port number or one of:
      {'r}-  default{'_}
        The default port for uamp (either release or debug).

      {'r}debug{'_}
        Use the debug port: 33284.

      {'r}release  uamp{'_}
        Use the release port: 8267.
"
    );
}

fn print_query_help(color: bool, header: bool) {
    if !header {
        print_formats_header(color);
    }

    printmcln!(
        color,
        "  {'w bold}query:{'_}
    Query is just combination of base, filter, order and unique filter. It has
    the form:
      {'gr}[,{'bold}<base>{'_bold}@]\
      [{'bold}<filter>{'_bold}]\
      [@[{'bold}<order>{'_bold}][@{'bold}<unique>{'_bold}]]{'_}

    See `{'c}uamp {'b}h {'w bold}base filter order unique{'_}` for more info.
"
    );
}

fn print_base_help(color: bool, header: bool) {
    if !header {
        print_formats_header(color);
    }

    printmcln!(
        color,
        "  {'w bold}base:{'_}
    Base is the the source of songs to query. It is basically comma separated
    list of sources of the songs:
      {'gr}[{'bold}<source> {'_bold}[, {'bold}<source>{'_bold} [, ...]]]{'_}

    The source may be any of the following:
      {'r}lib  library{'_}
        All songs in the main library. This is the default.
        
      {'r}tmp  temporary{'_}
        All temporary songs.
        
      {'r}all{'_}
        All songs.
        
      {'r}none{'_}
        No songs.
        
      {'w}<number>{'_}
        Playlist with the given index from the playlist stack. 0 is the current
        playlist, 1 is the playlist after that and so on.
"
    );
}

fn print_filter_help(color: bool, header: bool) {
    if !header {
        print_formats_header(color);
    }

    printmcln!(
        color,
        "  {'w bold}filter:{'_}
    Specifies how to filter songs. These are the kinds of filters:
      {'r}any{'_}
        All songs pass this filter.

      {'r}none{'_}
        No songs pass this filter.

      {'r}s  an  any-name{'w}:<pattern>{'_}
        Matches all songs where either title, artist or album matches.

      {'r}n  tit  title  name{'w}:<pattern>{'_}
        Matches all songs where title matches.

      {'r}p  art  artist  performer  auth  author{'w}:<pattern>{'_}
        Matches all songs where the performer name matches.

      {'r}a  alb  album{'w}:<pattern>{'_}
        Matches all songs where the album name matches.

      {'r}t  trk  track  track-number{'w}:<uint>{'_}
        Matches all songs with the given track number.

      {'r}d  disc{'w}:<uint>{'_}
        Matches all songs with the given disc number. {'w}0{'_} means no disc
        number.

      {'r}y  year{'w}:<int>{'_}
        Matches all songs with release within the given year.

      {'r}g  genre{'w}:<pattern>{'_}
        Matches all songs which genre that matches.

    Instead of `{'i}:{'_}` you can use different separator to change the
    comparison:
      `{'i}={'_}` The string must match exactly.
      `{'i}-{'_}` The string must contain the exact pattern.
      `{'i}:{'_}` The strings converted to lowercase ascii without whitespace
          must match.
      `{'i}~{'_}` The string converted to lowercase asci without whitespace
          must contain the pattern (also converted in the same way).

    You can combine filters using:
      `{'i}+{'_}`    Or - one of the filters must pass.
      `{'i}.{'_}`    And - Both of the filters must pass.
      `{'i}{{  }}{'_}` Brackets to change the precedence.

    `{'i}.{'_}` (And) is evaluated first so `{'i}an:a+an:b.an:c{'_}` is the
    same as `{'i}an:a+[an:b.an:c]{'_}`.

    You can use `{'i}/{'_}` to enclose string literals. Inside the string
    literals you can use `{'i}//{'_}` to escape single /.

    @ is not allowed in filters so you need to use string literal to express
    this character in the filter.

    Example filters:
      `{'i}alb:/smoke+mirrors/+alb:trench{'_}`
"
    );
}

fn print_order_help(color: bool, header: bool) {
    if !header {
        print_formats_header(color);
    }

    printmcln!(
        color,
        "  {'w bold}order:{'_}
    Specifies how to sort songs. It has this form:
      {'gr}[<|>|/|\\|~][+|-]{'w}<ord>{'_}

    The value of ord sets the parameter by which the songs are sorted. Possible
    values are:
      {'r}same{'_}
        Don't change the order.

      {'r}rev  reverse{'_}
        Reverse the songs.

      {'r}rng  rand  random  randomize{'_}
        Shuffle the songs.

      {'r}path{'_}
        Sort by the song path.

      {'r}n  tit  title  name{'_}
        Sort by the song title.

      {'r}p  art  artist  performer  auth  author{'_}
        Sort by the artist.

      {'r}a  alb  album{'_}
        Sort by the album name.

      {'r}t  trk  track  track-number{'_}
        Sort by the track number.

      {'r}d  disc{'_}
        Sort by the disc number.

      {'r}y  year  date{'_}
        Sort by the release date.

      {'r}len  length{'_}
        Sort by the length of the song.

      {'r}g  genre{'_}
        Sort by the genre.

    You can alter the sorting with one of the following options (some
    parameters ignore some of this):
      `{'i}<  /{'_}`    sort in ascending order (this is the default).
      `{'i}>  \\  ~{'_}` sort in descending order.
      `{'i}+{'_}`       use complex sorting.
      `{'i}-{'_}`       use simple sorting.

    If the complexity of the sorting is not set, it will use the default from
    settings.
"
    );
}

fn print_unique_help(color: bool, header: bool) {
    if !header {
        print_formats_header(color);
    }

    printmcln!(
        color,
        "  {'w bold}unique:{'_}
    Specifies if the resulting songs should be filtered so that only unique
    songs will remain. It is one of the following:
      {'r}u  unique  id  path  songs{'_}
        Each song is unique.

      {'r}n  tit  title  name{'_}
        Each song title is unique.

      {'r}p  art  artist  performer  auth  author{'_}
        Artists will be unique.

      {'r}a  alb  album{'_}
        Album names will be unique.

      {'r}t trk  track  track-number{'_}
        Track numbers will be unique.

      {'r}d disc{'_}
        Disc number will be unique.
      
      {'r}y  year  date{'_}
        Years will be unique.

      {'r}len  length{'_}
        Song lengths will be unique.

      {'r}g  genre{'_}
        Genres will be unique.
"
    );
}
