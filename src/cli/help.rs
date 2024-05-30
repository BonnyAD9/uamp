use termal::{gradient, printcln};

use crate::core::Result;

use super::Args;

/// Parses help arguments
pub fn help<'a>(
    args: &mut impl Iterator<Item = &'a str>,
    res: &mut Args,
) -> Result<()> {
    res.should_exit = true;

    let arg = match args.next() {
        Some(a) => a,
        None => {
            help_all();
            return Ok(());
        }
    };

    print_help_header();

    for arg in Some(arg).into_iter().chain(args) {
        match arg {
            "basic" => print_basic_help(),
            "i" | "instance" => print_instance_help(),
            "run" => print_run_help(),
            "all" | "elp" => print_help(),
            "--" => break,
            a => println!("Invalid help option {a}"),
        }
    }

    Ok(())
}

fn print_help() {
    print_basic_help();
    print_instance_help();
    print_run_help();
}

pub fn help_all() {
    print_help_header();
    print_help();
}

pub fn help_instance() {
    print_help_header();
    print_instance_help();
}

pub fn help_run() {
    print_help_header();
    print_run_help();
}

/// Prints the help header
pub fn print_help_header() {
    let v: Option<&str> = option_env!("CARGO_PKG_VERSION");
    printcln!(
        "Welcome in {'i g}uamp{'_} by {}{'_}
Version {}
",
        gradient("BonnyAD9", (250, 50, 170), (180, 50, 240)),
        v.unwrap_or("unknown")
    )
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
    - {'w}--{'_}: print nothing and stop printing help. (this can be used if you
      want to print only the title and version)
"
    )
}

/// Prints the instance help
fn print_instance_help() {
    printcln!(
        "{'g}Instance usage:
  {'c}uamp {'b}i {'gr}[{'dy}flags{'gr}] [{'dr}parameters{'gr}] [--]{'_}
    Sends the parameters as messages to a running instance of uamp using the
    server.

{'g}Instance flags:
  {'y}-h  -?  --help{'_}
    Prints the instance help.

  {'y}-p  --port {'w}<port>{'_}
    Sets the port on which is server of the instance.

  {'y}-a  --address {'w}<address>{'_}
    Sets address of the server of the instance.

{'g}Instance parameters:
  {'r}info nfo{'_}
    Shows the info about the playback of the currently runing instance.

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
    Shuffle the current playlist.

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

  {'r}save{'_}
    Triggers save (but saves only if there is change).
"
    );
}

fn print_run_help() {
    printcln!(
        "{'g}Run usage:
  {'c}uamp {'b}run {'gr}[{'dy}flags{'gr}] [--]{'_}
    Runs new instance of uamp.

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
"
    )
}
