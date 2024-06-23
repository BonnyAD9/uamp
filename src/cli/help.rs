use termal::{gradient, printcln};

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
            help_all();
            return;
        }
    };

    help_version();

    let mut show_cmsg = false;
    let mut cmsg_shown = false;

    for arg in Some(arg).into_iter().chain(args) {
        match arg {
            "basic" => print_basic_help(),
            "i" | "instance" => {
                print_instance_help();
                show_cmsg = true;
            }
            "run" => {
                print_run_help();
                show_cmsg = true;
            }
            "all" | "elp" => print_help(),
            "control-message" | "control-msg" | "cmsg" => {
                print_control_messages_help();
                cmsg_shown = true;
            }
            "--" => break,
            a => println!("Invalid help option {a}"),
        }
    }

    if show_cmsg && !cmsg_shown {
        print_control_messages_help();
    }
}

/// Prints the whole help.
pub fn help_all() {
    help_version();
    print_help();
}

/// Prints the help for the instance action.
pub fn help_instance() {
    help_version();
    print_instance_help();
    print_control_messages_help();
}

/// Prints help for the run action.
pub fn help_run() {
    help_version();
    print_run_help();
    print_control_messages_help();
}

/// Prints the help header and version.
pub fn help_version() {
    let v: Option<&str> = option_env!("CARGO_PKG_VERSION");
    printcln!(
        "Welcome in {'i g}uamp{'_} by {}{'_}
Version {}
",
        gradient("BonnyAD9", (250, 50, 170), (180, 50, 240)),
        v.unwrap_or("unknown")
    )
}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

fn print_help() {
    print_basic_help();
    print_instance_help();
    print_run_help();
    print_control_messages_help();
}

/// Prints the basic usage help
fn print_basic_help() {
    printcln!(
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
    Equivalent to `{'b}i {'gr}[arg] {'w}--{'_}`.

  {'y}-R{'gr}[arg]{'_}
    Equivalent to `{'b}run {'gr}[arg] {'w}--{'_}`.

  {'y}-H{'gr}[arg]{'_}
    Equivalent to `{'b}h {'gr}[arg] {'w}--{'_}`.

{'g}Actions:{'_}
  {'b}i  instance {'gr}[instance arguments] [--]{'_}
    Operates on a running instance of uamp.

  {'b}run {'gr}[run arguments] [--]{'_}
    Runs new instance of uamp.

  {'b}h  help {'gr}[help aguments]{'_}
    Shows help. With no arguments whole help, with arguments only help specific
    to the given arguments. Possible arguments are:
    - {'w}all elp{'_}: print whole help.
    - {'w}basic{'_}: print the basic help.
    - {'w}i instance{'_}: print help for {'b}instance{'_}.
    - {'w}run{'_}: print help for {'b}run{'_}.
    - {'w}control-message control-msg cmsg{'_}: print help for control messages.
    - {'w}--{'_}: print nothing and stop printing help. (this can be used if you
      want to print only the title and version)
"
    )
}

/// Prints the instance help
fn print_instance_help() {
    printcln!(
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

  {'r}play p{'w}=<file path>{'_}
    Play the given file in the secondary playlist.
"
    );
}

fn print_run_help() {
    printcln!(
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
"
    )
}

fn print_control_messages_help() {
    printcln!(
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

  {'r}load-songs{'gr}[=<opts>]{'_}
    Look for new songs. This can be modifed with the load options of the form
    `{'i}[r|l][e|n|m]{'_}`:
    - `{'i}r{'_}` also remove songs with invalid path.
    - `{'i}l{'_}` don't remove songs with invalid path.
    - `{'i}e{'_}` add new songs to the end of the queue.
    - `{'i}n{'_}` add new songs after the current song.
    - `{'i}m{'_}` randomly mix the songs in after the current song.

  {'r}shuffle-playlist  shuffle{'_}
    Shuffle the current playlist. The difference from {'r}sort{'w}=rng{'_} is
    that {'r}shuffle{'_} will respect the setting shuffle current.

  {'r}sort-playlist  sort{'w}=<ord>{'_}
    Sorts the current playlist according to the value of {'w}ord{'_}:
    - `{'i}rev{'_}` `{'i}reverse{'_}`                   reverse the playlist.
    - `{'i}rng{'_}` `{'i}rand{'_}` `{'i}random{'_}` `{'i}randomize{'_}` \
      shuffle the playlist.
    - `{'i}path{'_}`                            sort by the path.
    - `{'i}title{'_}` `{'i}name{'_}`                    sort by the song title.
    - `{'i}artist{'_}` `{'i}performer` `{'_}author`     sort by the artist.
    - `{'i}album{'_}`                           sort by the album name.
    - `{'i}track{'_}`                           sort by the track.
    - `{'i}disc{'_}`                            sort by the disc number.
    - `{'i}year{'_}` `{'i}date{'_}`                     sort by the date of \
      release.
    - `{'i}len{'_}` `{'i}length{'_}`                    sort by the length of \
      the song.
    - `{'i}genre{'_}`                           sort by the genre.

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

  {'r}set-playlist  sp{'gr}[=all]{'_}
    Loads all songs into the playlist.

  {'r}push-playlist  push{'gr}[=all]{'_}
    Set the secondary playlist. The primary playlist can be restored with
    {'r}pop{'_}.

  {'r}pop-playlist  pop{'_}
    Remove the secondary playlist and restore the primary playlist.

  {'r}save{'_}
    Triggers save (but saves only if there is change).

  {'r}alias  al{'w}=<alias name>{'_}
    Runs the actions given by the alias.

  {'r}playlist-end-action  playlist-end  pl-end  spea{'gr}[=<alias name>]{'_}
    Sets the playlist end action of the current playlist to actions specified
    by the alias.
"
    )
}
