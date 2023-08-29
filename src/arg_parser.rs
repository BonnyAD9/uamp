use crate::{messenger, uamp_app::ControlMsg};
use eyre::{Report, Result};
use termal::{gradient, printcln};

/// Action that can be done with cli
pub enum Action {
    Message(messenger::Message),
}

/// Contains the CLI arguments values
#[derive(Default)]
pub struct Args {
    pub actions: Vec<Action>,
    pub should_exit: bool,
    pub should_run: bool,
}

/// Returns error with the given message
macro_rules! ret_err {
    ($($arg:tt)*) => {
        return Err(Report::msg(format!($($arg)*)))
    };
}

/// Gets the next value from a iterator, returns error when there is no value.
/// The two last arguments are used to produce the error message
/// - the first argument says the option after which the error occured
/// - the second argument explains what value was expected
///
/// It can also parse and validate the value
///
/// # Usage
/// ```
/// // gets the next value from iterator
/// let val = next!(iterator, "option", "name");
///
/// // gets the value and parses it into f32
/// let val = next!(f32, iterator, "option", "f32");
///
/// // gets the value, parses it into f32 and validates it
/// let val = next!(
///     f32,
///     iterator,
///     "option",
///     "f32 in range 0..=1",
///     |v| (0.0..=1.).contains(v)
/// );
/// ```
macro_rules! next {
    ($iter:ident, $after:literal, $what:literal) => {
        match $iter.next() {
            Some(a) => a,
            None => {
                ret_err!("Expected {} action after '{}'", $what, $after)
            }
        }
    };

    ($typ:ident, $iter:ident, $after:literal, $what:literal) => {
        match $iter
            .next()
            .ok_or(Report::msg("next is none"))
            .and_then(|v| v.parse::<$typ>().map_err(|e| e.into()))
        {
            Ok(a) => a,
            Err(_) => {
                ret_err!("Expected {} action after '{}'", $what, $after)
            }
        }
    };

    ($typ:ident, $iter:ident, $after:literal, $what:literal, $val:expr) => {
        match $iter
            .next()
            .ok_or(Report::msg("next is none"))
            .and_then(|v| v.parse::<$typ>().map_err(|e| e.into()))
        {
            Ok(a) if { $val }(&a) => a,
            _ => {
                ret_err!("Expected {} action after '{}'", $what, $after)
            }
        }
    };
}

/// creates expression that checks whether a variable starts with any of the
/// strings
///
/// # Example
/// ```
/// let val = "arg2=hi";
/// if starts!(val, "arg1" | "arg2") {
///     // now we know that `val` starts either with `"arg1"` or `"arg2"`
/// }
/// ```
macro_rules! starts {
    ($i:ident, $($s:literal)|+) => {{
        $($i.starts_with($s))||+
    }};
}

/// Adds control message as action to be sent to existing instance
macro_rules! msg_control {
    ($arg:ident, $msg:ident) => {
        $arg.actions
            .push(Action::Message(messenger::Message::Control(
                ControlMsg::$msg,
            )))
    };

    ($arg:ident, $msg:ident $t:tt) => {
        $arg.actions
            .push(Action::Message(messenger::Message::Control(
                ControlMsg::$msg$t,
            )))
    };
}

/// Parses the string value, returns error if it cannot be parsed. The second
/// argument is used to produce the error message
///
/// It can also validate the value
///
/// # Examples
/// ```
/// // parses the `&str` to `f32`
/// let val = parse!(f32, "f32", "3.1415");
///
/// // parses the `&str` to `f32` and validates it
/// let val = parse!(
///     f32,
///     "f32 in range 0..=1",
///     "3.1415",
///     |v| (0.0..=1.).contains(v)
/// );
/// ```
macro_rules! parse {
    ($t:ty, $msg:literal, $s:expr) => {
        match $s
            .ok_or(Report::msg("none"))
            .and_then(|s| s.parse::<$t>().map_err(|e| e.into()))
        {
            Ok(a) => a,
            _ => ret_err!("Expected {}", $msg),
        }
    };

    ($t:ty, $msg:literal, $s:expr, $val:expr) => {
        match $s
            .ok_or(Report::msg("none"))
            .and_then(|s| s.parse::<$t>().map_err(|e| e.into()))
        {
            Ok(a) if { $val }(&a) => a,
            _ => ret_err!("Expected {}", $msg),
        }
    };
}

/// Parses the string value, returns None if the passed value is none, returns
/// error if it cannot be parsed. The second argument is used to produce the
/// error message.
///
/// It can also validate the value
///
/// # Examples
/// ```
/// // parses the `&str` to `f32`
/// let val = parse!(f32, "f32", Some("3.1415"));
///
/// // parses the `&str` to `f32` and validates it
/// let val = parse!(
///     f32,
///     "f32 in range 0..=1",
///     None,
///     |v| (0.0..=1.).contains(v)
/// );
/// ```
macro_rules! maybe_parse {
    ($t:ty, $msg:literal, $s:expr) => {
        match $s.map(|s| s.parse::<$t>()) {
            Some(Err(_)) => ret_err!("Expected {}", $msg),
            Some(Ok(a)) => Some(a),
            _ => None,
        }
    };

    ($t:ty, $msg:literal, $s:expr, $val:expr) => {
        match $s.map(|s| s.parse::<$t>()) {
            Some(Err(_)) => ret_err!("Expected {}", $msg),
            Some(Ok(a)) if { $val }(&a) => Some(a),
            None => None,
            _ => ret_err!("Expected {}", $msg),
        }
    };
}

/// Gets supstring immidietly following the substring `p` in `s`
fn get_after<'a>(s: &'a str, p: &str) -> Option<&'a str> {
    let mut i = s.split(p);
    i.next();
    i.next()
}

/// Parses the CLI arguments and returns the parsed arguments
pub fn parse_args<'a>(args: impl Iterator<Item = &'a str>) -> Result<Args> {
    let mut res = Args::default();

    let mut args = args.skip(1);

    while let Some(a) = args.next() {
        match a {
            "i" | "instance" => instance(&mut args, &mut res)?,
            "h" | "help" | "-h" | "--help" | "-?" => {
                help(&mut args, &mut res)?
            }
            a => ret_err!("Invalid argument '{a}'"),
        }
    }

    Ok(res)
}

/// Parses the instance action arguments
fn instance<'a>(
    args: &mut impl Iterator<Item = &'a str>,
    res: &mut Args,
) -> Result<()> {
    res.should_exit = true;
    let a = next!(args, "instance", "instance action");

    match a {
        "play-pause" | "pp" => msg_control!(res, PlayPause),
        "volume-up" | "vol-up" | "vu" => msg_control!(res, VolumeUp),
        "volume-down" | "vol-down" | "vd" => msg_control!(res, VolumeDown),
        "next-song" | "ns" => msg_control!(res, NextSong),
        "previous-song" | "prev-song" | "ps" => msg_control!(res, PrevSong),
        v if starts!(v, "volume" | "vol" | "v") => {
            let vol = parse!(
                f32,
                "expected float in range 0..=1",
                get_after(v, "="),
                |v| (0.0..=1.).contains(v)
            );
            msg_control!(res, SetVolume(vol));
        }
        "mute" => msg_control!(res, ToggleMute),
        "find-songs" | "fs" => msg_control!(res, FindSongs),
        "--" => ret_err!("Expected instance action after 'instance'"),
        _ => ret_err!("Invalid argument '{a}' after 'instance'"),
    }

    Ok(())
}

//==================================<< HELP >>===============================//

/// Parses help arguments
fn help<'a>(
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
    let args = arg.iter().map(|s| *s).chain(args);

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
    printcln!(
        "Welcome in {'i g}uamp{'_} by {}{'_}
Version 0.1.1
",
        gradient("BonnyAD9", (250, 50, 170), (180, 50, 240))
    )
}

/// Prints the basic usage help
fn print_basic_help() {
    printcln!(
        "{'g}Usage:{'_}
  {'w bold}uamp{'_}
    starts the gui of the player

  {'w bold}uamp{'_} {'gr}[action] [--] [action] ...{'_}
    does the given action

{'g}Actions:{'_}
  {'y}i  instance {'bold w}<instance-action>{'_} {'gr}[--]{'_}
    operates on a running instance of uamp

  {'y}h  help  -h  -?  --help{'_}
    shows help, with no argument whole help, with arguments only help specific
    to the given option.
    Available options are: {'bold w}basic{'_}, {'bold w}i instance{'_}
"
    )
}

/// Prints the instance help
fn print_instance_help() {
    printcln!(
        "{'g}Instance actions:
  {'y}pp  play-pause{'_}
    toggle between the states playing and paused

  {'y}vu  vol-up  volume-up{'_}
    increase the volume by the default amount

  {'y}vd  vol-down  volume-down{'_}
    decrease the volume by the default amount

  {'y}ns  next-song{'_}
    go to the next song

  {'y}ps  prev-song  previous-song{'_}
    go to the previous song

  {'y}v  vol  volume{'bold w}=<value>{'_}
    set the volume to the given {'bold w}value{'_}, {'bold w}value{'_} must be
    in range from 0 to 1

  {'y}mute{'_}
    toggle mute/unmute

  {'y}fs  find-songs{'_}
    look for new songs
"
    )
}
