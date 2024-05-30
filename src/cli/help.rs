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
            print_help();
            return Ok(());
        }
    };

    if arg == "--" {
        print_help();
        return Ok(());
    }

    // prepend the first argument back to a iterator
    let arg = [arg];
    let args = arg.iter().copied().chain(args);

    print_help_header();

    for arg in args {
        match arg {
            "basic" => print_basic_help(),
            "i" | "instance" => print_instance_help(),
            a => println!("Invalid help option {a}"),
        }
    }

    Ok(())
}

/// Prints all of help
fn print_help() {
    print_help_header();
    print_basic_help();
    print_instance_help();
}

/// Prints the help header
fn print_help_header() {
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

  {'c}uamp{'_} {'gr}[action] [--] [action] ... [flags]{'_}
    Does the given action.

{'g}Flags:{'_}
  {'y}-p  --port {'w}<port>{'_}
    Sets the port for the server comunication. If used when starting gui, it
    will disable config saves.

  {'y}-a  --address {'w}<address>{'_}
    Sets the server address for the comunication. If used when starting gui, it
    will disable config saves.

{'g}Actions:{'_}
  {'b}i  instance {'w}<instance-action>{'_} {'gr}[--]{'_}
    Operates on a running instance of uamp.

  {'b}h  help  -h  -?  --help{'_}
    Shows help, with no argument whole help, with arguments only help specific
    to the given option.
    Available options are: {'w}basic{'_}, {'w}i instance{'_}
"
    )
}

/// Prints the instance help
fn print_instance_help() {
    printcln!(
        "{'g}Instance actions:
  {'r}info{'_}
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
